use crate::minecraft_launcher::app::{Action, Tab, TabBinding, TabTrait};
use crate::minecraft_launcher::manifest::main::{MinVersion, Version};
use crate::minecraft_launcher::modding;
use crate::minecraft_launcher::rendering::utils::{StatefulList, StatefulTable};
use crossterm::event::KeyCode;

use std::io::Stdout;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Modifier, Style};
use tui::text::Span;
use tui::widgets::{Block, Borders, Cell, List, ListItem, Row, Table};
use tui::Frame;

pub struct VersionTab {
    pub selected: Option<MinVersion>,
    pub selected_mod_loader: Option<Box<dyn modding::ModLoaderInstaller>>,
    pub selected_mod_loader_version: Option<String>,
    pub snapshot: bool,
    pub old: bool,
    pub all_versions: Vec<MinVersion>,
    pub mc_version_table: StatefulTable<MinVersion>,
    pub loader_list: StatefulList<Box<dyn modding::ModLoaderInstaller>>,
    pub loader_version_list: StatefulList<String>,
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

        self.mc_version_table = StatefulTable::with_items(items);
    }

    pub fn build_mod_loader_list(&mut self) {
        let items = match &self.selected {
            None => vec![],
            Some(min_version) => match self
                .modding_handler
                .get_loaders_for_version(min_version.id.clone())
            {
                Ok(loaders) => loaders,
                Err(_err) => vec![],
            },
        };

        self.loader_list = StatefulList::with_items_oob(items);
    }

    pub fn build_mod_loader_version_list(&mut self) {
        let items = match &self.selected_mod_loader {
            None => vec![],
            Some(mod_loader) => {
                match mod_loader.get_loader_versions(self.selected.clone().unwrap().id) {
                    Ok(versions) => {
                        let mut list = vec![];

                        for version in versions.iter() {
                            list.push(version.0.clone())
                        }

                        list
                    }
                    Err(err) => panic!("{}", err),
                }
            }
        };

        self.loader_version_list = StatefulList::with_items_oob(items);
    }

    fn render_version_list(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let version_list: Vec<Row> = self
            .mc_version_table
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

        f.render_stateful_widget(table, area, &mut self.mc_version_table.state);
    }

    fn render_loader_list(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let loader_list: Vec<ListItem> = self
            .loader_list
            .items
            .iter()
            .map(|loader| ListItem::new(Span::raw(loader.get_name())))
            .collect();

        let list = List::new(loader_list)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Mod Loader List"),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");

        f.render_stateful_widget(list, area, &mut self.loader_list.state)
    }

    fn render_loader_version_list(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let loader_version_list: Vec<ListItem> = self
            .loader_version_list
            .items
            .iter()
            .map(|loader| ListItem::new(Span::raw(loader)))
            .collect();

        let list = List::new(loader_version_list)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Mod Loader Version List"),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");

        f.render_stateful_widget(list, area, &mut self.loader_version_list.state)
    }

    pub fn select(&mut self) {
        if self.selected_mod_loader.is_some() {
            match self.loader_version_list.selected() {
                None => self.selected_mod_loader_version = None,
                Some(version) => self.selected_mod_loader_version = Some(version.clone()),
            }
        } else if self.selected.is_some() {
            match self.loader_list.selected() {
                None => self.selected_mod_loader = None,
                Some(mod_loader) => self.selected_mod_loader = Some(mod_loader.clone_instance()),
            }
        } else {
            match self
                .mc_version_table
                .items
                .get(self.mc_version_table.state.selected().unwrap_or(0))
            {
                None => self.selected = None,
                Some(version) => self.selected = Some(version.clone()),
            }
        }
    }
}

impl TabTrait for VersionTab {
    fn render(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 1)])
            .split(area);

        if self.selected_mod_loader.is_some() {
            self.render_loader_version_list(f, chunks[0]);
        } else if self.selected.is_some() {
            self.render_loader_list(f, chunks[0])
        } else {
            self.render_version_list(f, chunks[0]);
        }
    }

    fn on_key_press(&mut self, key_code: KeyCode) -> Action {
        match key_code {
            KeyCode::Enter => {
                self.select();

                match &self.selected_mod_loader_version {
                    None => match &self.selected_mod_loader {
                        None => match &self.selected {
                            None => Action::None,
                            Some(_version) => {
                                self.build_mod_loader_list();
                                Action::None
                            }
                        },
                        Some(mod_loader) => {
                            if mod_loader.is_vanilla() {
                                Action::NextTab(Tab::Download(
                                    self.selected.clone().unwrap(),
                                    self.versions.clone(),
                                    mod_loader.clone_instance(),
                                    None,
                                ))
                            } else {
                                self.build_mod_loader_version_list();
                                Action::None
                            }
                        }
                    },
                    Some(mod_loader_version) => Action::NextTab(Tab::Download(
                        self.selected.clone().unwrap(),
                        self.versions.clone(),
                        self.selected_mod_loader.take().unwrap(),
                        Some(mod_loader_version.clone()),
                    )),
                }
            }
            KeyCode::Up => {
                if self.selected_mod_loader.is_some() {
                    self.loader_version_list.previous();
                } else if self.selected.is_some() {
                    self.loader_list.previous();
                } else {
                    self.mc_version_table.previous();
                }
                Action::None
            }
            KeyCode::Down => {
                if self.selected_mod_loader.is_some() {
                    self.loader_version_list.next();
                } else if self.selected.is_some() {
                    self.loader_list.next();
                } else {
                    self.mc_version_table.next();
                }
                Action::None
            }
            KeyCode::Left => {
                if self.selected_mod_loader_version.is_some() {
                    self.selected_mod_loader_version = None
                } else if self.selected_mod_loader.is_some() {
                    self.selected_mod_loader = None
                } else if self.selected.is_some() {
                    self.selected = None
                }

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
