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

const LAYOUTS: [Layout; 2] = [Layout::BoardLayout, Layout::BCMLayout];

const BOARD_LAYOUT_SIZE: Size = Size {
    width: 1570.0,
    height: 780.0,
};

const BCM_LAYOUT_SIZE: Size = Size {
    width: 860.0,
    height: 976.0,
};

const TEXT_WIDTH: u16 = 18;
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

    pub const fn get_default_window_size() -> Size {
        BOARD_LAYOUT_SIZE
    }

    /// Set the new layout as being selected and return the window size required
    pub fn update(&mut self, new_layout: Layout) -> Size {
        self.selected_layout = new_layout;
        match self.selected_layout {
            Layout::BoardLayout => BOARD_LAYOUT_SIZE,
            Layout::BCMLayout => BCM_LAYOUT_SIZE,
        }
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
        .text_size(TEXT_WIDTH)
        .placeholder("Choose Layout")
        .into()
    }
}

#[cfg(test)]
mod test {
    use crate::views::layout_selector::{
        Layout, LayoutSelector, BCM_LAYOUT_SIZE, BOARD_LAYOUT_SIZE,
    };

    #[test]
    fn default_is_board() {
        assert_eq!(LayoutSelector::get_default_window_size(), BOARD_LAYOUT_SIZE);
    }

    #[test]
    fn initial() {
        let mut layout_selector = LayoutSelector::new();
        assert_eq!(
            layout_selector.update(layout_selector.get()),
            BOARD_LAYOUT_SIZE
        );
    }

    #[test]
    fn switch_to_bcm() {
        let mut layout_selector = LayoutSelector::new();
        assert_eq!(layout_selector.update(Layout::BCMLayout), BCM_LAYOUT_SIZE);
        assert_eq!(layout_selector.get(), Layout::BCMLayout);
    }
}
