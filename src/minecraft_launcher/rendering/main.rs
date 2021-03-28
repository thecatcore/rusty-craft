use std::{error::Error, io, thread};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal, Frame};
use std::time::{Duration, Instant};
use crate::minecraft_launcher::manifest::main::MinVersion;
use tui::layout::{Layout, Constraint, Rect, Direction};
use tui::widgets::{Tabs, Borders, Block, Row, Table, Cell, List, ListItem};
use tui::text::{Spans, Span};
use tui::style::{Style, Color, Modifier};
use tui::backend::Backend;

use crate::minecraft_launcher::rendering::utils::{StatefulList, StatefulTable};
use std::sync::mpsc;

enum Event<I> {
    Input(I),
    Tick,
}

/// Crossterm demo
#[derive(Debug)]
struct Cli {
    /// time in ms between two ticks.
    tick_rate: u64,
    /// whether unicode symbols are used to improve the overall look of the app
    enhanced_graphics: bool,
}

pub fn main(versions: &Vec<(MinVersion, bool)>) -> Result<(), Box<dyn Error>> {

    // let events = Events::new();
    let cli = Cli {
        tick_rate: 250,
        enhanced_graphics: true
    };

    let mut items: Vec<(MinVersion, bool)> = Vec::new();

    for version in versions.clone() {
        items.push((version.clone().0, version.1));
    }

    let mut list = StatefulTable::with_items(items);

    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, rx) = mpsc::channel();

    let tick_rate = Duration::from_millis(cli.tick_rate);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    tx.send(Event::Input(key)).unwrap();
                }
            }
            if last_tick.elapsed() >= tick_rate {
                tx.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });


    terminal.clear()?;

    loop {
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(f.size());

            let mut ve: Vec<Spans> = Vec::new();
            ve.append(&mut vec![Spans::from("Test")]);

            let tabs = Tabs::new(ve)
                .block(Block::default().borders(Borders::ALL))
                .highlight_style(Style::default().fg(Color::Yellow))
                .select(0);
            f.render_widget(tabs, chunks[0]);
            draw_tab(f, chunks[1], &mut list)
        })?;

        match rx.recv()? {
            Event::Input(key) => match key.code {
                // Key::Backspace => {}
                KeyCode::Left => {}
                KeyCode::Right => {}
                KeyCode::Up => {
                    list.previous();
                }
                KeyCode::Down => {
                    list.next();
                }
                KeyCode::Char(_c) => {}
                KeyCode::Esc => {
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    terminal.show_cursor()?;
                    break
                }
                KeyCode::Enter => {}
                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}

fn draw_tab<B: Backend>(f: &mut Frame<B>, area: Rect, versions: &mut StatefulTable<(MinVersion, bool)>) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 1)])
        .split(area);

    let version_list: Vec<Row> = versions
        .items
        .iter()
        .map(|v| {
            let cells = vec![
                Cell::from(Span::raw(format!("{}", v.0.id))),
                Cell::from(Span::raw(format!("{}", v.0._type.to_string()))),
                Cell::from(Span::raw(format!("{}", match v.1 {
                    true => { "Yes" }
                    false => { "No" }
                }))),
                Cell::from(Span::raw(format!("{:?}", v.0.release_time))),
            ];
            Row::new(cells)
        })
        .collect();

    let table = Table::new(version_list)
        .block(Block::default().borders(Borders::ALL).title("Version List"))
        .header(Row::new(vec!["Name", "Type", "Installed", "Release Date"]))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ")
        .widths(&[
            Constraint::Ratio(5, 12),
            Constraint::Ratio(3, 24),
            Constraint::Ratio(5, 36),
            Constraint::Ratio(4, 12)
    ]);

    // let table = List::new(version_list)
    //     .block(Block::default().borders(Borders::ALL).title("Version List"))
    //     .highlight_style(Style::default().add_modifier(Modifier::BOLD))
    //     .highlight_symbol("> ");

    f.render_stateful_widget(table, chunks[0], &mut versions.state);
}