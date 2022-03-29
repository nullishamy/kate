use anyhow::Result;
use std::io;
use std::io::Stdout;
use tui::backend::CrosstermBackend;
use tui::Terminal;

pub struct TUI {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TUI {
    pub fn new() -> Result<Self> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self { terminal })
    }
}

pub struct TUIWriter {}
