use anyhow::Result;
use clap::Parser;
use crossterm::{
    execute,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::path::PathBuf;
use tui::app::App;
use tui::run_app;

mod core;
mod forensic;
mod tui;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target directory or disk to scan
    #[arg(short, long)]
    target: Option<PathBuf>,

    /// Search pattern (regex)
    #[arg(short, long)]
    pattern: Option<String>,

    /// Output directory for recovered files
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Enable deep scan (slower but more thorough)
    #[arg(short, long)]
    deep: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    // Create app
    let mut app = App::new(args.target.clone(), args.pattern.clone(), args.deep);
    
    // Run app
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

