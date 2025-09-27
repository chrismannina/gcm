use anyhow::{Context, Result};
use git2::{DiffOptions, Repository, StatusOptions};

pub struct GitAnalyzer {
    repo: Repository,
}

impl GitAnalyzer {
    pub fn new() -> Result<Self> {
        let repo = Repository::open_from_env()
            .context("Failed to open git repository. Make sure you're in a git repository.")?;
        Ok(Self { repo })
    }

    pub fn has_staged_changes(&self) -> Result<bool> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(false);

        let statuses = self.repo.statuses(Some(&mut opts))
            .context("Failed to get repository status")?;

        Ok(statuses.iter().any(|entry| {
            let status = entry.status();
            status.contains(git2::Status::INDEX_NEW) ||
            status.contains(git2::Status::INDEX_MODIFIED) ||
            status.contains(git2::Status::INDEX_DELETED) ||
            status.contains(git2::Status::INDEX_RENAMED) ||
            status.contains(git2::Status::INDEX_TYPECHANGE)
        }))
    }

    pub fn stage_all_changes(&self) -> Result<()> {
        let mut index = self.repo.index()
            .context("Failed to get repository index")?;

        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
            .context("Failed to stage all changes")?;

        index.write()
            .context("Failed to write index")?;

        Ok(())
    }

    pub fn get_staged_diff(&self) -> Result<StagedChanges> {
        let head = self.repo.head()
            .context("Failed to get HEAD reference")?;

        let head_tree = head.peel_to_tree()
            .context("Failed to get HEAD tree")?;

        let mut index = self.repo.index()
            .context("Failed to get repository index")?;

        let index_oid = index.write_tree()
            .context("Failed to write index tree")?;

        let index_tree = self.repo.find_tree(index_oid)
            .context("Failed to find index tree")?;

        let mut diff_opts = DiffOptions::new();
        let diff = self.repo.diff_tree_to_tree(
            Some(&head_tree),
            Some(&index_tree),
            Some(&mut diff_opts),
        ).context("Failed to generate diff")?;

        let mut diff_text = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            let content = std::str::from_utf8(line.content()).unwrap_or("");
            diff_text.push_str(content);
            true
        }).context("Failed to format diff")?;

        // Get file stats
        let stats = diff.stats()
            .context("Failed to get diff stats")?;

        // Get changed files with their status
        let mut changed_files = Vec::new();
        diff.foreach(
            &mut |delta, _progress| {
                let file_path = delta.new_file().path()
                    .or_else(|| delta.old_file().path())
                    .and_then(|p| p.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let status = match delta.status() {
                    git2::Delta::Added => "added",
                    git2::Delta::Deleted => "deleted",
                    git2::Delta::Modified => "modified",
                    git2::Delta::Renamed => "renamed",
                    _ => "changed",
                }.to_string();

                changed_files.push(FileChange {
                    path: file_path,
                    status,
                });
                true
            },
            None,
            None,
            None,
        ).context("Failed to iterate over diff")?;

        Ok(StagedChanges {
            diff_text,
            files_changed: changed_files,
            insertions: stats.insertions(),
            deletions: stats.deletions(),
        })
    }

    pub fn get_project_context(&self) -> Result<ProjectContext> {
        let workdir = self.repo.workdir()
            .context("Failed to get repository working directory")?;

        let mut context = ProjectContext {
            project_name: workdir
                .file_name()
                .and_then(|name| name.to_str())
                .map(String::from),
            readme_content: None,
            package_json: None,
            cargo_toml: None,
            recent_commits: Vec::new(),
        };

        // Try to read README
        let readme_path = workdir.join("README.md");
        if readme_path.exists() {
            context.readme_content = std::fs::read_to_string(&readme_path)
                .ok()
                .map(|content| content.lines().take(50).collect::<Vec<_>>().join("\n"));
        }

        // Try to read package.json
        let package_json_path = workdir.join("package.json");
        if package_json_path.exists() {
            context.package_json = std::fs::read_to_string(&package_json_path).ok();
        }

        // Try to read Cargo.toml
        let cargo_toml_path = workdir.join("Cargo.toml");
        if cargo_toml_path.exists() {
            context.cargo_toml = std::fs::read_to_string(&cargo_toml_path).ok();
        }

        // Get recent commits
        let mut revwalk = self.repo.revwalk()
            .context("Failed to create revwalk")?;

        revwalk.push_head()
            .context("Failed to push HEAD to revwalk")?;

        for (i, oid) in revwalk.enumerate() {
            if i >= 10 {
                break;
            }

            if let Ok(oid) = oid {
                if let Ok(commit) = self.repo.find_commit(oid) {
                    if let Some(msg) = commit.summary() {
                        context.recent_commits.push(msg.to_string());
                    }
                }
            }
        }

        Ok(context)
    }

    pub fn commit_with_message(&self, message: &str) -> Result<()> {
        let signature = self.repo.signature()
            .context("Failed to get git signature")?;

        let head = self.repo.head()
            .context("Failed to get HEAD reference")?;

        let head_commit = head.peel_to_commit()
            .context("Failed to get HEAD commit")?;

        let mut index = self.repo.index()
            .context("Failed to get repository index")?;

        let tree_id = index.write_tree()
            .context("Failed to write tree")?;

        let tree = self.repo.find_tree(tree_id)
            .context("Failed to find tree")?;

        self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&head_commit],
        ).context("Failed to create commit")?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct ProjectContext {
    pub project_name: Option<String>,
    pub readme_content: Option<String>,
    pub package_json: Option<String>,
    pub cargo_toml: Option<String>,
    pub recent_commits: Vec<String>,
}

#[derive(Debug)]
pub struct StagedChanges {
    pub diff_text: String,
    pub files_changed: Vec<FileChange>,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Debug)]
pub struct FileChange {
    pub path: String,
    pub status: String,
}