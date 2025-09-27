use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short = 'c', long, help = "Generate message and commit")]
    pub commit: bool,

    #[arg(short = 'a', long, help = "Stage all changes first")]
    pub all: bool,

    #[arg(short = 'n', long, default_value = "1", help = "Number of message suggestions")]
    pub number: u8,

    #[arg(long, help = "Amend the previous commit")]
    pub amend: bool,

    #[arg(long, help = "Override the LLM model to use")]
    pub model: Option<String>,
}