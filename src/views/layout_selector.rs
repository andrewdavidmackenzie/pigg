use crate::Message;
use iced::widget::Button;
use iced::{Color, Length, Size};

use crate::views::hardware_view::HardwareTarget;
use crate::views::hardware_view::HardwareTarget::NoHW;
use crate::views::info_row::{MENU_BAR_BUTTON_STYLE, MENU_BUTTON_STYLE};
use crate::views::layout_selector::Layout::{BCMLayout, BoardLayout};
use iced::{Background, Element, Renderer, Theme};
use iced_aw::menu;
use iced_aw::menu::StyleSheet;
use iced_aw::menu::{Item, Menu, MenuBar};
use iced_aw::style::MenuBarStyle;

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
            Layout::BoardLayout => BOARD_LAYOUT_SIZE,
            Layout::BCMLayout => BCM_LAYOUT_SIZE,
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

        let button = if hardware_target != &NoHW {
            match self.selected_layout {
                Layout::BoardLayout => {
                    let show_bcp_layout: Item<'a, Message, _, _> = Item::new(
                        Button::new("BCP Pin Layout")
                            .width(Length::Fill)
                            .on_press(Message::LayoutChanged(BCMLayout))
                            .style(MENU_BUTTON_STYLE.get_button_style()),
                    );
                    menu_items.push(show_bcp_layout);
                    Button::new("layout: board")
                }
                Layout::BCMLayout => {
                    let show_physical_layout: Item<'a, Message, _, _> = Item::new(
                        Button::new("Board Pin Layout")
                            .width(Length::Fill)
                            .on_press(Message::LayoutChanged(BoardLayout))
                            .style(MENU_BUTTON_STYLE.get_button_style()),
                    );
                    menu_items.push(show_physical_layout);

                    Button::new("layout: bcp")
                }
            }
            .style(MENU_BAR_BUTTON_STYLE.get_button_style())
            .on_press(Message::MenuBarButtonClicked)
        } else {
            Button::new("layout").style(MENU_BAR_BUTTON_STYLE.get_button_style())
        };

        let menu_root = Item::with_menu(button, Menu::new(menu_items).width(135.0).offset(10.0));

        MenuBar::new(vec![menu_root])
            .style(|theme: &iced::Theme| menu::Appearance {
                bar_background: Background::Color(Color::TRANSPARENT),
                menu_shadow: iced::Shadow {
                    color: Color::BLACK,
                    offset: iced::Vector::new(1.0, 1.0),
                    blur_radius: 10f32,
                },
                menu_background_expand: iced::Padding::from([5, 5]),
                ..theme.appearance(&MenuBarStyle::Default)
            })
            .into()
    }
}
