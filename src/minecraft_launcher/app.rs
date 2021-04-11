use crate::minecraft_launcher::manifest::main::{MinVersion, Version};
use crate::minecraft_launcher::rendering::utils::StatefulTable;
use crossterm::event::KeyCode;
use crate::minecraft_launcher::install;
use tui::Frame;
use tui::layout::{Rect, Layout, Direction, Constraint};
use tui::widgets::{Row, Table, Block, Borders, Cell};
use tui::text::Span;
use tui::style::{Style, Modifier};
use tui::backend::CrosstermBackend;
use std::io::Stdout;

pub struct App {
    pub version_tab: VersionTab,
    pub current_tab: Tab
}

impl App {
    pub fn new(min_versions: Vec<MinVersion>, versions: Vec<Version>) -> App {
        let mut app = App {
            version_tab: VersionTab {
                selected: None,
                snapshot: false,
                old: false,
                all_versions: min_versions.clone(),
                current_table: StatefulTable::with_items(min_versions),
                versions
            },
            current_tab: Tab::Version
        };
        app.version_tab.build_table_state();
        app
    }

    pub fn render(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        match self.current_tab {
            Tab::Version => {
                self.version_tab.render(f, area);
            }
            Tab::Mod => {}
            Tab::ModVersion => {}
        }
    }

    pub fn on_key_press(&mut self, key_code: KeyCode) -> KeyCode {
        let key = key_code.clone();
        match self.current_tab {
            Tab::Version => {
                self.version_tab.on_key_press(key_code);
            }
            Tab::Mod => {}
            Tab::ModVersion => {}
        }
        key
    }
}

pub enum Tab {
    Version,
    Mod,
    ModVersion
}

#[derive(Clone)]
pub struct VersionTab {
    pub selected: Option<MinVersion>,
    pub snapshot: bool,
    pub old: bool,
    pub all_versions: Vec<MinVersion>,
    pub current_table: StatefulTable<MinVersion>,
    pub versions: Vec<Version>
}

impl VersionTab {
    pub fn build_table_state(&mut self) {
        let mut items: Vec<MinVersion> = Vec::new();

        for version in self.all_versions.clone() {
            if (self.snapshot == version._type.is_snapshot())
                && (self.old == version._type.is_old())
            {

                items.push(version.clone());
            }
        }

        self.current_table = StatefulTable::with_items(items);
    }

    pub fn render(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        // self.build_table_state();
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 1)])
            .split(area);

        let version_list: Vec<Row> = self.current_table
            .items
            .iter()
            .map(|v| {
                let cells = vec![
                    Cell::from(Span::raw(format!("{}", v.id))),
                    Cell::from(Span::raw(format!("{}", v._type.to_string()))),
                    Cell::from(Span::raw(format!(
                        "{}",
                        match v.installed {
                            true => {
                                "Yes"
                            }
                            false => {
                                "No"
                            }
                        }
                    ))),
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

        f.render_stateful_widget(table, chunks[0], &mut self.current_table.state);
    }

    pub fn on_key_press(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Enter => {
                self.select();
                match &self.selected {
                    None => {}
                    Some(version) => {
                        match install::install_version(version.clone().id, self.clone().versions) {
                            None => {
                                panic!("Failed to install version {}", version.id)
                            }
                            Some(_) => {
                                panic!("Successfully installed version {}", version.id)
                            }
                        }
                    }
                }
            }
            KeyCode::Up => {
                self.current_table.previous();
            }
            KeyCode::Down => {
                self.current_table.next();
            }
            _ => {}
        }
    }

    pub fn select(&mut self) {
        match self.current_table.items.get(self.current_table.state.selected().expect(":flushed:")) {
            None => self.selected = None,
            Some(version) => self.selected = Some(version.clone()),
        }
    }
}
