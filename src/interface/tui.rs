use std::collections::VecDeque;
use std::io;
use std::io::{Stdout, Write};

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tokio::sync::mpsc;
use tracing::{info, Level};
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::MakeWriter;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{List, ListItem, Tabs};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    widgets::{Block, Borders},
    Frame, Terminal,
};

#[derive(Debug)]
pub enum TuiCommand {
    Log(String),
    Close,
    Refresh,
}

#[derive(Clone)]
struct LogWriter {
    pub log_writer: mpsc::UnboundedSender<TuiCommand>,
}

impl Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let _ = self.log_writer.send(TuiCommand::Log(
            std::str::from_utf8(buf).unwrap().to_string(),
        ));
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl MakeWriter<'_> for LogWriter {
    type Writer = Self;

    fn make_writer(&self) -> Self::Writer {
        self.clone()
    }
}

struct TUIState {
    log_lines: VecDeque<String>,
    current_tab: usize,
}

pub fn start_tui(
    write: mpsc::UnboundedSender<TuiCommand>,
    read: mpsc::UnboundedReceiver<TuiCommand>,
) -> Result<()> {
    let mut term = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let format = fmt::format()
        .with_ansi(false)
        .without_time()
        .with_level(true)
        .with_target(false)
        .with_thread_names(false)
        .with_source_location(false)
        .compact();

    tracing_subscriber::fmt()
        .with_writer(LogWriter { log_writer: write })
        .with_max_level(Level::INFO)
        .event_format(format)
        .init();

    info!("tui selected and loaded");

    tokio::spawn(async move {
        enable_raw_mode().unwrap();
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture).unwrap();

        let mut cmds = read;
        let mut state = TUIState {
            log_lines: VecDeque::new(),
            current_tab: 0,
        };

        fn close(mut term: Terminal<CrosstermBackend<Stdout>>) {
            disable_raw_mode().unwrap();
            execute!(
                term.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )
            .unwrap();

            term.show_cursor().unwrap();
            std::process::exit(0);
        }

        term.clear().unwrap();

        while let Some(cmd) = cmds.recv().await {
            if let TuiCommand::Close = cmd {
                return close(term);
            }

            term.draw(|f| {
                match cmd {
                    TuiCommand::Log(msg) => {
                        state.log_lines.push_back(msg);
                    }
                    TuiCommand::Refresh => {
                        //no-op, this just causes a re-render
                    }
                    c => unimplemented!("unimplemented cmd {:?}", c),
                };

                // render the UI:
                do_render(f, &mut state);
            })
            .unwrap();
        }
    });

    Ok(())
}

fn do_render<B>(f: &mut Frame<B>, state: &mut TUIState)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(f.size());

    let titles = ["Logs", "Classes", "Heap", "GC"]
        .iter()
        .map(|t| Spans::from(Span::styled(*t, Style::default().fg(Color::Gray))))
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(state.current_tab);

    f.render_widget(tabs, chunks[0]);
    render_log(f, state, chunks[1]);
}

fn render_log<B>(f: &mut Frame<B>, state: &mut TUIState, area: Rect)
where
    B: Backend,
{
    let info_style = Style::default().fg(Color::Blue);
    let warning_style = Style::default().fg(Color::Yellow);
    let error_style = Style::default().fg(Color::Red);

    let logs: Vec<ListItem> = state
        .log_lines
        .iter()
        .map(|msg| {
            let msg = msg.strip_prefix(' ').unwrap_or(msg);
            let (level, style) = if msg.starts_with("INFO") {
                ("INFO", info_style)
            } else if msg.starts_with("WARN") {
                ("WARN", warning_style)
            } else if msg.starts_with("ERROR") {
                ("ERROR", error_style)
            } else {
                ("???", Style::default().fg(Color::White))
            };

            let content = vec![Spans::from(vec![
                Span::styled(format!("{:<9}", level), style),
                Span::raw(msg[level.len()..msg.len()].to_owned()),
            ])];
            ListItem::new(content)
        })
        .collect();

    let logs = List::new(logs).block(Block::default().borders(Borders::ALL));

    f.render_widget(logs, area);
}
