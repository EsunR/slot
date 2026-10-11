//! A draft layout: nothing reaches the game or the card until Save is selected.
use crate::input::Remap;
use slot_input::{Action, Btn};
use slot_ui::lang;

const KEYS: [Btn; 12] = [
    Btn::Up,
    Btn::Down,
    Btn::Left,
    Btn::Right,
    Btn::A,
    Btn::B,
    Btn::X,
    Btn::Y,
    Btn::L1,
    Btn::R1,
    Btn::Start,
    Btn::Select,
];
// X has no libretro bit, so it is also the disabled target.
const TARGETS: [Btn; 11] = [
    Btn::Up,
    Btn::Down,
    Btn::Left,
    Btn::Right,
    Btn::A,
    Btn::B,
    Btn::L1,
    Btn::R1,
    Btn::Start,
    Btn::Select,
    Btn::X,
];
const PAGE: usize = 5;

pub(crate) enum Outcome {
    Stay,
    Back,
    Close,
    Save,
    Reset,
}

pub(crate) struct RemapMenu {
    pub draft: Remap,
    pub row: usize,
    pub failed: bool,
}

impl RemapMenu {
    pub fn new(draft: Remap) -> Self {
        Self {
            draft,
            row: 0,
            failed: false,
        }
    }

    pub fn apply(&mut self, action: Action) -> Outcome {
        match action {
            Action::GbaDown(Btn::Up) => self.row = (self.row + KEYS.len() + 1) % (KEYS.len() + 2),
            Action::GbaDown(Btn::Down) => self.row = (self.row + 1) % (KEYS.len() + 2),
            Action::GbaDown(Btn::Left | Btn::Right) if self.row < KEYS.len() => {
                let key = KEYS[self.row];
                let current = TARGETS
                    .iter()
                    .position(|b| *b == self.draft.target(key))
                    .unwrap_or(TARGETS.len() - 1);
                let step = if action == Action::GbaDown(Btn::Left) {
                    TARGETS.len() - 1
                } else {
                    1
                };
                self.draft
                    .set(key, TARGETS[(current + step) % TARGETS.len()]);
                self.failed = false;
            }
            Action::GbaDown(Btn::A) if self.row == KEYS.len() => return Outcome::Save,
            Action::GbaDown(Btn::A) if self.row == KEYS.len() + 1 => return Outcome::Reset,
            Action::GbaDown(Btn::B) => return Outcome::Back,
            Action::GameMenu => return Outcome::Close,
            _ => {}
        }
        Outcome::Stay
    }

    pub fn selected_face(&self) -> usize {
        1 + self.row % PAGE
    }

    pub fn lines(&self) -> Vec<String> {
        let first = self.row / PAGE * PAGE;
        let mut lines = vec![format!("{}  {}/3", lang::REMAP_ROW, self.row / PAGE + 1)];
        for row in first..(first + PAGE).min(KEYS.len() + 2) {
            lines.push(if row < KEYS.len() {
                let key = KEYS[row];
                let target = self.draft.target(key);
                let target = if TARGETS[..10].contains(&target) {
                    name(target)
                } else {
                    lang::REMAP_NONE
                };
                format!("{}  >  {}", name(key), target)
            } else if row == KEYS.len() {
                lang::REMAP_SAVE.to_string()
            } else {
                lang::REMAP_RESET.to_string()
            });
        }
        lines.push(lang::REMAP_EDIT_HINT.to_string());
        lines.push(
            if self.failed {
                lang::REMAP_SAVE_FAILED
            } else {
                lang::REMAP_BACK_HINT
            }
            .to_string(),
        );
        lines
    }
}

fn name(btn: Btn) -> &'static str {
    match btn {
        Btn::Up => "UP",
        Btn::Down => "DOWN",
        Btn::Left => "LEFT",
        Btn::Right => "RIGHT",
        Btn::A => "A",
        Btn::B => "B",
        Btn::X => "X",
        Btn::Y => "Y",
        Btn::L1 => "L1",
        Btn::R1 => "R1",
        Btn::Start => "START",
        Btn::Select => "SELECT",
        _ => "-",
    }
}
