use crate::Message;
use iced::widget::Button;
use iced::{Length, Size};

use crate::hw_definition::config::HardwareConfig;
use crate::hw_definition::description::HardwareDescription;
use crate::views::hardware_view::HardwareConnection::NoConnection;
use crate::views::hardware_view::{
    bcm_layout_size, board_layout_size, compact_layout_size, HardwareConnection,
};
use crate::views::info_row::{menu_bar_button, menu_button_style};
use crate::views::layout_menu::Layout::{Board, Compact, Logical};
use iced::{Renderer, Theme};
use iced_aw::menu::{Item, Menu};

/// These are the possible layouts to chose from
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Layout {
    #[default]
    Board,
    Logical,
    Compact,
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
        board_layout_size(40)
    }

    /// Set the new layout as being selected and return the window size required
    pub fn update(&mut self, new_layout: Layout) {
        self.selected_layout = new_layout;
    }

    /// Return what is the window size request for the currently selected layout
    pub fn window_size_requested(
        &self,
        hardware_description: &Option<HardwareDescription>,
        hardware_config: &HardwareConfig,
    ) -> Size {
        match self.selected_layout {
            Board => board_layout_size(
                hardware_description
                    .as_ref()
                    .map(|hw| hw.pins.pins().len())
                    .unwrap_or(40),
            ),
            Logical => bcm_layout_size({
                hardware_description
                    .as_ref()
                    .map(|hw| hw.pins.pins().iter().filter(|p| p.bcm.is_some()).count())
                    .unwrap_or(26)
            }),
            Compact => compact_layout_size(hardware_config.pin_functions.len()),
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
            .style(menu_button_style);
        if hardware_connection != &NoConnection && self.selected_layout != Logical {
            show_bcp_layout = show_bcp_layout.on_press(Message::LayoutChanged(Logical));
        }
        menu_items.push(Item::new(show_bcp_layout));

        let mut show_physical_layout = Button::new("Board Pin Layout")
            .width(Length::Fill)
            .style(menu_button_style);
        if hardware_connection != &NoConnection && self.selected_layout != Board {
            show_physical_layout = show_physical_layout.on_press(Message::LayoutChanged(Board));
        }
        menu_items.push(Item::new(show_physical_layout));

        let mut show_reduced_layout = Button::new("Compact Layout")
            .width(Length::Fill)
            .style(menu_button_style);

        if hardware_connection != &NoConnection && self.selected_layout != Compact {
            show_reduced_layout = show_reduced_layout.on_press(Message::LayoutChanged(Compact));
        }

        menu_items.push(Item::new(show_reduced_layout));

        let button = match self.selected_layout {
            Board => Button::new("layout: board"),
            Logical => Button::new("layout: bcp"),
            Compact => Button::new("layout: compact"),
        }
        .style(menu_bar_button)
        .on_press(Message::MenuBarButtonClicked); // Needed for highlighting;

        Item::with_menu(button, Menu::new(menu_items).width(135.0))
    }
}
