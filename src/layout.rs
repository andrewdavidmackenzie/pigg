#![allow(unused)] // Need to remove

use iced::Size;

use crate::pin_layout::{BCM_PIN_LAYOUT_WIDTH, BOARD_PIN_LAYOUT_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    BoardLayout,
    BCMLayout,
}

const BOARD_LAYOUT_SPACING: u16 = 470;
const BCM_LAYOUT_SPACING: u16 = 640;
// TODO use these later, together with config column width to calculate the required window width
const BOARD_WINDOW_WIDTH: f32 = BOARD_PIN_LAYOUT_WIDTH;
const BOARD_LAYOUT_WIDTH: f32 = 1570.0;
const BCM_WINDOW_WIDTH: f32 = BCM_PIN_LAYOUT_WIDTH;
const BOARD_LAYOUT_HEIGHT: f32 = 780.0;
const BCM_LAYOUT_WIDTH: f32 = 1000.0;
const BCM_LAYOUT_HEIGHT: f32 = 950.0;

impl Layout {
    pub const ALL: [Layout; 2] = [Layout::BoardLayout, Layout::BCMLayout];

    pub fn get_spacing(&self) -> u16 {
        match self {
            Layout::BoardLayout => BOARD_LAYOUT_SPACING,
            Layout::BCMLayout => BCM_LAYOUT_SPACING,
        }
    }

    pub fn get_window_size(&self) -> Size {
        match self {
            Layout::BoardLayout => Size {
                width: BOARD_LAYOUT_WIDTH,
                height: BOARD_LAYOUT_HEIGHT,
            },
            Layout::BCMLayout => Size {
                width: BCM_LAYOUT_WIDTH,
                height: BCM_LAYOUT_HEIGHT,
            },
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
