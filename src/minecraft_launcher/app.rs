use crate::minecraft_launcher::manifest::main::MinVersion;
use crate::minecraft_launcher::rendering::utils::StatefulTable;

pub struct App {
    pub version_tab: VersionTab,
    pub current_tab: Tab
}

pub enum Tab {
    Version,
    Mod,
    ModVersion
}

pub struct VersionTab {
    pub selected: Option<String>,
    pub snapshot: bool,
    pub old: bool,
    pub all_versions: Vec<MinVersion>,
    pub current_table: StatefulTable<MinVersion>
}

impl VersionTab {
    pub fn build_table_state(&self) {
        let mut items: Vec<MinVersion> = Vec::new();

        for version in self.all_versions.clone() {
            if (self.snapshot && version._type.is_snapshot())
                || (self.old && version._type.is_old())
            {
                items.push(version.clone());
            }
        }

        StatefulTable::with_items(items);
    }

    pub fn render() {

    }

    pub fn select(&mut self, list: StatefulTable<MinVersion>) {
        match list.items.get(list.state.selected().expect(":flushed:")) {
            None => self.selected = None,
            Some(version) => self.selected = Some(version.clone().id),
        }
    }
}
