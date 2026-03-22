use iced::widget::{column, text};
use iced::{Element, Font};

pub const FONT_BOLD: Font = Font {
    family: iced::font::Family::Name("Ubuntu Sans"),
    weight: iced::font::Weight::Bold,
    stretch: iced::font::Stretch::SemiExpanded,
    style: iced::font::Style::Normal,
};

pub fn labelled<'a, 'ti: 'a, 't: 'a, M: 'a>(
    label: impl text::IntoFragment<'t>,
    elem: impl Into<Element<'ti, M>>,
) -> Element<'a, M> {
    column![text(label).size(17), elem.into(),]
        .spacing(2.0)
        .into()
}
