use crate::Message;
use iced::widget::Button;
use iced::{Length, Size};

use crate::hw_definition::config::HardwareConfig;
use crate::views::hardware_styles::SPACE_BETWEEN_PIN_ROWS;
use crate::views::hardware_view::HardwareConnection;
use crate::views::hardware_view::HardwareConnection::NoConnection;
use crate::views::info_row::{menu_bar_button, menu_button};
use crate::views::layout_menu::Layout::{Board, Logical, Reduced};
use iced::{Renderer, Theme};
use iced_aw::menu::{Item, Menu};

/// These are the possible layouts to chose from
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Layout {
    #[default]
    Board,
    Logical,
    Reduced,
}

const BOARD_LAYOUT_SIZE: Size = Size {
    width: 1400.0,
    height: 720.0,
};

const BCM_LAYOUT_SIZE: Size = Size {
    width: 720.0,
    height: 910.0,
};

// calculate the height required based on the number of configured pins
fn reduced_layout_size(hardware_config: &HardwareConfig) -> Size {
    Size {
        width: 720.0,
        height: 28.0 + 28.0 /* InfoRow Height */
            + (hardware_config.pin_functions.len() as f32 * (28.0 + SPACE_BETWEEN_PIN_ROWS)),
    }
}

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
    pub fn update(&mut self, new_layout: Layout, hardware_config: &HardwareConfig) -> Size {
        self.selected_layout = new_layout;
        match self.selected_layout {
            Board => BOARD_LAYOUT_SIZE,
            Logical => BCM_LAYOUT_SIZE,
            Layout::Reduced => reduced_layout_size(hardware_config),
        }
    }

    // Return the currently selected layout
    pub fn get(&self) -> Layout {
        self.selected_layout
    }

    /// Create the view that shows menu to change layout
    pub fn view<'a>(
        &self,
        hardware_connection: &'a HardwareConnection,
    ) -> Item<'a, Message, Theme, Renderer> {
        let mut menu_items: Vec<Item<'a, Message, _, _>> = vec![];

        let mut show_bcp_layout = Button::new("BCP Pin Layout")
            .width(Length::Fill)
            .style(menu_button);
        if hardware_connection != &NoConnection && self.selected_layout != Logical {
            show_bcp_layout = show_bcp_layout.on_press(Message::LayoutChanged(Logical));
        }
        menu_items.push(Item::new(show_bcp_layout));

        let mut show_physical_layout = Button::new("Board Pin Layout")
            .width(Length::Fill)
            .style(menu_button);
        if hardware_connection != &NoConnection && self.selected_layout != Board {
            show_physical_layout = show_physical_layout.on_press(Message::LayoutChanged(Board));
        }
        menu_items.push(Item::new(show_physical_layout));

        let mut show_reduced_layout = Button::new("Reduced Layout")
            .width(Length::Fill)
            .style(menu_button);

        if hardware_connection != &NoConnection && self.selected_layout != Reduced {
            show_reduced_layout = show_reduced_layout.on_press(Message::LayoutChanged(Reduced));
        }

        menu_items.push(Item::new(show_reduced_layout));

        let button = match self.selected_layout {
            Board => Button::new("layout: board"),
            Logical => Button::new("layout: bcp"),
            Layout::Reduced => Button::new("layout: reduced"),
        }
        .style(menu_bar_button)
        .on_press(Message::MenuBarButtonClicked); // Needed for highlighting;

        Item::with_menu(button, Menu::new(menu_items).width(135.0).offset(10.0))
    }
}
