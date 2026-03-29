use std::fmt::Debug;

use enum_dispatch::enum_dispatch;
use iced::{Element, Subscription, Task};

use crate::app::{AppMessenger, IntoOrPanic, ViewLike};
use crate::name_of_trait;

pub mod settings;
pub mod verification;

#[derive(Debug, Clone)]
#[enum_dispatch(ModalAdapt)]
pub enum Modal {
    Settings(settings::Modal),
    Verification(verification::Modal),
}

#[derive(Debug, Clone)]
pub enum ModalMessage {
    Settings(settings::Message),
    Verification(verification::Message),
}

#[enum_dispatch]
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

pub trait BootModalFn: Sync + Send + FnMut(AppMessenger) -> (Modal, Task<ModalMessage>) {
    fn clone_box(&self) -> Box<dyn BootModalFn>;
}

impl<T> BootModalFn for T
where
    T: 'static + FnMut(AppMessenger) -> (Modal, Task<ModalMessage>) + Clone + Sync + Send,
{
    fn clone_box(&self) -> Box<dyn BootModalFn> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn BootModalFn> {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}

impl Debug for Box<dyn BootModalFn> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(name_of_trait!(BootModalFn))
    }
}
