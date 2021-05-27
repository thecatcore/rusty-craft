use tui::widgets::{ListState, TableState};

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
    pub allow_oob: bool
}

impl<T> StatefulList<T> {
    pub fn new() -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items: Vec::new(),
            allow_oob: false
        }
    }

    pub fn new_inverted() -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items: Vec::new(),
            allow_oob: true
        }
    }

    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        let mut state = ListState::default();
        state.select(Some(0));
        StatefulList { state, items, allow_oob: false }
    }

    pub fn with_items_oob(items: Vec<T>) -> StatefulList<T> {
        let mut state = ListState::default();
        state.select(Some(0));
        StatefulList { state, items, allow_oob: true }
    }

    pub fn with_items_inverted(items: Vec<T>) -> StatefulList<T> {
        let mut state = ListState::default();
        state.select(Some(items.len() - 1));
        StatefulList { state, items, allow_oob: false }
    }

    pub fn with_items_inverted_oob(items: Vec<T>) -> StatefulList<T> {
        let mut state = ListState::default();
        state.select(Some(items.len() - 1));
        StatefulList { state, items, allow_oob: true }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    if self.allow_oob {
                        0
                    } else {
                        i
                    }
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i < 1 {
                    if self.allow_oob {
                        self.items.len() - 1
                    } else {
                        0
                    }
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn selected(&mut self) -> Option<&T> {
        self.items.get(self.state.selected().unwrap_or(0))
    }
}

#[derive(Clone)]
pub struct StatefulTable<T> {
    pub state: TableState,
    pub items: Vec<T>,
}

impl<T> StatefulTable<T> {
    pub fn new() -> StatefulTable<T> {
        let mut state = TableState::default();
        state.select(Some(0));
        StatefulTable {
            state,
            items: Vec::new(),
        }
    }

    pub fn with_items(items: Vec<T>) -> StatefulTable<T> {
        let mut state = TableState::default();
        state.select(Some(0));
        StatefulTable { state, items }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}
