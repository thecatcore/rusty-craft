use crate::minecraft_launcher::app::{Action, Tab, TabBinding, TabTrait};
use crate::minecraft_launcher::manifest::main::{MinVersion, Version};
use crate::minecraft_launcher::modding;
use crate::minecraft_launcher::rendering::utils::StatefulTable;
use crossterm::event::KeyCode;
use std::io::Stdout;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Modifier, Style};
use tui::text::Span;
use tui::widgets::{Block, Borders, Cell, Row, Table};
use tui::Frame;

pub struct VersionTab {
    pub selected: Option<MinVersion>,
    pub selected_modloader: Option<Box<dyn modding::ModLoaderInstaller>>,
    pub selected_modloader_version: Option<String>,
    pub snapshot: bool,
    pub old: bool,
    pub all_versions: Vec<MinVersion>,
    pub current_table: StatefulTable<MinVersion>,
    pub versions: Vec<Version>,
    pub modding_handler: modding::ModLoaderHandler,
}

impl VersionTab {
    pub fn build_table_state(&mut self) {
        let mut items: Vec<MinVersion> = Vec::new();

        for version in self.all_versions.clone() {
            if version._type.is_release()
                || (self.snapshot && version._type.is_snapshot())
                || (self.old && version._type.is_old())
            {
                items.push(version.clone())
            }
        }

        self.current_table = StatefulTable::with_items(items);
    }

    fn render_version_list(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let version_list: Vec<Row> = self
            .current_table
            .items
            .iter()
            .map(|v| {
                let cells = vec![
                    Cell::from(Span::raw(v.id.to_string())),
                    Cell::from(Span::raw(v._type.to_string())),
                    Cell::from(Span::raw(
                        match v.installed {
                            true => "Yes",
                            false => "No",
                        }
                        .to_string(),
                    )),
                    Cell::from(Span::raw(format!("{:?}", v.release_time))),
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
                Constraint::Ratio(4, 12),
            ]);

        f.render_stateful_widget(table, area, &mut self.current_table.state);
    }

    pub fn select(&mut self) {
        match self
            .current_table
            .items
            .get(self.current_table.state.selected().expect(":flushed:"))
        {
            None => self.selected = None,
            Some(version) => self.selected = Some(version.clone()),
        }
    }
}

impl TabTrait for VersionTab {
    fn render(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 1)])
            .split(area);

        if self.selected_modloader.is_some() {
            let selected_modloader = self.selected_modloader.take().unwrap();

            self.selected_modloader = Some(selected_modloader);
        } else if let Some(version) = self.selected.clone() {
        } else {
            self.render_version_list(f, chunks[0]);
        }
    }

    fn on_key_press(&mut self, key_code: KeyCode) -> Action {
        match key_code {
            KeyCode::Enter => {
                self.select();
                match &self.selected {
                    None => Action::None,
                    Some(version) => {
                        Action::NextTab(Tab::Download(version.clone(), self.versions.clone()))
                    }
                }
            }
            KeyCode::Up => {
                self.current_table.previous();
                Action::None
            }
            KeyCode::Down => {
                self.current_table.next();
                Action::None
            }
            KeyCode::Char('s') => {
                self.snapshot = !self.snapshot;
                self.build_table_state();
                Action::None
            }
            KeyCode::Char('o') => {
                self.old = !self.old;
                self.build_table_state();
                Action::None
            }
            _ => Action::None,
        }
    }

    fn get_bindings(&self) -> Vec<TabBinding> {
        let mut vec = Vec::new();

        vec.push(TabBinding::Default(
            String::from("ENTER"),
            String::from("Install and Launch selected version"),
        ));
        vec.push(TabBinding::Default(
            String::from("UP"),
            String::from("Move selector up"),
        ));
        vec.push(TabBinding::Default(
            String::from("DOWN"),
            String::from("Move selector down"),
        ));
        vec.push(TabBinding::Enablable(
            String::from("S"),
            String::from("Show/Hide snapshots"),
            self.snapshot,
        ));
        vec.push(TabBinding::Enablable(
            String::from("O"),
            String::from("Show/Hide old betas and alphas"),
            self.old,
        ));

        vec
    }
}
