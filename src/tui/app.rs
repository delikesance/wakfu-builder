use crate::optimizer::{Role, Mode, Range, Element};
use crate::models::Equipment;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(PartialEq)]
pub enum AppState {
    Setup,
    Optimizing,
    Results,
}

pub struct App {
    pub state: AppState,
    pub level: i32,
    pub role: Role,
    pub mode: Mode,
    pub range: Range,
    pub element: Element,
    pub selected_index: usize,
    pub items: Vec<Equipment>,
    pub best_build: Vec<Equipment>,
    pub optimize_handle: Option<Arc<Mutex<Option<Vec<Equipment>>>>>,
    pub optimize_stats_handle: Option<Arc<Mutex<Option<HashMap<i32, f32>>>>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Setup,
            level: 200,
            role: Role::DPS,
            mode: Mode::Solo,
            range: Range::Hybrid,
            element: Element::All,
            selected_index: 0,
            items: Vec::new(),
            best_build: Vec::new(),
            optimize_handle: None,
            optimize_stats_handle: None,
        }
    }

    pub fn next_setting(&mut self) {
        self.selected_index = (self.selected_index + 1) % 5;
    }

    pub fn prev_setting(&mut self) {
        if self.selected_index == 0 {
            self.selected_index = 4;
        } else {
            self.selected_index -= 1;
        }
    }

    pub fn adjust_setting(&mut self, delta: i32) {
        match self.selected_index {
            0 => {
                self.level = (self.level + delta).clamp(1, 245);
            }
            1 => {
                self.role = match self.role {
                    Role::DPS => if delta > 0 { Role::Tank } else { Role::Support },
                    Role::Tank => if delta > 0 { Role::Support } else { Role::DPS },
                    Role::Support => if delta > 0 { Role::DPS } else { Role::Tank },
                };
            }
            2 => {
                self.mode = match self.mode {
                    Mode::Solo => Mode::Team,
                    Mode::Team => Mode::Solo,
                };
            }
            3 => {
                self.range = match self.range {
                    Range::Melee => if delta > 0 { Range::Distance } else { Range::Hybrid },
                    Range::Distance => if delta > 0 { Range::Hybrid } else { Range::Melee },
                    Range::Hybrid => if delta > 0 { Range::Melee } else { Range::Distance },
                };
            }
            4 => {
                self.element = match self.element {
                    Element::Fire => if delta > 0 { Element::Earth } else { Element::All },
                    Element::Earth => if delta > 0 { Element::Water } else { Element::Fire },
                    Element::Water => if delta > 0 { Element::Air } else { Element::Earth },
                    Element::Air => if delta > 0 { Element::All } else { Element::Water },
                    Element::All => if delta > 0 { Element::Fire } else { Element::Air },
                };
            }
            _ => {}
        }
    }
}
