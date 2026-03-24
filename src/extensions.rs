use iced::Element;
use iced::widget::{Column, Stack};

pub trait PushMaybe<'a, M> {
    fn push_maybe<'b>(self, e: Option<impl Into<Element<'b, M>>>) -> Self
    where
        'b: 'a;
}

impl<'a, M> PushMaybe<'a, M> for Column<'a, M> {
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

impl<'a, M> PushMaybe<'a, M> for Stack<'a, M> {
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
