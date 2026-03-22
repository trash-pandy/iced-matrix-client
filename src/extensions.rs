use iced::Element;
use iced::widget::Column;

pub trait ColumnExt<'a, M> {
    fn push_maybe<'b>(self, e: Option<impl Into<Element<'b, M>>>) -> Self
    where
        'b: 'a;
}

impl<'a, M> ColumnExt<'a, M> for Column<'a, M> {
    fn push_maybe<'b>(self, e: Option<impl Into<Element<'b, M>>>) -> Self
    where
        'b: 'a,
    {
        if let Some(e) = e {
            self.push(e.into())
        } else {
            self
        }
    }
}
