use std::sync::RwLock;

use iced::widget::{column, text};
use iced::{Color, Element, Font, Pixels, Shadow, Theme, Vector};

static APP_THEME: RwLock<Theme> = RwLock::new(Theme::TokyoNightStorm);

pub fn get_app_theme() -> Theme {
    APP_THEME.read().unwrap().clone()
}

pub fn set_app_theme(theme: Theme) {
    *APP_THEME.write().unwrap() = theme;
}

pub const FONT_BOLD: Font = Font {
    family: iced::font::Family::Name("Ubuntu Sans"),
    weight: iced::font::Weight::Bold,
    stretch: iced::font::Stretch::SemiExpanded,
    style: iced::font::Style::Normal,
};

pub const FONT_MEDIUM: Font = Font {
    family: iced::font::Family::Name("Ubuntu Sans"),
    weight: iced::font::Weight::Medium,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

pub const SPACING_LARGE: Pixels = Pixels(8.0);
pub const SPACING_MEDIUM: Pixels = Pixels(4.0);
pub const SPACING_SMALL: Pixels = Pixels(2.0);

pub const TEXT_SMALL: Pixels = Pixels(14.0);
pub const TEXT_MED: Pixels = Pixels(17.0);
pub const TEXT_LARGE: Pixels = Pixels(21.0);

pub const MODAL_SHADOW: Shadow = Shadow {
    color: Color::from_rgba(0.0, 0.0, 0.0, 0.6),
    offset: Vector::new(0.0, 6.0),
    blur_radius: 24.0,
};

pub fn labelled<'a, 'ti: 'a, 't: 'a, M: 'a>(
    label: impl text::IntoFragment<'t>,
    elem: impl Into<Element<'ti, M>>,
) -> Element<'a, M> {
    column![text(label).size(TEXT_MED), elem.into(),]
        .spacing(SPACING_SMALL)
        .into()
}
