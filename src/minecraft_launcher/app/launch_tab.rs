use crate::minecraft_launcher::app::{Tab, TabTrait, Action, TabBinding};
use tui::Frame;
use tui::layout::Rect;
use tui::backend::CrosstermBackend;
use crossterm::event::KeyCode;
use std::io::Stdout;
use crate::minecraft_launcher::manifest::version;
use crate::minecraft_launcher::arguments;
use crate::minecraft_launcher::arguments::LaunchOptions;
use crate::minecraft_launcher::rendering::utils::StatefulList;
use crate::minecraft_launcher::launch;
use crate::minecraft_launcher::path;
use std::path::PathBuf;
use tui::widgets::{List, Block, Borders, ListItem};
use tui::text::Span;

pub struct GameLogTab {
    launch_options: Option<LaunchOptions>,
    game_logs: StatefulList<String>
}

impl GameLogTab {
    pub fn new() -> GameLogTab {
        GameLogTab { launch_options: None, game_logs: StatefulList::new() }
    }

    pub fn init(&mut self, version: &version::Main, player_name: String, player_uuid: String, player_token: String, user_type: String) {
        match LaunchOptions::from_version(version, player_name, player_uuid, player_token, user_type) {
            Ok(launch_options) => {
                self.launch_options = Some(launch_options);
            }
            Err(err) => {}
        };


        match self.launch_options.clone() {
            None => {}
            Some(mut launch_options) => {
                if let Some(args) = arguments::get_args_from_manifest(version, &launch_options) {
                    match path::get_java_executable_path(version) {
                        Ok(java_exe) => {
                            let result = launch::main(java_exe, launch_options.fill_argument_list(args));
                            if result.2.len() > 0 {
                                let erreur = String::from_utf8(result.2).unwrap();
                                let erreurs: Vec<&str> = erreur.split("\n").collect();
                                let mut erreur: Vec<String> = Vec::new();
                                for err in erreurs {
                                    erreur.push(err.to_string());
                                }
                                self.game_logs = StatefulList::with_items(erreur);
                            } else if result.1.len() > 0 {
                                let log = String::from_utf8(result.1).unwrap();
                                let logs: Vec<&str> = log.split("\n").collect();
                                let mut log: Vec<String> = Vec::new();
                                for err in logs {
                                    log.push(err.to_string());
                                }
                                self.game_logs = StatefulList::with_items(log);
                            }
                        }
                        Err(_) => {}
                    }
                }
            }
        };
    }
}

impl TabTrait for GameLogTab {
    fn render(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let list_items: Vec<ListItem> = self.game_logs
            .items
            .iter()
            .map(|line| {
                ListItem::new(Span::raw(line))
            }).collect();

        let list = List::new(list_items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_symbol("> ");

        f.render_stateful_widget(list, area, &mut self.game_logs.state)
    }

    fn on_key_press(&mut self, key_code: KeyCode) -> Action {
        match key_code {
            KeyCode::Up => {
                self.game_logs.previous();
                Action::None
            }
            KeyCode::Down => {
                self.game_logs.next();
                Action::None
            }
            _ => Action::None
        }
    }

    fn get_bindings(&self) -> Vec<TabBinding> {
        vec![]
    }
}
