use std::fmt::Debug;

use enum_dispatch::enum_dispatch;
use iced::{Element, Subscription, Task};

use crate::app::{Init, IntoOrPanic, ViewLike};

pub mod chat;
pub mod login;

#[derive(Debug, Clone)]
#[enum_dispatch]
pub enum Page {
    Login(login::Page),
    Chat(chat::Page),
}

#[derive(Debug, Clone)]
pub enum PageMessage {
    Login(login::Message),
    Chat(chat::Message),
}

#[enum_dispatch(Page)]
pub trait PageAdapt {
    fn adapt_subscription(&self) -> Subscription<PageMessage>;
    fn adapt_update(&mut self, message: PageMessage) -> Task<PageMessage>;
    fn adapt_view(&self) -> Element<'_, PageMessage>;
}

impl<T: ViewLike<PageMessage>> PageAdapt for T {
    fn adapt_subscription(&self) -> Subscription<PageMessage> {
        self.subscription().map(Into::into)
    }

    fn adapt_update(&mut self, message: PageMessage) -> Task<PageMessage> {
        self.update(message.into_or_panic()).map(Into::into)
    }

    fn adapt_view(&self) -> Element<'_, PageMessage> {
        self.view().map(Into::into)
    }
}

pub trait BootPageFn: Sync + Send + FnMut(Init) -> (Page, Task<PageMessage>) {
    fn clone_box(&self) -> Box<dyn BootPageFn>;
}

impl<T> BootPageFn for T
where
    T: 'static + FnMut(Init) -> (Page, Task<PageMessage>) + Clone + Sync + Send,
{
    fn clone_box(&self) -> Box<dyn BootPageFn> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn BootPageFn> {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}

impl Debug for Box<dyn BootPageFn> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NewPageFn").finish()
    }
}
