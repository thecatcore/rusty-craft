use crate::minecraft_launcher::app::{Action, Tab, TabBinding, TabTrait};
use crate::minecraft_launcher::arguments;
use crate::minecraft_launcher::arguments::LaunchOptions;
use crate::minecraft_launcher::launch;
use crate::minecraft_launcher::manifest::version;
use crate::minecraft_launcher::path;
use crate::minecraft_launcher::rendering::utils::StatefulList;
use crossterm::event::KeyCode;
use std::io::{Read, Stdout};
use std::path::PathBuf;
use std::process::{Child, ChildStderr, ChildStdout};
use tui::backend::CrosstermBackend;
use tui::layout::Rect;
use tui::text::Span;
use tui::widgets::{Block, Borders, List, ListItem};
use tui::Frame;

pub struct GameLogTab {
    launch_options: Option<LaunchOptions>,
    game_logs: StatefulList<String>,
    child_process: Option<Child>,
    child_stdout: Option<ChildStdout>,
    child_stderr: Option<ChildStderr>,
}

impl GameLogTab {
    pub fn new() -> GameLogTab {
        GameLogTab {
            launch_options: None,
            game_logs: StatefulList::new(),
            child_process: None,
            child_stdout: None,
            child_stderr: None,
        }
    }

    pub fn init(
        &mut self,
        version: &version::Main,
        player_name: String,
        player_uuid: String,
        player_token: String,
        user_type: String,
    ) {
        match LaunchOptions::from_version(
            version,
            player_name,
            player_uuid,
            player_token,
            user_type,
        ) {
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
                            self.child_process = Some(launch::main(
                                java_exe,
                                launch_options.fill_argument_list(args),
                            ));
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
        let list_items: Vec<ListItem> = self
            .game_logs
            .items
            .iter()
            .map(|line| ListItem::new(Span::raw(line)))
            .collect();

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
            _ => Action::None,
        }
    }

    fn tick(&mut self) -> Action {
        if self.child_process.is_some() {
            let mut child = self.child_process.take().unwrap();
            self.child_stdout = child.stdout.take();
            self.child_stderr = child.stderr.take();
            self.child_process = Some(child);
        }

        if self.child_process.is_some()
            && self.child_stdout.is_some()
            && self.child_stderr.is_some()
        {
            let mut stdout = self.child_stdout.take().unwrap();
            let mut stderr = self.child_stderr.take().unwrap();

            let mut stdout_string = String::new();
            stdout.read_to_string(&mut stdout_string);

            let mut stderr_string = String::new();
            stderr.read_to_string(&mut stderr_string);

            let mut lines: Vec<String> = Vec::new();

            lines.push("==========Stdout=========".to_string());

            let stdout_lines: Vec<&str> = stdout_string.split("\n").collect();

            for stdout_line in stdout_lines {
                lines.push(stdout_line.to_string());
            }

            lines.push("==========Stderr=========".to_string());

            let stderr_lines: Vec<&str> = stderr_string.split("\n").collect();

            for stderr_line in stderr_lines {
                lines.push(stderr_line.to_string());
            }

            if self.game_logs.items.len() < lines.len() {
                self.game_logs = StatefulList::with_items(lines);
            }
            self.child_stdout = Some(stdout);
            self.child_stderr = Some(stderr);
        }

        Action::None
    }

    fn get_bindings(&self) -> Vec<TabBinding> {
        vec![]
    }
}
