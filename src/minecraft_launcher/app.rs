use crate::minecraft_launcher::manifest::main::{MinVersion, Version};
use crate::minecraft_launcher::manifest::version;
use crate::minecraft_launcher::rendering::main::{Cli, Event};
use crate::minecraft_launcher::rendering::utils::StatefulTable;
use crossterm::event::KeyEvent;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::error::Error;
use std::io::Stdout;
use std::sync::mpsc;
use std::sync::mpsc::SendError;
use std::time::{Duration, Instant};
use std::{io, thread};
use tui::backend::CrosstermBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::Span;
use tui::text::Spans;
use tui::widgets::{Block, Borders, Paragraph, Tabs};
use tui::{Frame, Terminal};

pub mod download_tab;
mod launch_tab;
mod log_tab;
mod login_tab;
mod version_tab;

pub struct App {
    pub login_tab: login_tab::LoginTab,
    pub version_tab: version_tab::VersionTab,
    pub download_tab: download_tab::DownloadTab,
    pub current_tab: Tab,
}

impl App {
    pub fn new(min_versions: Vec<MinVersion>, versions: Vec<Version>) -> App {
        let mut app = App {
            login_tab: login_tab::LoginTab::new(),
            version_tab: version_tab::VersionTab {
                selected: None,
                snapshot: false,
                old: false,
                all_versions: min_versions.clone(),
                current_table: StatefulTable::with_items(min_versions),
                versions,
            },
            download_tab: download_tab::DownloadTab::new(),
            current_tab: Tab::Login,
        };
        app.version_tab.build_table_state();
        app
    }

    pub fn render(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Percentage(30)])
            .split(area);

        match self.current_tab {
            Tab::Login => self.login_tab.render(f, chunks[0]),
            Tab::Version => self.version_tab.render(f, chunks[0]),
            Tab::Download(_, _) => self.download_tab.render(f, chunks[0]),
            Tab::Launch(_) => {}
            Tab::Mod => {}
            Tab::ModVersion => {}
        };

        let bindings = self.get_bindings();

        let mut spans = Vec::new();

        for binding in bindings {
            match binding {
                TabBinding::Default(key, desc) => {
                    spans.push(Spans::from(Span::styled(
                        format!("{}: {}", key, desc),
                        Style::default().fg(Color::Yellow),
                    )));
                }
                TabBinding::Enablable(key, desc, enabled) => {
                    let mut style = Style::default();

                    if enabled {
                        style = style.bg(Color::Yellow).fg(Color::Black);
                    } else {
                        style = style.fg(Color::Yellow);
                    }

                    let span = Span::styled(format!("{}: {}", key, desc), style);

                    spans.push(Spans::from(span));
                }
            }
        }

        let bindings_paragraph = Paragraph::new(spans)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Left);

        f.render_widget(bindings_paragraph, chunks[1])
    }

    fn get_bindings(&mut self) -> Vec<TabBinding> {
        let mut vec = Vec::new();

        vec.push(TabBinding::Default(
            String::from("ESC"),
            String::from("Quit App"),
        ));

        match self.current_tab {
            Tab::Login => {
                let tab_vec = self.login_tab.get_bindings();
                for tab_binding in tab_vec {
                    vec.push(tab_binding);
                }
            }
            Tab::Version => {
                let tab_vec = self.version_tab.get_bindings();
                for tab_binding in tab_vec {
                    vec.push(tab_binding);
                }
            }
            Tab::Download(_, _) => {
                let tab_vec = self.download_tab.get_bindings();
                for tab_binding in tab_vec {
                    vec.push(tab_binding);
                }
            }
            Tab::Launch(_) => {}
            Tab::Mod => {}
            Tab::ModVersion => {}
        }

        vec
    }

    fn tick(&mut self) -> Action {
        match self.current_tab {
            Tab::Login => self.login_tab.tick(),
            Tab::Version => self.version_tab.tick(),
            Tab::Download(_, _) => self.download_tab.tick(),
            Tab::Launch(_) => Action::None,
            Tab::Mod => Action::None,
            Tab::ModVersion => Action::None,
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
        // let tx2 = tx.clone();
        thread::spawn(move || -> Result<(), SendError<Event<KeyEvent>>> {
            let mut last_tick = Instant::now();
            loop {
                // poll for tick rate duration, if no events, sent tick event.
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));
                if event::poll(timeout).unwrap() {
                    if let CEvent::Key(key) = event::read().unwrap() {
                        tx.send(Event::Input(key))?;
                    }
                }
                if last_tick.elapsed() >= tick_rate {
                    tx.send(Event::Tick)?;
                    last_tick = Instant::now();
                }
            }
            // Ok(())
        });
        // thread::spawn(move || {
        //     let mut last_tick = Instant::now();
        //     loop {
        //         if last_tick.elapsed() >= Duration::from_millis(250) {
        //             match tx2.send(Event::Tick) {
        //                 Ok(_) => {
        //                     last_tick = Instant::now();
        //                 }
        //                 Err(_) => {}
        //             };
        //         }
        //     }
        // });

        terminal.clear()?;

        loop {
            let selected_tab = match self.current_tab.clone() {
                Tab::Login => 0,
                Tab::Version => 1,
                Tab::Download(_, _) => 2,
                Tab::Launch(_) => 3,
                Tab::Mod => 4,
                Tab::ModVersion => 5,
            };
            terminal.draw(|f| {
                let main_chunks = Layout::default()
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(0),
                        Constraint::Length(1),
                    ])
                    .split(f.size());

                let mut ve: Vec<Spans> = Vec::new();
                ve.append(&mut vec![
                    Spans::from("Login"),
                    Spans::from("Version"),
                    Spans::from("Installation"),
                    Spans::from("Launch"),
                ]);

                let tabs = Tabs::new(ve)
                    .block(Block::default().borders(Borders::ALL))
                    .highlight_style(Style::default().fg(Color::Yellow))
                    .select(selected_tab);
                f.render_widget(tabs, main_chunks[0]);
                self.render(f, main_chunks[1]);
                f.render_widget(
                    Paragraph::new(vec![Spans::from("© 2021 CatCore")]),
                    main_chunks[2],
                );
            })?;

            match rx.recv()? {
                Event::Input(key) => match key.code {
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
                    _ => {
                        match self.on_key_press(key.code) {
                            Action::None => {}
                            Action::NextTab(tab) => {
                                self.current_tab = tab.clone();
                                match tab {
                                    Tab::Login => {}
                                    Tab::Version => {}
                                    Tab::Download(v, ref vs) => {
                                        self.download_tab.start(v, vs.clone())
                                    }
                                    Tab::Launch(version) => {}
                                    Tab::Mod => {}
                                    Tab::ModVersion => {}
                                }
                            }
                        };
                    }
                },
                Event::Tick => match self.tick() {
                    Action::None => {}
                    Action::NextTab(tab) => {
                        self.current_tab = tab.clone();
                        match tab {
                            Tab::Login => {}
                            Tab::Version => {}
                            Tab::Download(v, ref vs) => self.download_tab.start(v, vs.clone()),
                            Tab::Launch(version) => {}
                            Tab::Mod => {}
                            Tab::ModVersion => {}
                        }
                    }
                },
            }
        }

        Ok(())
    }

    pub fn on_key_press(&mut self, key_code: KeyCode) -> Action {
        match self.current_tab {
            Tab::Login => self.login_tab.on_key_press(key_code),
            Tab::Version => self.version_tab.on_key_press(key_code),
            Tab::Download(_, _) => self.download_tab.on_key_press(key_code),
            Tab::Launch(_) => Action::None,
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
    Login,
    Version,
    Download(MinVersion, Vec<Version>),
    Launch(version::Main),
    Mod,
    ModVersion,
}

pub enum TabBinding {
    Default(String, String),
    Enablable(String, String, bool),
}

pub trait TabTrait {
    fn render(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect);
    fn on_key_press(&mut self, _key_code: KeyCode) -> Action;
    fn tick(&mut self) -> Action {
        Action::None
    }
    fn get_bindings(&self) -> Vec<TabBinding>;
}
