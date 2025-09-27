use anyhow::Result;
use clap::Parser;
use colored::Colorize;

mod cli;
mod config;
mod git;
mod llm;

use cli::Cli;
use config::Config;
use git::GitAnalyzer;
use llm::LLMClient;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    let mut config = Config::load()?;

    if let Some(model) = args.model {
        config.model = model;
    }

    let git_analyzer = GitAnalyzer::new()?;

    if args.all {
        println!("{}", "Staging all changes...".yellow());
        git_analyzer.stage_all_changes()?;
    }

    if !git_analyzer.has_staged_changes()? {
        eprintln!("{}", "No staged changes found. Use 'git add' to stage changes or use -a flag.".red());
        std::process::exit(1);
    }

    let staged_changes = git_analyzer.get_staged_diff()?;
    let context = git_analyzer.get_project_context()?;

    if config.api_key.is_empty() {
        eprintln!("{}", "Error: API key not found. Please set one of the following:".red());
        eprintln!("  - OPENAI_API_KEY in .env file or environment variable");
        eprintln!("  - ANTHROPIC_API_KEY in .env file or environment variable");
        eprintln!("  - Configure in ~/.gcmrc or .gcm.yml");
        std::process::exit(1);
    }

    let llm_client = LLMClient::new(config.model.clone(), config.api_key.clone());

    println!("{}", "Generating commit message...".yellow());

    let messages = match llm_client.generate_commit_message(
        &staged_changes,
        Some(&context),
        args.number as usize,
    ).await {
        Ok(messages) => messages,
        Err(e) => {
            eprintln!("{} {}", "Error generating commit message:".red(), e);
            std::process::exit(1);
        }
    };

    if args.number == 1 {
        let message = &messages[0];
        println!("\n{}", "Generated commit message:".green());
        println!("  {}", message);

        if args.commit {
            match git_analyzer.commit_with_message(message) {
                Ok(_) => println!("\n{} {}", "Committed with message:".green(), message),
                Err(e) => {
                    eprintln!("{} {}", "Error committing:".red(), e);
                    std::process::exit(1);
                }
            }
        }
    } else {
        println!("\n{} {} commit message suggestions:", "Generated".green(), messages.len());
        for (i, message) in messages.iter().enumerate() {
            println!("  {}. {}", i + 1, message);
        }
    }

    Ok(())
}