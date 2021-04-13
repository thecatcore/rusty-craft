use crate::minecraft_launcher::manifest::main::{MinVersion, Version};
use crate::minecraft_launcher::rendering::main::{Cli, Event};
use crate::minecraft_launcher::rendering::utils::StatefulTable;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::error::Error;
use std::io::Stdout;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{io, thread};
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::Spans;
use tui::widgets::{Block, Borders, Tabs};
use tui::{Frame, Terminal};

pub mod download_tab;
mod version_tab;

pub struct App {
    pub version_tab: version_tab::VersionTab,
    pub download_tab: download_tab::DownloadTab,
    pub current_tab: Tab,
}

impl App {
    pub fn new(min_versions: Vec<MinVersion>, versions: Vec<Version>) -> App {
        let mut app = App {
            version_tab: version_tab::VersionTab {
                selected: None,
                snapshot: false,
                old: false,
                all_versions: min_versions.clone(),
                current_table: StatefulTable::with_items(min_versions),
                versions,
            },
            download_tab: download_tab::DownloadTab::new(),
            current_tab: Tab::Version,
        };
        app.version_tab.build_table_state();
        app
    }

    pub fn render(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        match self.current_tab {
            Tab::Version => {
                self.version_tab.render(f, area);
            }
            Tab::Download(_, _) => {
                self.download_tab.render(f, area);
            }
            Tab::Mod => {}
            Tab::ModVersion => {}
        }
    }

    pub fn run(mut self) -> Result<(), Box<dyn Error>> {
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
            let selected_tab = match self.current_tab.clone() {
                Tab::Version => 0,
                Tab::Download(_, _) => 1,
                Tab::Mod => 2,
                Tab::ModVersion => 3,
            };
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                    .split(f.size());

                let mut ve: Vec<Spans> = Vec::new();
                ve.append(&mut vec![
                    Spans::from("Version"),
                    Spans::from("Installation"),
                ]);

                let tabs = Tabs::new(ve)
                    .block(Block::default().borders(Borders::ALL))
                    .highlight_style(Style::default().fg(Color::Yellow))
                    .select(selected_tab);
                f.render_widget(tabs, chunks[0]);
                self.render(f, chunks[1])
            })?;

            match rx.recv()? {
                Event::Input(key) => match key.code.clone() {
                    KeyCode::Esc => match disable_raw_mode() {
                        Ok(_) => {
                            match execute!(
                                terminal.backend_mut(),
                                LeaveAlternateScreen,
                                DisableMouseCapture
                            ) {
                                Ok(_) => match terminal.show_cursor() {
                                    Ok(_) => break,
                                    Err(_) => {}
                                },
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    },
                    _ => {
                        match self.on_key_press(key.code) {
                            Action::None => {}
                            Action::NextTab(tab) => {
                                self.current_tab = tab.clone();
                                match tab {
                                    Tab::Version => {}
                                    Tab::Download(v, ref vs) => {
                                        self.download_tab.start(v, vs.clone())
                                    }
                                    Tab::Mod => {}
                                    Tab::ModVersion => {}
                                }
                            }
                        };
                    }
                },
                Event::Tick => {}
            }
        }

        Ok(())
    }

    pub fn on_key_press(&mut self, key_code: KeyCode) -> Action {
        match self.current_tab {
            Tab::Version => self.version_tab.on_key_press(key_code),
            Tab::Download(_, _) => self.download_tab.on_key_press(key_code),
            Tab::Mod => Action::None,
            Tab::ModVersion => Action::None,
        }
    }
}

pub enum Action {
    None,
    NextTab(Tab),
}

#[derive(Clone)]
pub enum Tab {
    Version,
    Download(MinVersion, Vec<Version>),
    Mod,
    ModVersion,
}
