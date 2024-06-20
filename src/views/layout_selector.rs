use crate::Message;
use iced::widget::pick_list;
use iced::{Element, Length, Size};

/// These are the possible layouts to chose from
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Layout {
    #[default]
    BoardLayout,
    BCMLayout,
}

// Implementing Display for Layout
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

// TODO use these later, together with config column width to calculate the required window width
// const BOARD_WINDOW_WIDTH: f32 = BOARD_PIN_LAYOUT_WIDTH;
const BOARD_LAYOUT_WIDTH: f32 = 1570.0;
// const BCM_WINDOW_WIDTH: f32 = BCM_PIN_LAYOUT_WIDTH;
const BOARD_LAYOUT_HEIGHT: f32 = 780.0;
const BCM_LAYOUT_WIDTH: f32 = 860.0;
const BCM_LAYOUT_HEIGHT: f32 = 976.0;
const LAYOUTS: [Layout; 2] = [Layout::BoardLayout, Layout::BCMLayout];

#[derive(Clone, PartialEq, Default)]
pub struct LayoutSelector {
    selected_layout: Layout,
}

impl LayoutSelector {
    pub fn new() -> Self {
        LayoutSelector {
            selected_layout: Layout::default(),
        }
    }

    pub fn get_default_window_size() -> Size {
        Size {
            width: BOARD_LAYOUT_WIDTH,
            height: BOARD_LAYOUT_HEIGHT,
        }
    }

    pub fn get_window_size(&self) -> Size {
        match self.selected_layout {
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

    /// Set the new layout as being selected and return the window size required
    pub fn update(&mut self, new_layout: Layout) -> Size {
        self.selected_layout = new_layout;
        self.get_window_size()
    }

    // Return the currently selected layout
    pub fn get(&self) -> Layout {
        self.selected_layout
    }

    /// Generate the view to represent the [LayoutSelector]
    pub fn view(&self) -> Element<'static, Message> {
        pick_list(
            &LAYOUTS[..],
            Some(self.selected_layout),
            Message::LayoutChanged,
        )
        .width(Length::Shrink)
        .placeholder("Choose Layout")
        .into()
    }
}
