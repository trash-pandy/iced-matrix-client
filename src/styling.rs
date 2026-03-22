use iced::widget::{column, text};
use iced::{Element, Font, Pixels, Theme};

pub const APP_THEME: Theme = Theme::TokyoNightStorm;

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

pub fn labelled<'a, 'ti: 'a, 't: 'a, M: 'a>(
    label: impl text::IntoFragment<'t>,
    elem: impl Into<Element<'ti, M>>,
) -> Element<'a, M> {
    column![text(label).size(TEXT_MED), elem.into(),]
        .spacing(SPACING_SMALL)
        .into()
}
