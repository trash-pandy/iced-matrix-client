use enum_dispatch::enum_dispatch;
use iced::{Element, Subscription, Task};

use crate::app::{IntoOrPanic, ViewLike};

pub mod settings;

#[derive(Debug)]
#[enum_dispatch]
pub enum Modal {
    Settings(settings::Modal),
}

#[derive(Debug, Clone)]
pub enum ModalMessage {
    Settings(settings::Message),
}

#[enum_dispatch(Modal)]
pub trait ModalAdapt {
    fn adapt_subscription(&self) -> Subscription<ModalMessage>;
    fn adapt_update(&mut self, message: ModalMessage) -> Task<ModalMessage>;
    fn adapt_view(&self) -> Element<'_, ModalMessage>;
}

impl<T: ViewLike<ModalMessage>> ModalAdapt for T {
    fn adapt_subscription(&self) -> Subscription<ModalMessage> {
        self.subscription().map(Into::into)
    }

    fn adapt_update(&mut self, message: ModalMessage) -> Task<ModalMessage> {
        self.update(message.into_or_panic()).map(Into::into)
    }

    fn adapt_view(&self) -> Element<'_, ModalMessage> {
        self.view().map(Into::into)
    }
}
