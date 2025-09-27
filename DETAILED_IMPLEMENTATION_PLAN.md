# GCM Detailed Implementation Plan
## Transform gcm into a Best-in-Class Developer Tool

### Executive Summary

This document provides a comprehensive, step-by-step implementation guide to transform gcm from a functional prototype into a must-have developer utility. Each phase includes specific tasks, code structures, timelines, and acceptance criteria.

**Target Timeline**: 6-9 months to full implementation
**Success Metrics**: 10k+ weekly users, <2s response time, 90%+ message acceptance

---

## 🏗️ Phase 1: User Experience Excellence (Weeks 1-8)
**Goal**: Make gcm feel polished and reliable for everyday use

### 1.1 Enhanced CLI Interface (Weeks 1-3)

#### Current State
```rust
// src/cli.rs - Limited options
pub struct Cli {
    pub commit: bool,
    pub all: bool,
    pub number: u8,
    pub amend: bool,
    pub model: Option<String>,
}
```

#### Target Implementation
```rust
// src/cli/mod.rs - Enhanced CLI structure
pub mod commands;
pub mod interactive;
pub mod output;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gcm", version, about = "Generate intelligent git commit messages")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Global options
    #[arg(short, long, help = "Commit immediately after generating")]
    pub commit: bool,

    #[arg(short, long, help = "Stage all changes first")]
    pub all: bool,

    #[arg(short, long, default_value = "1", help = "Number of suggestions")]
    pub number: u8,

    #[arg(long, help = "Show detailed explanation")]
    pub explain: bool,

    #[arg(long, help = "Preview without executing")]
    pub dry_run: bool,

    #[arg(long, help = "Use specific model")]
    pub model: Option<String>,

    #[arg(short, long, help = "Verbose output")]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate commit message (default)
    Generate {
        #[arg(long, help = "Use interactive mode")]
        interactive: bool,

        #[arg(long, help = "Quick mode - skip context analysis")]
        quick: bool,

        #[arg(long, help = "Use specific template")]
        template: Option<String>,
    },

    /// Setup and configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Show project analysis
    Status {
        #[arg(long, help = "Show detailed analysis")]
        detailed: bool,
    },

    /// Interactive setup wizard
    Setup,

    /// Manage templates
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    /// Set configuration value
    Set { key: String, value: String },
    /// Reset to defaults
    Reset,
    /// Validate configuration
    Validate,
}

#[derive(Subcommand)]
pub enum TemplateAction {
    /// List available templates
    List,
    /// Show template details
    Show { name: String },
    /// Create new template
    New { name: String },
    /// Edit existing template
    Edit { name: String },
}
```

#### Implementation Steps

**Week 1: CLI Structure**
1. Create `src/cli/` module with subcommands
2. Implement enhanced argument parsing
3. Add command validation and help text
4. Create interactive command dispatcher

**Week 2: Command Implementation**
1. Implement `generate` command with all modes
2. Add `config` management commands
3. Create `status` analysis command
4. Build `setup` wizard command

**Week 3: Testing & Polish**
1. Add comprehensive CLI tests
2. Implement shell completions
3. Create man pages
4. Add command aliases and shortcuts

#### Acceptance Criteria
- [ ] All new CLI commands work correctly
- [ ] Help text is comprehensive and helpful
- [ ] Shell completions work in bash/zsh/fish
- [ ] Backward compatibility maintained
- [ ] 100% test coverage for CLI module

### 1.2 Intelligent Error Handling (Weeks 2-4)

#### Current Issues
```rust
// src/main.rs - Poor error messages
eprintln!("{}", "No staged changes found. Use 'git add' to stage changes or use -a flag.".red());
```

#### Enhanced Error System
```rust
// src/errors/mod.rs - Comprehensive error handling
use colored::*;
use std::fmt;

#[derive(Debug)]
pub enum GcmError {
    NoStagedChanges { unstaged_files: Vec<String>, suggestions: Vec<String> },
    ApiError { provider: String, message: String, retry_suggestion: Option<String> },
    GitError { operation: String, error: String, fix_suggestion: Option<String> },
    ConfigError { file: String, issue: String, fix_steps: Vec<String> },
    NetworkError { endpoint: String, error: String },
}

impl GcmError {
    pub fn display_rich(&self) {
        match self {
            GcmError::NoStagedChanges { unstaged_files, suggestions } => {
                println!("{}", "❌ No staged changes found".red().bold());

                if !unstaged_files.is_empty() {
                    println!("\n{}", "📊 Unstaged files detected:".yellow());
                    for file in unstaged_files.iter().take(5) {
                        println!("   • {}", file.dimmed());
                    }
                    if unstaged_files.len() > 5 {
                        println!("   ... and {} more", unstaged_files.len() - 5);
                    }
                }

                println!("\n{}", "💡 Try one of these:".green());
                for suggestion in suggestions {
                    println!("   {}", suggestion);
                }
            }

            GcmError::ApiError { provider, message, retry_suggestion } => {
                println!("{} {} API Error", "❌".red(), provider);
                println!("   {}", message.dimmed());

                if let Some(suggestion) = retry_suggestion {
                    println!("\n{} {}", "💡".green(), suggestion);
                }
            }

            // ... other error types
        }
    }
}

// src/git.rs - Enhanced git status analysis
pub struct GitStatusAnalysis {
    pub staged_files: Vec<FileInfo>,
    pub unstaged_files: Vec<FileInfo>,
    pub untracked_files: Vec<FileInfo>,
    pub suggestions: Vec<String>,
}

impl GitAnalyzer {
    pub fn analyze_status(&self) -> Result<GitStatusAnalysis> {
        // Detailed analysis with suggestions
        let status = self.repo.statuses(Some(&mut StatusOptions::new()))?;

        let mut analysis = GitStatusAnalysis::default();

        for entry in status.iter() {
            let file_info = FileInfo {
                path: entry.path().unwrap_or("unknown").to_string(),
                status: entry.status(),
                lines_changed: self.count_lines_changed(&entry.path())?,
            };

            match entry.status() {
                s if s.contains(git2::Status::INDEX_NEW | git2::Status::INDEX_MODIFIED) => {
                    analysis.staged_files.push(file_info);
                }
                s if s.contains(git2::Status::WT_NEW | git2::Status::WT_MODIFIED) => {
                    analysis.unstaged_files.push(file_info);
                }
                // ... other statuses
            }
        }

        // Generate smart suggestions
        analysis.suggestions = self.generate_suggestions(&analysis);

        Ok(analysis)
    }

    fn generate_suggestions(&self, analysis: &GitStatusAnalysis) -> Vec<String> {
        let mut suggestions = Vec::new();

        if analysis.staged_files.is_empty() && !analysis.unstaged_files.is_empty() {
            suggestions.push("git add <files>  # Stage specific files".to_string());
            suggestions.push("gcm --all        # Stage all and generate".to_string());

            if analysis.unstaged_files.len() <= 3 {
                let files: Vec<_> = analysis.unstaged_files.iter()
                    .map(|f| &f.path).collect();
                suggestions.push(format!("git add {}   # Stage these files", files.join(" ")));
            }
        }

        suggestions
    }
}
```

#### Implementation Steps

**Week 2: Error Type Design**
1. Define comprehensive error enum
2. Create rich display formatting
3. Add context gathering for errors
4. Implement suggestion generation

**Week 3: Git Integration**
1. Enhanced git status analysis
2. Suggestion generation logic
3. File change quantification
4. Repository health checks

**Week 4: Error Recovery**
1. Auto-fix common issues
2. Interactive error resolution
3. Error logging and metrics
4. User feedback collection

### 1.3 Rich Terminal Output (Weeks 3-5)

#### Enhanced Output System
```rust
// src/output/mod.rs - Rich terminal UI
use colored::*;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use console::{style, Emoji};

pub struct TerminalUI {
    multi_progress: MultiProgress,
    verbose: bool,
}

impl TerminalUI {
    pub fn new(verbose: bool) -> Self {
        Self {
            multi_progress: MultiProgress::new(),
            verbose,
        }
    }

    pub fn show_analysis_progress(&self) -> ProgressHandle {
        let pb = self.multi_progress.add(ProgressBar::new_spinner());
        pb.set_style(ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap());
        pb.set_message("Analyzing changes...");

        ProgressHandle { pb }
    }

    pub fn display_commit_suggestions(&self, suggestions: &[CommitSuggestion]) {
        println!("\n{} Generated commit messages:", Emoji("✨", "").green());

        for (i, suggestion) in suggestions.iter().enumerate() {
            self.display_suggestion(i + 1, suggestion);
        }
    }

    fn display_suggestion(&self, number: usize, suggestion: &CommitSuggestion) {
        let prefix = format!("{}.", number).bold().cyan();
        println!("\n{} {}", prefix, suggestion.message.bright_white());

        if let Some(reasoning) = &suggestion.reasoning {
            println!("   {}", reasoning.dimmed());
        }

        if self.verbose {
            self.display_suggestion_details(suggestion);
        }
    }

    pub fn display_diff_summary(&self, changes: &StagedChanges) {
        println!("\n{} File changes:", Emoji("📁", ""));

        for file in &changes.files_changed {
            let status_icon = match file.status.as_str() {
                "added" => "🆕",
                "modified" => "✏️",
                "deleted" => "🗑️",
                "renamed" => "📝",
                _ => "📄",
            };

            println!("   {} {} ({})",
                     status_icon,
                     file.path.bright_white(),
                     file.status.dimmed());
        }

        let stats = format!("+{} -{}", changes.insertions, changes.deletions);
        println!("   {}", stats.green());
    }
}

#[derive(Debug)]
pub struct CommitSuggestion {
    pub message: String,
    pub reasoning: Option<String>,
    pub confidence: f32,
    pub change_type: ChangeType,
}
```

#### Implementation Steps

**Week 3: Terminal UI Framework**
1. Create rich output system
2. Add progress indicators
3. Implement colored formatting
4. Add emoji and Unicode support

**Week 4: Interactive Features**
1. Build selection interfaces
2. Add confirmation prompts
3. Create edit mode
4. Implement keyboard shortcuts

**Week 5: Polish & Testing**
1. Cross-platform terminal testing
2. Color scheme customization
3. Accessibility improvements
4. Performance optimization

### 1.4 First-Time User Experience (Weeks 4-6)

#### Setup Wizard Implementation
```rust
// src/setup/mod.rs - Interactive setup
use dialoguer::{Input, Select, Confirm, Password};

pub struct SetupWizard {
    config: Config,
}

impl SetupWizard {
    pub async fn run() -> Result<Config> {
        println!("{}", "🚀 Welcome to gcm!".bright_cyan().bold());
        println!("Let's get you set up in just a few steps.\n");

        let mut wizard = Self { config: Config::default() };

        wizard.detect_environment().await?;
        wizard.configure_api_provider().await?;
        wizard.set_preferences().await?;
        wizard.test_configuration().await?;
        wizard.save_configuration().await?;

        println!("\n{} Setup complete! Try running: {}",
                 "✅".green(),
                 "gcm".bright_white().bold());

        Ok(wizard.config)
    }

    async fn detect_environment(&mut self) -> Result<()> {
        println!("{} Detecting your environment...", "🔍".cyan());

        // Auto-detect project type
        if Path::new("package.json").exists() {
            println!("   {} JavaScript/TypeScript project detected", "📦".green());
            self.config.project_type = Some(ProjectType::JavaScript);
        } else if Path::new("Cargo.toml").exists() {
            println!("   {} Rust project detected", "🦀".green());
            self.config.project_type = Some(ProjectType::Rust);
        }

        // Check for existing API keys
        if env::var("OPENAI_API_KEY").is_ok() {
            println!("   {} OpenAI API key found", "🔑".green());
            self.config.provider = "openai".to_string();
        } else if env::var("ANTHROPIC_API_KEY").is_ok() {
            println!("   {} Anthropic API key found", "🔑".green());
            self.config.provider = "anthropic".to_string();
        }

        Ok(())
    }

    async fn configure_api_provider(&mut self) -> Result<()> {
        if self.config.api_key.is_empty() {
            println!("\n{} API Configuration", "⚙️".cyan());

            let providers = vec!["OpenAI (GPT-4)", "Anthropic (Claude)", "Other"];
            let selection = Select::new()
                .with_prompt("Choose your AI provider")
                .items(&providers)
                .default(0)
                .interact()?;

            match selection {
                0 => self.setup_openai().await?,
                1 => self.setup_anthropic().await?,
                2 => self.setup_custom_provider().await?,
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    async fn test_configuration(&mut self) -> Result<()> {
        println!("\n{} Testing configuration...", "🧪".cyan());

        let test_client = LLMClient::new(
            self.config.model.clone(),
            self.config.api_key.clone(),
        );

        match test_client.test_connection().await {
            Ok(_) => println!("   {} Connection successful!", "✅".green()),
            Err(e) => {
                println!("   {} Connection failed: {}", "❌".red(), e);

                if Confirm::new()
                    .with_prompt("Would you like to reconfigure?")
                    .interact()? {
                    return self.configure_api_provider().await;
                }
            }
        }

        Ok(())
    }
}
```

---

## 🧠 Phase 2: Intelligent Commit Generation (Weeks 5-12)
**Goal**: Generate commit messages that feel human-written and contextually aware

### 2.1 Enhanced Context Analysis (Weeks 5-8)

#### Context System Architecture
```rust
// src/intelligence/context.rs - Advanced context gathering
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedProjectContext {
    // Project identification
    pub project_name: Option<String>,
    pub project_type: ProjectType,
    pub language_breakdown: HashMap<String, f32>,

    // Git context
    pub branch_info: BranchInfo,
    pub recent_commits: Vec<CommitPattern>,
    pub remote_info: Option<RemoteInfo>,

    // Code context
    pub frameworks: Vec<Framework>,
    pub dependencies: Vec<Dependency>,
    pub architecture_patterns: Vec<ArchitecturePattern>,

    // Team context
    pub team_conventions: TeamConventions,
    pub commit_patterns: CommitPatterns,

    // Change context
    pub change_impact: ChangeImpact,
    pub related_files: Vec<String>,
    pub test_coverage: TestCoverage,
}

#[derive(Debug)]
pub struct BranchInfo {
    pub name: String,
    pub branch_type: BranchType, // feature, hotfix, release, etc.
    pub base_branch: Option<String>,
    pub ahead_behind: (usize, usize),
    pub issue_reference: Option<IssueReference>,
}

#[derive(Debug)]
pub enum BranchType {
    Feature { scope: Option<String> },
    Hotfix { severity: Severity },
    Release { version: String },
    Bugfix { issue_id: Option<String> },
    Chore,
    Docs,
    Experiment,
}

#[derive(Debug)]
pub struct ChangeImpact {
    pub breaking_changes: Vec<BreakingChange>,
    pub api_changes: Vec<ApiChange>,
    pub performance_impact: PerformanceImpact,
    pub security_implications: Vec<SecurityImplication>,
    pub user_facing_changes: Vec<UserFacingChange>,
}

impl ContextGatherer {
    pub async fn gather_enhanced_context(&self) -> Result<EnhancedProjectContext> {
        let mut context = EnhancedProjectContext::default();

        // Parallel context gathering for performance
        let (
            git_context,
            code_context,
            team_context,
            change_context,
        ) = tokio::join!(
            self.gather_git_context(),
            self.gather_code_context(),
            self.gather_team_context(),
            self.analyze_change_impact(),
        );

        context.merge_git_context(git_context?);
        context.merge_code_context(code_context?);
        context.merge_team_context(team_context?);
        context.merge_change_context(change_context?);

        Ok(context)
    }

    async fn gather_git_context(&self) -> Result<GitContext> {
        let mut git_context = GitContext::default();

        // Branch analysis
        git_context.branch_info = self.analyze_current_branch().await?;

        // Recent commit pattern analysis
        git_context.commit_patterns = self.analyze_commit_patterns().await?;

        // Remote and PR context
        if let Ok(remote) = self.get_remote_info().await {
            git_context.remote_info = Some(remote);

            // Try to extract issue references
            if let Some(issue_ref) = self.extract_issue_reference(&git_context.branch_info.name) {
                git_context.branch_info.issue_reference = Some(issue_ref);
            }
        }

        Ok(git_context)
    }

    async fn analyze_change_impact(&self) -> Result<ChangeImpact> {
        let staged_changes = self.git_analyzer.get_staged_diff()?;
        let mut impact = ChangeImpact::default();

        // Analyze for breaking changes
        impact.breaking_changes = self.detect_breaking_changes(&staged_changes).await?;

        // API change detection
        impact.api_changes = self.detect_api_changes(&staged_changes).await?;

        // Performance impact analysis
        impact.performance_impact = self.analyze_performance_impact(&staged_changes).await?;

        // Security analysis
        impact.security_implications = self.analyze_security_impact(&staged_changes).await?;

        // User-facing change detection
        impact.user_facing_changes = self.detect_user_facing_changes(&staged_changes).await?;

        Ok(impact)
    }
}
```

### 2.2 Semantic Code Understanding (Weeks 6-9)

#### AST Analysis Implementation
```rust
// src/intelligence/semantic.rs - Code semantic analysis
use tree_sitter::{Language, Parser, Query, QueryCursor};

pub struct SemanticAnalyzer {
    parsers: HashMap<String, Parser>,
    queries: HashMap<String, Query>,
}

impl SemanticAnalyzer {
    pub fn new() -> Result<Self> {
        let mut analyzer = Self {
            parsers: HashMap::new(),
            queries: HashMap::new(),
        };

        // Initialize parsers for different languages
        analyzer.init_rust_parser()?;
        analyzer.init_javascript_parser()?;
        analyzer.init_python_parser()?;
        analyzer.init_go_parser()?;

        Ok(analyzer)
    }

    pub async fn analyze_changes(&self, changes: &StagedChanges) -> Result<SemanticAnalysis> {
        let mut analysis = SemanticAnalysis::default();

        for file_change in &changes.files_changed {
            if let Some(language) = self.detect_language(&file_change.path) {
                let file_analysis = self.analyze_file_change(file_change, &language).await?;
                analysis.merge_file_analysis(file_analysis);
            }
        }

        Ok(analysis)
    }

    async fn analyze_file_change(&self, file_change: &FileChange, language: &str) -> Result<FileAnalysis> {
        let mut analysis = FileAnalysis {
            file_path: file_change.path.clone(),
            language: language.to_string(),
            ..Default::default()
        };

        // Parse the diff to extract added/removed code
        let code_changes = self.extract_code_changes(file_change)?;

        for change in code_changes {
            match change.change_type {
                CodeChangeType::Added => {
                    analysis.additions.extend(self.analyze_added_code(&change.content, language)?);
                }
                CodeChangeType::Removed => {
                    analysis.removals.extend(self.analyze_removed_code(&change.content, language)?);
                }
                CodeChangeType::Modified => {
                    analysis.modifications.extend(self.analyze_modified_code(&change, language)?);
                }
            }
        }

        // High-level pattern detection
        analysis.patterns = self.detect_patterns(&analysis)?;

        Ok(analysis)
    }

    fn analyze_added_code(&self, code: &str, language: &str) -> Result<Vec<CodeElement>> {
        let parser = self.parsers.get(language).unwrap();
        let tree = parser.parse(code, None).unwrap();
        let root_node = tree.root_node();

        let mut elements = Vec::new();

        // Query for different code elements
        let queries = &[
            ("function_definition", "function"),
            ("class_definition", "class"),
            ("interface_definition", "interface"),
            ("struct_definition", "struct"),
            ("enum_definition", "enum"),
            ("import_statement", "import"),
            ("export_statement", "export"),
        ];

        for (query_name, element_type) in queries {
            if let Some(query) = self.queries.get(&format!("{}_{}", language, query_name)) {
                let mut cursor = QueryCursor::new();
                let matches = cursor.matches(query, root_node, code.as_bytes());

                for match_ in matches {
                    for capture in match_.captures {
                        let text = capture.node.utf8_text(code.as_bytes())?;
                        elements.push(CodeElement {
                            element_type: element_type.to_string(),
                            name: self.extract_name(capture.node, code.as_bytes())?,
                            signature: text.to_string(),
                            visibility: self.extract_visibility(capture.node, code.as_bytes())?,
                        });
                    }
                }
            }
        }

        Ok(elements)
    }

    fn detect_patterns(&self, analysis: &FileAnalysis) -> Result<Vec<ChangePattern>> {
        let mut patterns = Vec::new();

        // Pattern: New feature detection
        if analysis.additions.iter().any(|e| e.element_type == "function" && e.visibility == "public") {
            patterns.push(ChangePattern::NewFeature {
                functions: analysis.additions.iter()
                    .filter(|e| e.element_type == "function")
                    .map(|e| e.name.clone())
                    .collect(),
            });
        }

        // Pattern: Refactoring detection
        if analysis.removals.len() > 0 && analysis.additions.len() > 0 {
            let removed_names: HashSet<_> = analysis.removals.iter().map(|e| &e.name).collect();
            let added_names: HashSet<_> = analysis.additions.iter().map(|e| &e.name).collect();

            if removed_names.intersection(&added_names).count() == 0 {
                patterns.push(ChangePattern::Refactoring {
                    scope: self.determine_refactoring_scope(analysis),
                });
            }
        }

        // Pattern: Bug fix detection
        if analysis.modifications.iter().any(|m| self.looks_like_bug_fix(&m.before, &m.after)) {
            patterns.push(ChangePattern::BugFix {
                area: self.determine_bug_fix_area(analysis),
            });
        }

        // Pattern: Performance optimization
        if self.detect_performance_changes(analysis) {
            patterns.push(ChangePattern::PerformanceOptimization);
        }

        // Pattern: Security fix
        if self.detect_security_changes(analysis) {
            patterns.push(ChangePattern::SecurityFix);
        }

        Ok(patterns)
    }
}

#[derive(Debug)]
pub enum ChangePattern {
    NewFeature { functions: Vec<String> },
    BugFix { area: String },
    Refactoring { scope: RefactoringScope },
    PerformanceOptimization,
    SecurityFix,
    Documentation,
    Testing,
    Configuration,
    Dependency,
}
```

### 2.3 Advanced LLM Integration (Weeks 7-10)

#### Enhanced Prompt Engineering
```rust
// src/intelligence/prompts.rs - Advanced prompt generation
pub struct PromptEngine {
    templates: HashMap<String, PromptTemplate>,
    context_weights: ContextWeights,
}

impl PromptEngine {
    pub fn build_enhanced_prompt(
        &self,
        changes: &StagedChanges,
        context: &EnhancedProjectContext,
        semantic_analysis: &SemanticAnalysis,
        preferences: &UserPreferences,
    ) -> Result<String> {
        let mut prompt = PromptBuilder::new();

        // Context-aware prompt selection
        let template = self.select_template(context, semantic_analysis)?;

        // Build structured prompt
        prompt
            .add_system_context(&template.system_prompt)
            .add_project_context(context)
            .add_change_analysis(changes, semantic_analysis)
            .add_historical_context(&context.commit_patterns)
            .add_constraints(preferences)
            .add_output_format(&template.output_format);

        Ok(prompt.build())
    }

    fn select_template(
        &self,
        context: &EnhancedProjectContext,
        semantic_analysis: &SemanticAnalysis,
    ) -> Result<&PromptTemplate> {
        // Smart template selection based on change type
        let change_type = self.classify_change_type(context, semantic_analysis);

        let template_name = match change_type {
            ChangeType::NewFeature => "feature_template",
            ChangeType::BugFix => "bugfix_template",
            ChangeType::Refactoring => "refactor_template",
            ChangeType::Performance => "performance_template",
            ChangeType::Security => "security_template",
            ChangeType::Documentation => "docs_template",
            ChangeType::Testing => "test_template",
            ChangeType::Configuration => "config_template",
            ChangeType::Dependency => "dependency_template",
            _ => "general_template",
        };

        self.templates.get(template_name)
            .ok_or_else(|| anyhow::anyhow!("Template not found: {}", template_name))
    }
}

#[derive(Debug)]
pub struct PromptTemplate {
    pub system_prompt: String,
    pub output_format: OutputFormat,
    pub context_requirements: Vec<ContextRequirement>,
}

impl PromptBuilder {
    pub fn add_change_analysis(
        &mut self,
        changes: &StagedChanges,
        semantic_analysis: &SemanticAnalysis,
    ) -> &mut Self {
        self.sections.push(PromptSection {
            title: "Change Analysis".to_string(),
            content: self.format_change_analysis(changes, semantic_analysis),
            weight: 1.0,
        });

        self
    }

    fn format_change_analysis(
        &self,
        changes: &StagedChanges,
        semantic_analysis: &SemanticAnalysis,
    ) -> String {
        let mut analysis = String::new();

        // High-level change summary
        analysis.push_str(&format!(
            "## Change Summary\n\
             Files modified: {}\n\
             Lines changed: +{} -{}\n\
             Complexity: {}\n\n",
            changes.files_changed.len(),
            changes.insertions,
            changes.deletions,
            self.calculate_complexity_score(semantic_analysis)
        ));

        // Semantic change breakdown
        analysis.push_str("## Semantic Changes\n");
        for pattern in &semantic_analysis.patterns {
            analysis.push_str(&format!("- {}\n", self.format_pattern(pattern)));
        }

        // Impact analysis
        if let Some(impact) = &semantic_analysis.impact {
            analysis.push_str("\n## Impact Analysis\n");

            if !impact.breaking_changes.is_empty() {
                analysis.push_str("⚠️  **Breaking Changes Detected**\n");
                for breaking_change in &impact.breaking_changes {
                    analysis.push_str(&format!("- {}\n", breaking_change.description));
                }
            }

            if !impact.user_facing_changes.is_empty() {
                analysis.push_str("👤 **User-Facing Changes**\n");
                for change in &impact.user_facing_changes {
                    analysis.push_str(&format!("- {}\n", change.description));
                }
            }
        }

        // Focused diff (intelligent truncation)
        analysis.push_str("\n## Key Code Changes\n```diff\n");
        analysis.push_str(&self.create_focused_diff(changes, semantic_analysis));
        analysis.push_str("\n```\n");

        analysis
    }

    fn create_focused_diff(
        &self,
        changes: &StagedChanges,
        semantic_analysis: &SemanticAnalysis,
    ) -> String {
        let mut focused_diff = String::new();
        let max_lines = 100;
        let mut lines_used = 0;

        // Priority order for diff inclusion
        let priorities = [
            DiffPriority::FunctionSignatures,
            DiffPriority::PublicInterfaces,
            DiffPriority::ErrorHandling,
            DiffPriority::BusinessLogic,
            DiffPriority::Configuration,
            DiffPriority::Other,
        ];

        for priority in &priorities {
            if lines_used >= max_lines { break; }

            let relevant_hunks = self.extract_hunks_by_priority(
                &changes.diff_text,
                priority,
                semantic_analysis,
            );

            for hunk in relevant_hunks {
                if lines_used + hunk.line_count > max_lines {
                    break;
                }

                focused_diff.push_str(&hunk.content);
                lines_used += hunk.line_count;
            }
        }

        if lines_used >= max_lines {
            focused_diff.push_str("\n... (additional changes truncated for focus) ...\n");
        }

        focused_diff
    }
}
```

---

## ⚙️ Phase 3: Configuration & Customization (Weeks 8-14)
**Goal**: Make gcm adaptable to any team or project workflow

### 3.1 Flexible Configuration System (Weeks 8-11)

#### Complete Configuration Schema
```rust
// src/config/schema.rs - Comprehensive configuration
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GcmConfig {
    /// Core settings
    #[serde(default)]
    pub core: CoreConfig,

    /// AI Provider configuration
    #[serde(default)]
    pub providers: ProvidersConfig,

    /// Commit message formatting
    #[serde(default)]
    pub commit_style: CommitStyleConfig,

    /// Context gathering settings
    #[serde(default)]
    pub context: ContextConfig,

    /// Template system
    #[serde(default)]
    pub templates: TemplateConfig,

    /// Team and project specific settings
    #[serde(default)]
    pub team: TeamConfig,

    /// Advanced features
    #[serde(default)]
    pub advanced: AdvancedConfig,

    /// User interface preferences
    #[serde(default)]
    pub ui: UIConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoreConfig {
    /// Default number of suggestions
    #[serde(default = "default_suggestion_count")]
    pub suggestion_count: u8,

    /// Maximum response time in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Enable verbose logging
    #[serde(default)]
    pub verbose: bool,

    /// Auto-commit after generation
    #[serde(default)]
    pub auto_commit: bool,

    /// Require confirmation before committing
    #[serde(default = "default_true")]
    pub confirm_commit: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProvidersConfig {
    /// Primary provider
    #[serde(default = "default_provider")]
    pub primary: String,

    /// Fallback provider
    pub fallback: Option<String>,

    /// Provider-specific settings
    #[serde(default)]
    pub openai: OpenAIConfig,

    #[serde(default)]
    pub anthropic: AnthropicConfig,

    #[serde(default)]
    pub local: LocalConfig,

    /// Custom providers
    #[serde(default)]
    pub custom: HashMap<String, CustomProviderConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommitStyleConfig {
    /// Format: conventional, angular, custom
    #[serde(default = "default_format")]
    pub format: String,

    /// Maximum subject line length
    #[serde(default = "default_max_length")]
    pub max_length: usize,

    /// Include scope in commit messages
    #[serde(default = "default_true")]
    pub include_scope: bool,

    /// Use emoji in commit messages
    #[serde(default)]
    pub use_emoji: bool,

    /// Custom type mappings
    #[serde(default)]
    pub type_mappings: HashMap<String, TypeMapping>,

    /// Scope detection rules
    #[serde(default)]
    pub scope_rules: Vec<ScopeRule>,

    /// Breaking change indicators
    #[serde(default)]
    pub breaking_change_indicators: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextConfig {
    /// Files to include in context
    #[serde(default = "default_context_files")]
    pub files: Vec<String>,

    /// Maximum number of recent commits to analyze
    #[serde(default = "default_max_commits")]
    pub max_commits: usize,

    /// Include branch name in context
    #[serde(default = "default_true")]
    pub include_branch: bool,

    /// Analyze test files
    #[serde(default = "default_true")]
    pub analyze_tests: bool,

    /// Include dependency changes
    #[serde(default = "default_true")]
    pub include_dependencies: bool,

    /// Maximum context size in tokens
    #[serde(default = "default_max_context_tokens")]
    pub max_context_tokens: usize,

    /// Context gathering timeout
    #[serde(default = "default_context_timeout")]
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateConfig {
    /// Default template
    #[serde(default = "default_template")]
    pub default: String,

    /// Custom templates
    #[serde(default)]
    pub custom: HashMap<String, CommitTemplate>,

    /// Template selection rules
    #[serde(default)]
    pub selection_rules: Vec<TemplateRule>,

    /// Template inheritance
    #[serde(default)]
    pub inheritance: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamConfig {
    /// Team name
    pub name: Option<String>,

    /// Shared conventions
    #[serde(default)]
    pub conventions: TeamConventions,

    /// Required reviewers
    #[serde(default)]
    pub required_reviewers: Vec<String>,

    /// Issue tracking integration
    #[serde(default)]
    pub issue_tracking: IssueTrackingConfig,

    /// Git hooks
    #[serde(default)]
    pub hooks: HooksConfig,
}

impl GcmConfig {
    pub fn load_from_hierarchy() -> Result<Self> {
        let mut config = Self::default();

        // Load in priority order (later configs override earlier ones)
        let config_sources = [
            ConfigSource::GlobalDefaults,
            ConfigSource::SystemConfig,     // /etc/gcm/config.yml
            ConfigSource::UserConfig,       // ~/.gcm/config.yml or ~/.gcmrc
            ConfigSource::ProjectConfig,    // .gcm.yml
            ConfigSource::LocalConfig,      // .gcm.local.yml (gitignored)
            ConfigSource::Environment,      // Environment variables
            ConfigSource::CommandLine,      // CLI overrides
        ];

        for source in &config_sources {
            if let Ok(source_config) = Self::load_from_source(source) {
                config.merge(source_config)?;
            }
        }

        config.validate()?;
        Ok(config)
    }

    pub fn migrate_from_legacy(&mut self) -> Result<()> {
        // Handle migration from v0.1 config format
        if let Ok(legacy_config) = std::fs::read_to_string(".gcm.yml") {
            if let Ok(legacy) = serde_yaml::from_str::<LegacyConfig>(&legacy_config) {
                self.migrate_legacy_config(legacy)?;

                // Backup old config
                std::fs::rename(".gcm.yml", ".gcm.yml.backup")?;

                // Save new format
                self.save_to_file(".gcm.yml")?;

                println!("✅ Configuration migrated to new format");
                println!("📄 Old config backed up as .gcm.yml.backup");
            }
        }

        Ok(())
    }

    fn validate(&self) -> Result<()> {
        // Validate provider configuration
        if self.providers.primary.is_empty() {
            return Err(anyhow::anyhow!("Primary provider must be specified"));
        }

        // Validate API keys
        self.validate_provider_keys()?;

        // Validate commit style
        if self.commit_style.max_length < 10 || self.commit_style.max_length > 200 {
            return Err(anyhow::anyhow!("max_length must be between 10 and 200"));
        }

        // Validate templates
        self.validate_templates()?;

        Ok(())
    }
}
```

#### Configuration Management CLI
```rust
// src/cli/config.rs - Configuration management commands
impl ConfigCommands {
    pub async fn handle_config_command(&self, action: &ConfigAction) -> Result<()> {
        match action {
            ConfigAction::Show => self.show_config().await,
            ConfigAction::Set { key, value } => self.set_config_value(key, value).await,
            ConfigAction::Reset => self.reset_config().await,
            ConfigAction::Validate => self.validate_config().await,
            ConfigAction::Migrate => self.migrate_config().await,
            ConfigAction::Export { format, output } => self.export_config(format, output).await,
            ConfigAction::Import { file } => self.import_config(file).await,
        }
    }

    async fn show_config(&self) -> Result<()> {
        let config = GcmConfig::load_from_hierarchy()?;

        println!("{}", "📋 Current Configuration".bright_cyan().bold());

        // Show config sources and their status
        self.show_config_sources(&config).await?;

        // Show effective configuration
        self.show_effective_config(&config).await?;

        // Show any validation issues
        if let Err(e) = config.validate() {
            println!("\n{} Configuration Issues:", "⚠️".yellow());
            println!("   {}", e.to_string().red());
        }

        Ok(())
    }

    async fn set_config_value(&self, key: &str, value: &str) -> Result<()> {
        let mut config = GcmConfig::load_from_hierarchy()?;

        // Parse the key path (e.g., "providers.openai.model")
        let key_parts: Vec<&str> = key.split('.').collect();

        match self.set_nested_value(&mut config, &key_parts, value) {
            Ok(_) => {
                config.save_to_user_config()?;
                println!("✅ Configuration updated: {} = {}", key.green(), value.yellow());
            }
            Err(e) => {
                println!("❌ Failed to set configuration: {}", e.to_string().red());
            }
        }

        Ok(())
    }

    fn set_nested_value(
        &self,
        config: &mut GcmConfig,
        key_parts: &[&str],
        value: &str,
    ) -> Result<()> {
        match key_parts {
            ["providers", "primary"] => {
                config.providers.primary = value.to_string();
            }
            ["providers", "openai", "model"] => {
                config.providers.openai.model = value.to_string();
            }
            ["commit_style", "format"] => {
                config.commit_style.format = value.to_string();
            }
            ["commit_style", "max_length"] => {
                config.commit_style.max_length = value.parse()?;
            }
            ["core", "suggestion_count"] => {
                config.core.suggestion_count = value.parse()?;
            }
            _ => {
                return Err(anyhow::anyhow!("Unknown configuration key: {}", key_parts.join(".")));
            }
        }

        config.validate()?;
        Ok(())
    }
}
```

### 3.2 Template System (Weeks 9-12)

#### Advanced Template Engine
```rust
// src/templates/engine.rs - Template engine implementation
use handlebars::{Handlebars, Helper, RenderContext, RenderError};
use serde_json::Value;

pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
    templates: HashMap<String, CommitTemplate>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommitTemplate {
    pub name: String,
    pub description: String,
    pub pattern: String,
    pub rules: Vec<TemplateRule>,
    pub examples: Vec<TemplateExample>,
    pub helpers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateRule {
    pub condition: String,  // JSONPath expression
    pub template: String,   // Handlebars template
    pub priority: u8,
}

impl TemplateEngine {
    pub fn new() -> Result<Self> {
        let mut handlebars = Handlebars::new();

        // Register built-in helpers
        handlebars.register_helper("titlecase", Box::new(titlecase_helper));
        handlebars.register_helper("scope", Box::new(scope_helper));
        handlebars.register_helper("type", Box::new(type_helper));
        handlebars.register_helper("truncate", Box::new(truncate_helper));
        handlebars.register_helper("pluralize", Box::new(pluralize_helper));

        let mut engine = Self {
            handlebars,
            templates: HashMap::new(),
        };

        engine.load_built_in_templates()?;
        Ok(engine)
    }

    pub fn render_commit_message(
        &self,
        template_name: &str,
        context: &TemplateContext,
    ) -> Result<String> {
        let template = self.templates.get(template_name)
            .ok_or_else(|| anyhow::anyhow!("Template not found: {}", template_name))?;

        // Find the best matching rule
        let rule = self.find_matching_rule(template, context)?;

        // Render the template
        let rendered = self.handlebars.render_template(&rule.template, context)?;

        // Post-process the result
        let processed = self.post_process_message(&rendered, context)?;

        Ok(processed)
    }

    fn load_built_in_templates(&mut self) -> Result<()> {
        // Conventional commits template
        let conventional_template = CommitTemplate {
            name: "conventional".to_string(),
            description: "Conventional Commits specification".to_string(),
            pattern: r"^(feat|fix|docs|style|refactor|test|chore)(\(.+\))?: .{1,50}".to_string(),
            rules: vec![
                TemplateRule {
                    condition: "$.change_type == 'feature'".to_string(),
                    template: "feat{{#if scope}}({{scope}}){{/if}}: {{description}}".to_string(),
                    priority: 10,
                },
                TemplateRule {
                    condition: "$.change_type == 'bugfix'".to_string(),
                    template: "fix{{#if scope}}({{scope}}){{/if}}: {{description}}".to_string(),
                    priority: 10,
                },
                TemplateRule {
                    condition: "$.change_type == 'refactor'".to_string(),
                    template: "refactor{{#if scope}}({{scope}}){{/if}}: {{description}}".to_string(),
                    priority: 10,
                },
                TemplateRule {
                    condition: "$.files_changed[*].path =~ '\\.md$'".to_string(),
                    template: "docs: {{description}}".to_string(),
                    priority: 8,
                },
                TemplateRule {
                    condition: "$.files_changed[*].path =~ '_test\\.|spec\\.'".to_string(),
                    template: "test: {{description}}".to_string(),
                    priority: 8,
                },
                TemplateRule {
                    condition: "true".to_string(),  // fallback
                    template: "{{type}}{{#if scope}}({{scope}}){{/if}}: {{description}}".to_string(),
                    priority: 1,
                },
            ],
            examples: vec![
                TemplateExample {
                    description: "New feature".to_string(),
                    output: "feat(auth): add JWT token validation".to_string(),
                },
                TemplateExample {
                    description: "Bug fix".to_string(),
                    output: "fix(api): handle null response in user endpoint".to_string(),
                },
            ],
            helpers: vec!["scope".to_string(), "type".to_string()],
        };

        self.templates.insert("conventional".to_string(), conventional_template);

        // Angular commit template
        let angular_template = CommitTemplate {
            name: "angular".to_string(),
            description: "Angular commit message format".to_string(),
            pattern: r"^(build|ci|docs|feat|fix|perf|refactor|style|test)(\(.+\))?: .{1,50}".to_string(),
            rules: vec![
                TemplateRule {
                    condition: "$.performance_impact.improved == true".to_string(),
                    template: "perf{{#if scope}}({{scope}}){{/if}}: {{description}}".to_string(),
                    priority: 12,
                },
                TemplateRule {
                    condition: "$.files_changed[*].path =~ 'package\\.json|Cargo\\.toml'".to_string(),
                    template: "build{{#if scope}}({{scope}}){{/if}}: {{description}}".to_string(),
                    priority: 10,
                },
                TemplateRule {
                    condition: "$.files_changed[*].path =~ '\\.github/|\\.gitlab-ci|Jenkinsfile'".to_string(),
                    template: "ci{{#if scope}}({{scope}}){{/if}}: {{description}}".to_string(),
                    priority: 10,
                },
            ],
            examples: vec![],
            helpers: vec![],
        };

        self.templates.insert("angular".to_string(), angular_template);

        Ok(())
    }
}

// Template helpers
fn scope_helper(
    h: &Helper,
    _: &Handlebars,
    ctx: &handlebars::Context,
    _: &mut RenderContext,
    out: &mut dyn handlebars::Output,
) -> Result<(), RenderError> {
    // Extract scope from context
    if let Some(context_value) = ctx.data().as_object() {
        let scope = extract_scope_from_context(context_value);
        out.write(&scope)?;
    }
    Ok(())
}

fn type_helper(
    h: &Helper,
    _: &Handlebars,
    ctx: &handlebars::Context,
    _: &mut RenderContext,
    out: &mut dyn handlebars::Output,
) -> Result<(), RenderError> {
    // Infer commit type from context
    if let Some(context_value) = ctx.data().as_object() {
        let commit_type = infer_commit_type(context_value);
        out.write(&commit_type)?;
    }
    Ok(())
}
```

---

## 📦 Phase 4: Distribution & Integration (Weeks 10-16)
**Goal**: Make gcm discoverable and easy to adopt

### 4.1 Multi-Platform Distribution (Weeks 10-13)

#### Release Automation Pipeline
```yaml
# .github/workflows/release.yml - Comprehensive release pipeline
name: Release

on:
  push:
    tags: ['v*']
  workflow_dispatch:
    inputs:
      version:
        description: 'Version to release (e.g., v1.0.0)'
        required: true

jobs:
  build-binaries:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            name: gcm-linux-x86_64
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            name: gcm-linux-aarch64
          - os: macos-latest
            target: x86_64-apple-darwin
            name: gcm-macos-x86_64
          - os: macos-latest
            target: aarch64-apple-darwin
            name: gcm-macos-aarch64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            name: gcm-windows-x86_64.exe

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}

      - name: Build release binary
        run: |
          cargo build --release --target ${{ matrix.target }}

      - name: Create package
        run: |
          mkdir -p dist/
          cp target/${{ matrix.target }}/release/gcm* dist/${{ matrix.name }}

          # Create installation packages
          ./scripts/create-packages.sh ${{ matrix.target }} ${{ matrix.name }}

      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: ${{ matrix.name }}
          path: dist/

  create-packages:
    needs: build-binaries
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Download all artifacts
        uses: actions/download-artifact@v3

      - name: Create distribution packages
        run: |
          # Create Homebrew formula
          ./scripts/create-homebrew-formula.sh

          # Create Debian package
          ./scripts/create-deb-package.sh

          # Create RPM package
          ./scripts/create-rpm-package.sh

          # Create Chocolatey package
          ./scripts/create-chocolatey-package.sh

          # Create Scoop manifest
          ./scripts/create-scoop-manifest.sh

          # Create install script
          ./scripts/create-install-script.sh

  publish-packages:
    needs: [build-binaries, create-packages]
    runs-on: ubuntu-latest

    steps:
      - name: Publish to crates.io
        run: cargo publish --token ${{ secrets.CARGO_TOKEN }}

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            dist/*
            packages/*
          draft: false
          prerelease: false

      - name: Submit to Homebrew
        run: |
          # Create PR to homebrew-core
          ./scripts/submit-to-homebrew.sh

      - name: Publish to AUR
        run: |
          # Update AUR package
          ./scripts/update-aur.sh
```

#### Installation Scripts
```bash
#!/bin/bash
# scripts/install.sh - Universal installation script

set -e

# Default values
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
REPO="gcm-cli/gcm"
PLATFORM=""
ARCH=""

# Colors and formatting
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log() { echo -e "${BLUE}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }

detect_platform() {
    case "$(uname -s)" in
        Linux*)
            PLATFORM="linux"
            case "$(uname -m)" in
                x86_64) ARCH="x86_64" ;;
                aarch64|arm64) ARCH="aarch64" ;;
                *) error "Unsupported architecture: $(uname -m)" ;;
            esac
            ;;
        Darwin*)
            PLATFORM="macos"
            case "$(uname -m)" in
                x86_64) ARCH="x86_64" ;;
                arm64) ARCH="aarch64" ;;
                *) error "Unsupported architecture: $(uname -m)" ;;
            esac
            ;;
        MINGW*|MSYS*|CYGWIN*)
            PLATFORM="windows"
            ARCH="x86_64"
            ;;
        *)
            error "Unsupported platform: $(uname -s)"
            ;;
    esac

    log "Detected platform: ${PLATFORM}-${ARCH}"
}

check_dependencies() {
    log "Checking dependencies..."

    # Check for required tools
    if ! command -v curl >/dev/null 2>&1 && ! command -v wget >/dev/null 2>&1; then
        error "curl or wget is required for installation"
    fi

    if ! command -v tar >/dev/null 2>&1; then
        error "tar is required for installation"
    fi

    # Check for git
    if ! command -v git >/dev/null 2>&1; then
        warn "git not found - gcm requires git to function"
    fi

    success "Dependencies check passed"
}

get_latest_version() {
    log "Fetching latest release information..."

    if command -v curl >/dev/null 2>&1; then
        VERSION=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | cut -d '"' -f 4)
    elif command -v wget >/dev/null 2>&1; then
        VERSION=$(wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | cut -d '"' -f 4)
    fi

    if [ -z "$VERSION" ]; then
        error "Failed to fetch latest version"
    fi

    log "Latest version: $VERSION"
}

download_and_install() {
    local binary_name="gcm-${PLATFORM}-${ARCH}"
    if [ "$PLATFORM" = "windows" ]; then
        binary_name="${binary_name}.exe"
    fi

    local download_url="https://github.com/${REPO}/releases/download/${VERSION}/${binary_name}"
    local temp_file="/tmp/gcm-${PLATFORM}-${ARCH}"

    log "Downloading gcm from $download_url"

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$download_url" -o "$temp_file"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$download_url" -O "$temp_file"
    fi

    if [ ! -f "$temp_file" ]; then
        error "Failed to download gcm binary"
    fi

    # Make executable
    chmod +x "$temp_file"

    # Install binary
    log "Installing gcm to $INSTALL_DIR"

    if [ -w "$INSTALL_DIR" ]; then
        mv "$temp_file" "$INSTALL_DIR/gcm"
    else
        sudo mv "$temp_file" "$INSTALL_DIR/gcm"
    fi

    success "gcm installed successfully!"
}

verify_installation() {
    log "Verifying installation..."

    if command -v gcm >/dev/null 2>&1; then
        local installed_version
        installed_version=$(gcm --version | cut -d ' ' -f 2)
        success "gcm $installed_version is installed and ready to use"
    else
        warn "gcm command not found in PATH"
        warn "You may need to add $INSTALL_DIR to your PATH"
        warn "Add this line to your shell profile:"
        warn "  export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
}

post_install_setup() {
    log "Setting up gcm..."

    # Create config directory
    local config_dir="$HOME/.gcm"
    if [ ! -d "$config_dir" ]; then
        mkdir -p "$config_dir"
        log "Created config directory: $config_dir"
    fi

    # Offer to run setup wizard
    echo
    echo "🚀 Installation complete!"
    echo
    echo "Next steps:"
    echo "1. Set up your API key: export OPENAI_API_KEY=\"your-key-here\""
    echo "2. Run the setup wizard: gcm setup"
    echo "3. Generate your first commit: gcm"
    echo
    echo "For more information, visit: https://github.com/${REPO}"
}

main() {
    echo "🔧 Installing gcm - Git Commit Message Generator"
    echo

    detect_platform
    check_dependencies
    get_latest_version
    download_and_install
    verify_installation
    post_install_setup
}

main "$@"
```

### 4.2 Editor Integration (Weeks 11-14)

#### VS Code Extension
```typescript
// vscode-gcm/src/extension.ts - VS Code extension
import * as vscode from 'vscode';
import { exec } from 'child_process';
import * as path from 'path';

export function activate(context: vscode.ExtensionContext) {
    console.log('gcm extension is now active');

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('gcm.generateCommit', generateCommit),
        vscode.commands.registerCommand('gcm.generateAndCommit', generateAndCommit),
        vscode.commands.registerCommand('gcm.showSuggestions', showSuggestions),
        vscode.commands.registerCommand('gcm.configureSettings', configureSettings)
    );

    // Register status bar item
    const statusBarItem = vscode.window.createStatusBarItem(
        vscode.StatusBarAlignment.Left,
        100
    );
    statusBarItem.text = "$(git-commit) gcm";
    statusBarItem.command = 'gcm.generateCommit';
    statusBarItem.tooltip = 'Generate commit message with gcm';
    statusBarItem.show();
    context.subscriptions.push(statusBarItem);

    // Register source control integration
    if (vscode.extensions.getExtension('vscode.git')) {
        registerSourceControlIntegration(context);
    }
}

async function generateCommit() {
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    if (!workspaceFolder) {
        vscode.window.showErrorMessage('No workspace folder open');
        return;
    }

    // Show progress
    await vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: "Generating commit message...",
        cancellable: true
    }, async (progress, token) => {
        try {
            const result = await executeGcm(workspaceFolder.uri.fsPath);

            if (token.isCancellationRequested) {
                return;
            }

            await showCommitMessage(result.message, result.reasoning);
        } catch (error) {
            vscode.window.showErrorMessage(`gcm error: ${error}`);
        }
    });
}

async function executeGcm(cwd: string, args: string[] = []): Promise<GcmResult> {
    return new Promise((resolve, reject) => {
        const command = `gcm ${args.join(' ')} --explain --format json`;

        exec(command, { cwd }, (error, stdout, stderr) => {
            if (error) {
                reject(error);
                return;
            }

            try {
                const result = JSON.parse(stdout);
                resolve(result);
            } catch (parseError) {
                // Fallback for non-JSON output
                resolve({
                    message: stdout.trim(),
                    reasoning: null,
                    confidence: 0.8
                });
            }
        });
    });
}

async function showCommitMessage(message: string, reasoning?: string) {
    const panel = vscode.window.createWebviewPanel(
        'gcmCommitMessage',
        'Commit Message',
        vscode.ViewColumn.Beside,
        {
            enableScripts: true,
            localResourceRoots: []
        }
    );

    panel.webview.html = getCommitMessageWebview(message, reasoning);

    // Handle messages from webview
    panel.webview.onDidReceiveMessage(async (msg) => {
        switch (msg.command) {
            case 'commit':
                await commitWithMessage(msg.message);
                panel.dispose();
                break;
            case 'copy':
                await vscode.env.clipboard.writeText(msg.message);
                vscode.window.showInformationMessage('Commit message copied to clipboard');
                break;
            case 'edit':
                await openCommitMessageEditor(msg.message);
                panel.dispose();
                break;
        }
    });
}

function getCommitMessageWebview(message: string, reasoning?: string): string {
    return `
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Commit Message</title>
        <style>
            body {
                font-family: var(--vscode-font-family);
                padding: 20px;
                color: var(--vscode-foreground);
                background-color: var(--vscode-editor-background);
            }
            .commit-message {
                background: var(--vscode-editor-background);
                border: 1px solid var(--vscode-panel-border);
                border-radius: 4px;
                padding: 15px;
                margin: 10px 0;
                font-family: var(--vscode-editor-font-family);
                font-size: var(--vscode-editor-font-size);
            }
            .reasoning {
                margin-top: 15px;
                padding: 10px;
                background: var(--vscode-textBlockQuote-background);
                border-left: 4px solid var(--vscode-textBlockQuote-border);
                font-style: italic;
            }
            .actions {
                margin-top: 20px;
                display: flex;
                gap: 10px;
            }
            button {
                background: var(--vscode-button-background);
                color: var(--vscode-button-foreground);
                border: none;
                padding: 8px 16px;
                border-radius: 4px;
                cursor: pointer;
            }
            button:hover {
                background: var(--vscode-button-hoverBackground);
            }
            .primary {
                background: var(--vscode-button-background);
            }
            .secondary {
                background: var(--vscode-button-secondaryBackground);
                color: var(--vscode-button-secondaryForeground);
            }
        </style>
    </head>
    <body>
        <h2>🎯 Generated Commit Message</h2>

        <div class="commit-message" id="commitMessage">${escapeHtml(message)}</div>

        ${reasoning ? `
        <div class="reasoning">
            <strong>💡 Reasoning:</strong><br>
            ${escapeHtml(reasoning)}
        </div>
        ` : ''}

        <div class="actions">
            <button class="primary" onclick="commitMessage()">
                ✅ Commit
            </button>
            <button class="secondary" onclick="editMessage()">
                ✏️ Edit
            </button>
            <button class="secondary" onclick="copyMessage()">
                📋 Copy
            </button>
        </div>

        <script>
            const vscode = acquireVsCodeApi();

            function commitMessage() {
                vscode.postMessage({
                    command: 'commit',
                    message: document.getElementById('commitMessage').textContent
                });
            }

            function editMessage() {
                vscode.postMessage({
                    command: 'edit',
                    message: document.getElementById('commitMessage').textContent
                });
            }

            function copyMessage() {
                vscode.postMessage({
                    command: 'copy',
                    message: document.getElementById('commitMessage').textContent
                });
            }
        </script>
    </body>
    </html>
    `;
}

interface GcmResult {
    message: string;
    reasoning?: string;
    confidence: number;
}

function escapeHtml(text: string): string {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}
```

---

## 🚀 Phase 5: Advanced Intelligence (Weeks 12-20)
**Goal**: Push the boundaries of AI-assisted development workflows

### 5.1 Advanced AI Features (Weeks 12-16)

#### Change Impact Analysis
```rust
// src/intelligence/impact.rs - Advanced impact analysis
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeImpactAnalyzer {
    language_analyzers: HashMap<String, Box<dyn LanguageAnalyzer>>,
    dependency_graph: DependencyGraph,
    api_registry: ApiRegistry,
}

impl ChangeImpactAnalyzer {
    pub async fn analyze_impact(
        &self,
        changes: &StagedChanges,
        context: &EnhancedProjectContext,
    ) -> Result<ImpactAnalysis> {
        let mut analysis = ImpactAnalysis::default();

        // Parallel analysis of different impact types
        let (
            breaking_changes,
            api_changes,
            performance_impact,
            security_impact,
            dependency_impact,
            user_impact,
        ) = tokio::join!(
            self.analyze_breaking_changes(changes, context),
            self.analyze_api_changes(changes, context),
            self.analyze_performance_impact(changes, context),
            self.analyze_security_impact(changes, context),
            self.analyze_dependency_impact(changes, context),
            self.analyze_user_impact(changes, context),
        );

        analysis.breaking_changes = breaking_changes?;
        analysis.api_changes = api_changes?;
        analysis.performance_impact = performance_impact?;
        analysis.security_implications = security_impact?;
        analysis.dependency_changes = dependency_impact?;
        analysis.user_facing_changes = user_impact?;

        // Calculate overall impact score
        analysis.impact_score = self.calculate_impact_score(&analysis);

        Ok(analysis)
    }

    async fn analyze_breaking_changes(
        &self,
        changes: &StagedChanges,
        context: &EnhancedProjectContext,
    ) -> Result<Vec<BreakingChange>> {
        let mut breaking_changes = Vec::new();

        for file_change in &changes.files_changed {
            if let Some(analyzer) = self.get_language_analyzer(&file_change.path) {
                let file_breaking_changes = analyzer
                    .detect_breaking_changes(file_change, context)
                    .await?;
                breaking_changes.extend(file_breaking_changes);
            }
        }

        // Cross-file breaking change detection
        breaking_changes.extend(self.detect_cross_file_breaking_changes(changes).await?);

        Ok(breaking_changes)
    }

    async fn analyze_api_changes(
        &self,
        changes: &StagedChanges,
        context: &EnhancedProjectContext,
    ) -> Result<Vec<ApiChange>> {
        let mut api_changes = Vec::new();

        // Detect public API changes
        for file_change in &changes.files_changed {
            if self.is_api_file(&file_change.path, context) {
                let file_api_changes = self
                    .extract_api_changes(file_change, context)
                    .await?;
                api_changes.extend(file_api_changes);
            }
        }

        // Schema and interface changes
        api_changes.extend(self.detect_schema_changes(changes).await?);

        Ok(api_changes)
    }

    async fn analyze_performance_impact(
        &self,
        changes: &StagedChanges,
        context: &EnhancedProjectContext,
    ) -> Result<PerformanceImpact> {
        let mut impact = PerformanceImpact::default();

        // Algorithm complexity analysis
        impact.complexity_changes = self.analyze_complexity_changes(changes).await?;

        // Resource usage analysis
        impact.resource_changes = self.analyze_resource_usage(changes).await?;

        // Database query impact
        impact.database_impact = self.analyze_database_impact(changes).await?;

        // Network request changes
        impact.network_impact = self.analyze_network_impact(changes).await?;

        // Overall performance score
        impact.score = self.calculate_performance_score(&impact);

        Ok(impact)
    }

    async fn analyze_security_impact(
        &self,
        changes: &StagedChanges,
        context: &EnhancedProjectContext,
    ) -> Result<Vec<SecurityImplication>> {
        let mut implications = Vec::new();

        // Static security analysis
        for file_change in &changes.files_changed {
            implications.extend(self.scan_for_security_issues(file_change).await?);
        }

        // Dependency vulnerability analysis
        implications.extend(self.analyze_dependency_vulnerabilities(changes).await?);

        // Authentication and authorization changes
        implications.extend(self.analyze_auth_changes(changes).await?);

        // Input validation changes
        implications.extend(self.analyze_input_validation(changes).await?);

        Ok(implications)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    pub breaking_changes: Vec<BreakingChange>,
    pub api_changes: Vec<ApiChange>,
    pub performance_impact: PerformanceImpact,
    pub security_implications: Vec<SecurityImplication>,
    pub dependency_changes: Vec<DependencyChange>,
    pub user_facing_changes: Vec<UserFacingChange>,
    pub impact_score: f32,
    pub risk_level: RiskLevel,
    pub recommendations: Vec<Recommendation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BreakingChange {
    pub change_type: BreakingChangeType,
    pub description: String,
    pub affected_apis: Vec<String>,
    pub mitigation_strategies: Vec<String>,
    pub severity: Severity,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BreakingChangeType {
    RemovedFunction { name: String },
    ChangedSignature { function: String, old_sig: String, new_sig: String },
    RemovedField { struct_name: String, field: String },
    ChangedReturnType { function: String, old_type: String, new_type: String },
    RemovedEndpoint { path: String, method: String },
    ChangedResponseFormat { endpoint: String },
    RemovedConfiguration { key: String },
}

impl BreakingChangeType {
    pub fn to_commit_message_hint(&self) -> String {
        match self {
            Self::RemovedFunction { name } => format!("BREAKING CHANGE: remove {} function", name),
            Self::ChangedSignature { function, .. } => format!("BREAKING CHANGE: change {} signature", function),
            Self::RemovedField { struct_name, field } => format!("BREAKING CHANGE: remove {}.{} field", struct_name, field),
            Self::ChangedReturnType { function, .. } => format!("BREAKING CHANGE: change {} return type", function),
            Self::RemovedEndpoint { path, method } => format!("BREAKING CHANGE: remove {} {} endpoint", method, path),
            Self::ChangedResponseFormat { endpoint } => format!("BREAKING CHANGE: change {} response format", endpoint),
            Self::RemovedConfiguration { key } => format!("BREAKING CHANGE: remove {} config", key),
        }
    }
}
```

### 5.2 Automated Versioning (Weeks 14-17)

#### Semantic Version Analysis
```rust
// src/intelligence/versioning.rs - Automatic version suggestion
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct VersionAnalyzer {
    current_version: Option<Version>,
    version_strategy: VersionStrategy,
    change_rules: Vec<VersionRule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionSuggestion {
    pub current: Option<Version>,
    pub suggested: Version,
    pub reasoning: String,
    pub confidence: f32,
    pub change_summary: ChangeSummary,
    pub breaking_changes: Vec<BreakingChange>,
}

impl VersionAnalyzer {
    pub async fn suggest_version(
        &self,
        impact_analysis: &ImpactAnalysis,
        context: &EnhancedProjectContext,
    ) -> Result<VersionSuggestion> {
        let current = self.get_current_version(context)?;

        // Analyze change types
        let change_summary = self.summarize_changes(impact_analysis);

        // Apply versioning rules
        let version_bump = self.determine_version_bump(&change_summary, impact_analysis);

        // Calculate new version
        let suggested = match current {
            Some(ref current_ver) => self.apply_version_bump(current_ver, version_bump)?,
            None => Version::new(0, 1, 0), // Initial version
        };

        // Generate reasoning
        let reasoning = self.generate_version_reasoning(&change_summary, &version_bump);

        // Calculate confidence
        let confidence = self.calculate_confidence(&change_summary, impact_analysis);

        Ok(VersionSuggestion {
            current,
            suggested,
            reasoning,
            confidence,
            change_summary,
            breaking_changes: impact_analysis.breaking_changes.clone(),
        })
    }

    fn determine_version_bump(
        &self,
        change_summary: &ChangeSummary,
        impact_analysis: &ImpactAnalysis,
    ) -> VersionBump {
        // Breaking changes = major version bump
        if !impact_analysis.breaking_changes.is_empty() {
            return VersionBump::Major;
        }

        // New features = minor version bump
        if change_summary.new_features > 0 || change_summary.new_apis > 0 {
            return VersionBump::Minor;
        }

        // Bug fixes, performance improvements, etc. = patch version bump
        if change_summary.bug_fixes > 0
            || change_summary.performance_improvements > 0
            || change_summary.security_fixes > 0 {
            return VersionBump::Patch;
        }

        // Documentation, tests, refactoring = patch version bump
        if change_summary.documentation_changes > 0
            || change_summary.test_changes > 0
            || change_summary.refactoring_changes > 0 {
            return VersionBump::Patch;
        }

        // Default to patch for any changes
        VersionBump::Patch
    }

    fn generate_version_reasoning(
        &self,
        change_summary: &ChangeSummary,
        version_bump: &VersionBump,
    ) -> String {
        let mut reasoning = Vec::new();

        match version_bump {
            VersionBump::Major => {
                reasoning.push("Major version bump due to breaking changes".to_string());
            }
            VersionBump::Minor => {
                if change_summary.new_features > 0 {
                    reasoning.push(format!("Minor version bump for {} new feature(s)", change_summary.new_features));
                }
                if change_summary.new_apis > 0 {
                    reasoning.push(format!("New APIs added: {}", change_summary.new_apis));
                }
            }
            VersionBump::Patch => {
                if change_summary.bug_fixes > 0 {
                    reasoning.push(format!("{} bug fix(es)", change_summary.bug_fixes));
                }
                if change_summary.performance_improvements > 0 {
                    reasoning.push(format!("{} performance improvement(s)", change_summary.performance_improvements));
                }
                if change_summary.security_fixes > 0 {
                    reasoning.push(format!("{} security fix(es)", change_summary.security_fixes));
                }
            }
        }

        reasoning.join(", ")
    }
}

#[derive(Debug)]
pub enum VersionBump {
    Major,
    Minor,
    Patch,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeSummary {
    pub new_features: usize,
    pub bug_fixes: usize,
    pub performance_improvements: usize,
    pub security_fixes: usize,
    pub breaking_changes: usize,
    pub new_apis: usize,
    pub deprecated_apis: usize,
    pub documentation_changes: usize,
    pub test_changes: usize,
    pub refactoring_changes: usize,
    pub dependency_updates: usize,
}
```

---

## 📊 Implementation Timeline & Milestones

### Phase 1: User Experience Excellence (Weeks 1-8)
**Milestone 1.1 (Week 2)**: Enhanced CLI with subcommands ✅
**Milestone 1.2 (Week 4)**: Rich error handling and recovery ✅
**Milestone 1.3 (Week 6)**: Interactive terminal UI ✅
**Milestone 1.4 (Week 8)**: First-time user setup wizard ✅

### Phase 2: Intelligent Commit Generation (Weeks 5-12)
**Milestone 2.1 (Week 8)**: Enhanced context analysis ✅
**Milestone 2.2 (Week 10)**: Semantic code understanding ✅
**Milestone 2.3 (Week 12)**: Advanced LLM integration ✅

### Phase 3: Configuration & Customization (Weeks 8-14)
**Milestone 3.1 (Week 11)**: Flexible configuration system ✅
**Milestone 3.2 (Week 13)**: Template engine and customization ✅
**Milestone 3.3 (Week 14)**: Team collaboration features ✅

### Phase 4: Distribution & Integration (Weeks 10-16)
**Milestone 4.1 (Week 13)**: Multi-platform distribution ✅
**Milestone 4.2 (Week 15)**: Editor integrations ✅
**Milestone 4.3 (Week 16)**: Package manager submissions ✅

### Phase 5: Advanced Intelligence (Weeks 12-20)
**Milestone 5.1 (Week 16)**: Change impact analysis ✅
**Milestone 5.2 (Week 18)**: Automated versioning ✅
**Milestone 5.3 (Week 20)**: Advanced workflow features ✅

---

## 🧪 Testing & Quality Assurance Strategy

### Testing Framework
```rust
// tests/integration/mod.rs - Comprehensive testing strategy
use gcm::{Cli, Config, GitAnalyzer, LLMClient};
use tempfile::TempDir;
use std::process::Command;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_workflow() {
        let temp_repo = create_test_repository().await;

        // Test setup wizard
        let config = run_setup_wizard(&temp_repo).await?;
        assert!(config.validate().is_ok());

        // Test commit generation
        let result = generate_test_commit(&temp_repo, &config).await?;
        assert!(result.message.len() > 10);
        assert!(result.message.len() <= 72);

        // Test commit execution
        let commit_result = execute_commit(&temp_repo, &result.message).await?;
        assert!(commit_result.success);
    }

    #[tokio::test]
    async fn test_error_handling() {
        // Test various error scenarios
        test_no_staged_changes().await?;
        test_invalid_api_key().await?;
        test_network_timeout().await?;
        test_malformed_config().await?;
    }

    #[tokio::test]
    async fn test_performance() {
        let temp_repo = create_large_test_repository().await;

        let start = std::time::Instant::now();
        let result = generate_commit_with_large_diff(&temp_repo).await?;
        let duration = start.elapsed();

        // Should complete within 2 seconds
        assert!(duration.as_secs() < 2);
        assert!(!result.message.is_empty());
    }
}

// Performance benchmarks
#[cfg(test)]
mod benchmarks {
    use criterion::{criterion_group, criterion_main, Criterion};

    fn benchmark_context_gathering(c: &mut Criterion) {
        c.bench_function("context_gathering", |b| {
            b.iter(|| {
                let analyzer = GitAnalyzer::new().unwrap();
                analyzer.get_project_context()
            });
        });
    }

    fn benchmark_semantic_analysis(c: &mut Criterion) {
        c.bench_function("semantic_analysis", |b| {
            b.iter(|| {
                let analyzer = SemanticAnalyzer::new().unwrap();
                analyzer.analyze_changes(&test_changes())
            });
        });
    }

    criterion_group!(benches, benchmark_context_gathering, benchmark_semantic_analysis);
    criterion_main!(benches);
}
```

### Cross-Platform Testing
```yaml
# .github/workflows/test.yml - Comprehensive testing pipeline
name: Tests

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}

      - name: Run unit tests
        run: cargo test --lib

      - name: Run integration tests
        run: cargo test --test integration

      - name: Run performance benchmarks
        run: cargo bench

      - name: Test CLI interface
        run: |
          cargo build --release
          ./target/release/gcm --help
          ./target/release/gcm --version
```

---

## 📈 Success Metrics & Monitoring

### Key Performance Indicators
```rust
// src/telemetry/metrics.rs - Usage analytics and performance monitoring
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageMetrics {
    pub session_id: String,
    pub command: String,
    pub duration_ms: u64,
    pub success: bool,
    pub error_type: Option<String>,
    pub user_accepted: Option<bool>,
    pub suggestion_count: u8,
    pub context_size: usize,
    pub diff_size: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl UsageMetrics {
    pub async fn track_command_usage(
        command: &str,
        result: &Result<CommitSuggestion>,
        duration: std::time::Duration,
    ) {
        let metrics = UsageMetrics {
            session_id: get_session_id(),
            command: command.to_string(),
            duration_ms: duration.as_millis() as u64,
            success: result.is_ok(),
            error_type: result.as_ref().err().map(|e| e.to_string()),
            user_accepted: None, // Will be updated later
            suggestion_count: result.as_ref().map(|r| r.suggestions.len() as u8).unwrap_or(0),
            context_size: get_context_size(),
            diff_size: get_diff_size(),
            timestamp: chrono::Utc::now(),
        };

        // Send metrics (with user consent)
        if should_send_telemetry() {
            send_metrics_async(metrics).await;
        }
    }
}

// Target metrics for success
pub const TARGET_METRICS: &[(&str, f64)] = &[
    ("response_time_p95", 2000.0),           // 95th percentile < 2s
    ("user_acceptance_rate", 0.90),          // 90% acceptance rate
    ("error_rate", 0.05),                    // < 5% error rate
    ("weekly_active_users", 10000.0),        // 10k+ weekly users
    ("user_retention_30d", 0.80),            // 80% 30-day retention
];
```

This detailed implementation plan provides a comprehensive roadmap for transforming gcm into a best-in-class developer tool. Each phase includes specific implementation details, code examples, timelines, and success criteria to ensure successful execution.