use crate::Message;
use iced::widget::Button;
use iced::{Length, Size};

use crate::views::hardware_view::HardwareTarget;
use crate::views::hardware_view::HardwareTarget::NoHW;
use crate::views::info_row::{
    MENU_BAR_BUTTON_HOVER_STYLE, MENU_BAR_BUTTON_STYLE, MENU_BAR_STYLE, MENU_BUTTON_HOVER_STYLE,
    MENU_BUTTON_STYLE,
};
use crate::views::layout_menu::Layout::{BCMLayout, BoardLayout};
use iced::widget::button::Status::Hovered;
use iced::{Element, Renderer, Theme};
use iced_aw::menu::{Item, Menu, MenuBar};

/// These are the possible layouts to chose from
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Layout {
    #[default]
    BoardLayout,
    BCMLayout,
}

const BOARD_LAYOUT_SIZE: Size = Size {
    width: 1400.0,
    height: 720.0,
};

const BCM_LAYOUT_SIZE: Size = Size {
    width: 700.0,
    height: 916.0,
};

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
            BoardLayout => BOARD_LAYOUT_SIZE,
            BCMLayout => BCM_LAYOUT_SIZE,
        }
    }

    // Return the currently selected layout
    pub fn get(&self) -> Layout {
        self.selected_layout
    }

    /// Create the view that shows menu to change layout
    pub fn view<'a>(
        &self,
        hardware_target: &'a HardwareTarget,
    ) -> Element<'a, Message, Theme, Renderer> {
        let mut menu_items: Vec<Item<'a, Message, _, _>> = vec![];

        let button = match self.selected_layout {
            BoardLayout => {
                let mut show_bcp_layout =
                    Button::new("BCP Pin Layout")
                        .width(Length::Fill)
                        .style(|_, status| {
                            if status == Hovered {
                                MENU_BUTTON_HOVER_STYLE
                            } else {
                                MENU_BUTTON_STYLE
                            }
                        });

                if hardware_target != &NoHW {
                    show_bcp_layout = show_bcp_layout.on_press(Message::LayoutChanged(BCMLayout));
                }
                menu_items.push(Item::new(show_bcp_layout));
                Button::new("layout: board")
            }
            BCMLayout => {
                let mut show_physical_layout = Button::new("Board Pin Layout")
                    .width(Length::Fill)
                    .style(|_, status| {
                        if status == Hovered {
                            MENU_BUTTON_HOVER_STYLE
                        } else {
                            MENU_BUTTON_STYLE
                        }
                    });

                if hardware_target != &NoHW {
                    show_physical_layout =
                        show_physical_layout.on_press(Message::LayoutChanged(BoardLayout));
                }

                menu_items.push(Item::new(show_physical_layout));
                Button::new("layout: bcp")
            }
        };

        let button = button
            .style(move |_theme, status| {
                if status == Hovered {
                    MENU_BAR_BUTTON_HOVER_STYLE
                } else {
                    MENU_BAR_BUTTON_STYLE
                }
            })
            .on_press(Message::MenuBarButtonClicked); // Needed for highlighting;

        let menu_root = Item::with_menu(button, Menu::new(menu_items).width(135.0).offset(10.0));

        MenuBar::new(vec![menu_root])
            .style(|_, _| MENU_BAR_STYLE)
            .into()
    }
}
