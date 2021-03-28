pub struct App {
    pub version_tab: VersionTab
}

pub struct VersionTab {
    pub selected: String,
    pub snapshot: bool,
    pub old: bool
}