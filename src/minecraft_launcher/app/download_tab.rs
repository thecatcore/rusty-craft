use crate::minecraft_launcher::app::Action;
use crate::minecraft_launcher::install;
use crate::minecraft_launcher::manifest::main::{MinVersion, Version};
use crossterm::event::KeyCode;
use std::io::Stdout;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvError, SendError, Sender};
use std::thread;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Gauge};
use tui::Frame;

pub struct DownloadTab {
    rx: Option<Receiver<Message>>,
    current_step: u8,
    current_sub_step: Option<(String, u64, u64)>,
    current_sub_sub_step: Option<(String, u64, u64)>,
}

impl DownloadTab {
    pub fn new() -> DownloadTab {
        DownloadTab {
            rx: None,
            current_step: 1,
            current_sub_step: None,
            current_sub_sub_step: None,
        }
    }

    pub fn start(&mut self, version: MinVersion, versions: Vec<Version>) {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            tx.send(Message::Init)
                .expect("Cannot send message to receiver!");
            match install::install_version(version.clone().id, versions, tx) {
                None => {
                    panic!("Failed to install version {}", version.id)
                }
                Some(_) => {
                    panic!("Successfully installed version {}", version.id)
                }
            }
        });

        self.rx = Some(rx);
    }

    pub fn render(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        match &self.rx {
            None => {}
            Some(rx) => {
                match rx.recv() {
                    Ok(msg) => match msg {
                        Message::Init => {}
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
                    },
                    Err(err) => {
                        // println!("Error while trying to receive message from install thread: {}", err)
                    }
                }

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Ratio(1, 3),
                        Constraint::Ratio(1, 3),
                        Constraint::Ratio(1, 3),
                    ])
                    .split(area);

                let main_gauge = Gauge::default()
                    .block(Block::default().borders(Borders::ALL))
                    .gauge_style(Style::default().bg(Color::White).fg(Color::Black))
                    .ratio((self.current_step as f64 / 7.0) as f64)
                    .label(format!(
                        "{}/6 - {}",
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
                            .ratio((tuple.1 as f64 / tuple.2 as f64) as f64)
                            .label(format!("{}/{} - {}", tuple.1, tuple.2, tuple.0));
                        f.render_widget(sub_gauge, chunks[1]);
                    }
                }

                match self.current_sub_sub_step.clone() {
                    None => {}
                    Some(tuple) => {
                        let sub_gauge = Gauge::default()
                            .block(Block::default().borders(Borders::ALL))
                            .gauge_style(Style::default().bg(Color::White).fg(Color::Black))
                            .ratio((tuple.1 as f64 / tuple.2 as f64) as f64)
                            .label(format!("{}/{} - {}", tuple.1, tuple.2, tuple.0));
                        f.render_widget(sub_gauge, chunks[2]);
                    }
                }
            }
        }
    }

    pub fn on_key_press(&mut self, key_code: KeyCode) -> Action {
        Action::None
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
        _ => "Done",
    }
}

pub enum Message {
    Init,
    NewStep(u8),
    NewSubStep(String, u64, u64),
    NewSubSubStep(String, u64, u64),
}
