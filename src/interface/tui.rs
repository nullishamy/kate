use std::collections::VecDeque;
use std::io;
use std::io::{Stdout, Write};

use crate::runtime::vm::VMState;
use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{info, Level};
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::MakeWriter;
use tui::backend::Backend;
use tui::layout::{Alignment, Direction, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, LineGauge, List, ListItem, Paragraph, Tabs};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    widgets::Borders,
    Frame, Terminal,
};

#[derive(Debug)]
pub struct UpdateHeap {
    pub new_size: usize,
    pub total: usize,
}

#[derive(Debug)]
pub struct UpdateCPU {
    pub new_usage: usize,
    pub total: usize,
}

#[derive(Debug)]
pub enum TuiCommand {
    Log(String),
    Close,
    Refresh,
    VMState(VMState),
    Tab(usize),
    Heap(UpdateHeap),
    CPU(UpdateCPU),
}

#[derive(Clone)]
struct LogWriter {
    pub log_writer: mpsc::UnboundedSender<TuiCommand>,
}

#[derive(Clone)]
pub struct TUIWriter {
    write: UnboundedSender<TuiCommand>,
}

impl TUIWriter {
    pub fn send(&self, cmd: TuiCommand) -> Result<()> {
        Ok(self.write.send(cmd)?)
    }
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
    vm_state: VMState,
    cpu_used: usize,
    cpu_total: usize,
    heap_used: usize,
    heap_total: usize,
}

pub fn start_tui(
    write: mpsc::UnboundedSender<TuiCommand>,
    read: mpsc::UnboundedReceiver<TuiCommand>,
) -> Result<TUIWriter> {
    let format = fmt::format()
        .with_ansi(false)
        .without_time()
        .with_level(true)
        .with_target(false)
        .with_thread_names(false)
        .with_source_location(false)
        .compact();

    tracing_subscriber::fmt()
        .with_writer(LogWriter {
            log_writer: write.clone(),
        })
        .with_max_level(Level::INFO)
        .event_format(format)
        .init();

    info!("tui selected and loaded");

    tokio::spawn(async {
        let mut term = Terminal::new(CrosstermBackend::new(io::stdout())).unwrap();

        enable_raw_mode().unwrap();
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture).unwrap();

        let mut cmds = read;

        let mut state = TUIState {
            log_lines: VecDeque::new(),
            current_tab: 0,
            vm_state: VMState::Shutdown,
            cpu_used: 0,
            cpu_total: 1,
            heap_used: 0,
            heap_total: 1,
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
                // side effect if needed
                match cmd {
                    TuiCommand::Log(msg) => {
                        state.log_lines.push_back(msg);
                    }
                    TuiCommand::Refresh => {
                        //no-op, this just causes a re-render
                    }
                    TuiCommand::VMState(new_status) => {
                        state.vm_state = new_status;
                    }
                    TuiCommand::Tab(new_tab) => {
                        state.current_tab = new_tab;
                    }
                    TuiCommand::CPU(data) => {
                        state.cpu_total = data.total;
                        state.cpu_used = data.new_usage;
                    }
                    TuiCommand::Heap(data) => {
                        state.heap_total = data.total;
                        state.heap_used = data.new_size;
                    }
                    c => unimplemented!("unimplemented cmd {:?}", c),
                };

                // render the UI
                do_render(f, &mut state);
            })
            .unwrap();
        }
    });

    Ok(TUIWriter { write })
}

fn do_render<B>(f: &mut Frame<B>, state: &mut TUIState)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(f.size());

    let tab_bar = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(chunks[0]);

    // render the top bar
    render_tabs(f, state, tab_bar[0]);
    render_status(f, state, tab_bar[1]);

    // render the main body
    match state.current_tab {
        0 => render_log(f, state, chunks[1]),
        1 => render_classes(f, state, chunks[1]),
        2 => render_heap(f, state, chunks[1]),
        3 => render_gc(f, state, chunks[1]),
        4 => render_resources(f, state, chunks[1]),
        _ => unreachable!(), // there should never be more than 4 tabs
                             // this would be a programming error
    }
}

fn render_resources<B>(f: &mut Frame<B>, state: &mut TUIState, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0), // min 0 takes up the rest of the space
        ])
        .margin(1)
        .split(area);

    let block = Block::default().borders(Borders::ALL);
    f.render_widget(block, area);

    let heap_perc = state.heap_used as f64 / state.heap_total as f64;
    let heap_gauge = LineGauge::default()
        .block(Block::default().title("Heap").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Magenta))
        .ratio(heap_perc);

    let cpu_perc = state.cpu_used as f64 / state.cpu_total as f64;
    let cpu_gauge = LineGauge::default()
        .block(Block::default().title("CPU").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Magenta))
        .ratio(cpu_perc);

    f.render_widget(heap_gauge, chunks[0]);
    f.render_widget(cpu_gauge, chunks[1]);
}

fn render_gc<B>(f: &mut Frame<B>, _state: &mut TUIState, area: Rect)
where
    B: Backend,
{
    let block = Block::default().borders(Borders::ALL);
    let para = Paragraph::new("Unimplemented").block(block);
    f.render_widget(para, area);
}

fn render_heap<B>(f: &mut Frame<B>, _state: &mut TUIState, area: Rect)
where
    B: Backend,
{
    let block = Block::default().borders(Borders::ALL);
    let para = Paragraph::new("Unimplemented").block(block);
    f.render_widget(para, area);
}

fn render_classes<B>(f: &mut Frame<B>, _state: &mut TUIState, area: Rect)
where
    B: Backend,
{
    let block = Block::default().borders(Borders::ALL);
    let para = Paragraph::new("Unimplemented").block(block);
    f.render_widget(para, area);
}

fn render_status<B>(f: &mut Frame<B>, state: &mut TUIState, area: Rect)
where
    B: Backend,
{
    let block = Block::default().borders(Borders::ALL).title("Status");
    let paused = Style::default().fg(Color::LightYellow);
    let booting = Style::default().fg(Color::Yellow);
    let shutdown = Style::default().fg(Color::Red);
    let gc = Style::default().fg(Color::Magenta);
    let online = Style::default().fg(Color::Green);

    let style = match state.vm_state {
        VMState::Shutdown => shutdown,
        VMState::Booting => booting,
        VMState::Online => online,
        VMState::Paused => paused,
        VMState::ShuttingDown => booting,
        VMState::GC => gc,
    };
    let status = Paragraph::new(format!("{:?}", state.vm_state))
        .block(block)
        .style(style)
        .alignment(Alignment::Center);

    f.render_widget(status, area);
}

fn render_tabs<B>(f: &mut Frame<B>, state: &mut TUIState, area: Rect)
where
    B: Backend,
{
    let titles = ["(L)ogs", "(C)lasses", "(H)eap", "(G)C", "(R)resources"]
        .iter()
        .map(|t| Spans::from(Span::styled(*t, Style::default().fg(Color::Gray))))
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(state.current_tab);

    f.render_widget(tabs, area);
}

fn render_log<B>(f: &mut Frame<B>, state: &mut TUIState, area: Rect)
where
    B: Backend,
{
    let info_style = Style::default().fg(Color::Blue);
    let warning_style = Style::default().fg(Color::Yellow);
    let error_style = Style::default().fg(Color::Red);
    let unknown_style = Style::default().fg(Color::White);

    let logs: Vec<ListItem> = state
        .log_lines
        .iter()
        .map(|msg| {
            //TODO: make this more robust
            let msg = msg.strip_prefix(' ').unwrap_or(msg);
            let (level, style) = if msg.starts_with("INFO") {
                ("INFO", info_style)
            } else if msg.starts_with("WARN") {
                ("WARN", warning_style)
            } else if msg.starts_with("ERROR") {
                ("ERROR", error_style)
            } else {
                ("????", unknown_style)
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
