use crate::minecraft_launcher::manifest::main::MinVersion;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::time::{Duration, Instant};
use std::{error::Error, io, thread};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, List, ListItem, Row, Table, Tabs},
    Frame, Terminal,
};

use crate::minecraft_launcher::app::App;
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

pub fn main(mut app: App) -> Result<(), Box<dyn Error>> {
    // let events = Events::new();
    let cli = Cli {
        tick_rate: 250,
        enhanced_graphics: true,
    };

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
            app.render(f, chunks[1])
        })?;

        match rx.recv()? {
            Event::Input(key) => match app.on_key_press(key.code) {
                KeyCode::Esc => {
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    terminal.show_cursor()?;
                    break;
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}
