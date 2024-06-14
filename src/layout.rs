use crate::pin_layout::{BCM_PIN_LAYOUT_WIDTH, BOARD_PIN_LAYOUT_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    BoardLayout,
    BCMLayout,
}

const BOARD_LAYOUT_SPACING: u16 = 470;
const BCM_LAYOUT_SPACING: u16 = 640;
const BOARD_WINDOW_WIDTH: f32 = BOARD_PIN_LAYOUT_WIDTH;
const BCM_WINDOW_WIDTH: f32 = BCM_PIN_LAYOUT_WIDTH;
pub const BOARD_LAYOUT_SIZE: (f32, f32) = (1570.0, 780.0);
pub const BCM_LAYOUT_SIZE: (f32, f32) = (1000.0, 950.0);

impl Layout {
    pub const ALL: [Layout; 2] = [Layout::BoardLayout, Layout::BCMLayout];

    pub fn get_spacing(&self) -> u16 {
        match self {
            Layout::BoardLayout => BOARD_LAYOUT_SPACING,
            Layout::BCMLayout => BCM_LAYOUT_SPACING,
        }
    }
}

// Implementing format for Layout
// TODO could maybe put the Name as a &str inside the enum elements above?
impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Layout::BoardLayout => "Board Pin Layout",
                Layout::BCMLayout => "BCM Pin Layout",
            }
        )
    }
}
