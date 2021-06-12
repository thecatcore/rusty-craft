use crate::minecraft_launcher::app::{Action, Tab, TabBinding, TabTrait};
use crate::minecraft_launcher::install;

use crate::minecraft_launcher::manifest::main::{MinVersion, Version};
use crate::minecraft_launcher::manifest::version;

use crate::minecraft_launcher::modding::ModLoaderInstaller;
use crossterm::event::KeyCode;
use std::io::Stdout;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::Spans;
use tui::widgets::{Block, Borders, Gauge, Paragraph, Wrap};
use tui::Frame;
use crate::minecraft_launcher::manifest::version::Main;

pub struct DownloadTab {
    rx: Option<Receiver<Message>>,
    current_step: u8,
    current_sub_step: Option<(String, u64, u64)>,
    current_sub_sub_step: Option<(String, u64, u64)>,
    error: Option<String>,
    installed: Option<version::Main>,
}

impl DownloadTab {
    pub fn new() -> DownloadTab {
        DownloadTab {
            rx: None,
            current_step: 1,
            current_sub_step: None,
            current_sub_sub_step: None,
            error: None,
            installed: None,
        }
    }

    pub fn start(
        &mut self,
        version: MinVersion,
        versions: Vec<Version>,
        loader: Box<dyn ModLoaderInstaller>,
        loader_version: Option<String>,
    ) {
        let (tx, rx) = mpsc::channel();

        let modded_version = if !loader.is_vanilla() {
            match loader.create_profile(version.id.clone(), match loader_version {
                None => "".to_string(),
                Some(v) => v
            }) {
                Ok(version) => Some(version),
                Err(_) => None
            }
        } else {
            None
        };

        thread::spawn(move || {
            tx.send(Message::Init)
                .expect("Cannot send message to receiver!");
            match install::install_version(version.clone().id, versions, tx, modded_version) {
                None => {
                    // panic!("Failed to install version {}", version.id)
                }
                Some(_) => {
                    // panic!("Successfully installed version {}", version.id)
                }
            }
        });

        self.rx = Some(rx);
    }
}

impl TabTrait for DownloadTab {
    fn render(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        match &self.rx {
            None => {}
            Some(rx) => {
                let mut init = false;
                if let Ok(msg) = rx.recv() {
                    match msg {
                        Message::Init => {
                            init = true;
                        }
                        Message::NewStep(step) => {
                            self.current_step = step;
                            self.current_sub_step = None;
                            self.current_sub_sub_step = None;
                        }
                        Message::NewSubStep(name, index, max) => {
                            self.current_sub_step = Some((name, index, max));
                            self.current_sub_sub_step = None;
                        }
                        Message::NewSubSubStep(name, index, max) => {
                            self.current_sub_sub_step = Some((name, index, max))
                        }
                        Message::Error(err) => {
                            self.error = Some(err);
                        }
                        Message::Done(version) => {
                            self.installed = Some(version);
                        }
                    }
                }

                let chunks = Layout::default()
                    .constraints([
                        Constraint::Ratio(2, 7),
                        Constraint::Ratio(2, 7),
                        Constraint::Ratio(2, 7),
                        Constraint::Ratio(1, 7),
                    ])
                    .split(area);

                let main_gauge = Gauge::default()
                    .block(Block::default().borders(Borders::ALL))
                    .gauge_style(Style::default().bg(Color::White).fg(Color::Black))
                    .ratio(self.current_step as f64 / 8.0)
                    .label(format!(
                        "{}/8 - {}",
                        self.current_step,
                        get_step_name(self.current_step)
                    ));
                f.render_widget(main_gauge, chunks[0]);

                match self.current_sub_step.clone() {
                    None => {}
                    Some(tuple) => {
                        let sub_gauge = Gauge::default()
                            .block(Block::default().borders(Borders::ALL))
                            .gauge_style(Style::default().bg(Color::White).fg(Color::Black))
                            .ratio(tuple.1 as f64 / tuple.2 as f64)
                            .label(format!("{}/{} - {}", tuple.1, tuple.2, tuple.0));
                        f.render_widget(sub_gauge, chunks[1]);
                    }
                }

                match self.current_sub_sub_step.clone() {
                    None => {}
                    Some(tuple) => {
                        let percent = ((tuple.1 as f64 / tuple.2 as f64) * 100.0) as u16;
                        let sub_gauge = Gauge::default()
                            .block(Block::default().borders(Borders::ALL))
                            .gauge_style(Style::default().bg(Color::White).fg(Color::Black))
                            .percent(percent)
                            .label(format!("{}/{} - {}", tuple.1, tuple.2, tuple.0));
                        f.render_widget(sub_gauge, chunks[2]);
                    }
                }

                match self.error.clone() {
                    None => {}
                    Some(err) => {
                        let paragraph = Paragraph::new(Spans::from(err)).wrap(Wrap { trim: true });
                        f.render_widget(paragraph, chunks[3]);
                    }
                }

                if !init {
                    let mut iterations = 1;
                    let mut res = rx.recv_timeout(Duration::from_millis(1));
                    while res.is_ok() {
                        let mut skipable = true;
                        if let Ok(msg) = res.clone() {
                            match msg {
                                Message::Init => {
                                    skipable = false;
                                }
                                Message::NewStep(step) => {
                                    self.current_step = step;
                                    self.current_sub_step = None;
                                    self.current_sub_sub_step = None;
                                    skipable = false;
                                }
                                Message::NewSubStep(name, index, max) => {
                                    self.current_sub_step = Some((name, index, max));
                                    self.current_sub_sub_step = None;
                                }
                                Message::NewSubSubStep(name, index, max) => {
                                    self.current_sub_sub_step = Some((name, index, max))
                                }
                                Message::Error(err) => {
                                    self.error = Some(err);
                                    skipable = false;
                                }
                                Message::Done(version) => {
                                    self.installed = Some(version);
                                    skipable = false;
                                }
                            }
                        }
                        if iterations > 100 {
                            break;
                        }
                        if skipable {
                            res = rx.recv_timeout(Duration::from_millis(1));
                            iterations += 1;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
    }

    fn on_key_press(&mut self, _key_code: KeyCode) -> Action {
        Action::None
    }

    fn tick(&mut self) -> Action {
        match self.installed.clone() {
            None => Action::None,
            Some(version) => Action::NextTab(Tab::Launch(version)),
        }
    }

    fn get_bindings(&self) -> Vec<TabBinding> {
        vec![]
    }
}

fn get_step_name(index: u8) -> &'static str {
    match index {
        1 => "Checking version manifest",
        2 => "Checking Java version",
        3 => "Checking client jar",
        4 => "Checking libraries",
        5 => "Checking assets",
        6 => "Checking log file",
        7 => "Extracting natives",
        _ => "Done",
    }
}

#[derive(Clone)]
pub enum Message {
    Init,
    NewStep(u8),
    NewSubStep(String, u64, u64),
    NewSubSubStep(String, u64, u64),
    Error(String),
    Done(version::Main),
}
