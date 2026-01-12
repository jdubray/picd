mod app;
mod tui;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "pictd-md")]
#[command(about = "Fill markdown image placeholders from clipboard", long_about = None)]
struct Args {
    /// Path to the markdown file
    #[arg(value_name = "MARKDOWN_FILE")]
    markdown: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Verify markdown file exists
    if !args.markdown.exists() {
        anyhow::bail!("Markdown file not found: {}", args.markdown.display());
    }

    // Run the TUI application
    tui::run(&args.markdown)
}
