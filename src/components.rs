use iced::Element;
use iced::widget::{column, text};

pub fn labelled<'a, 'ti: 'a, 't: 'a, M: 'a>(
    label: impl text::IntoFragment<'t>,
    elem: impl Into<Element<'ti, M>>,
) -> Element<'a, M> {
    column![text(label).size(17), elem.into(),]
        .spacing(2.0)
        .into()
}
