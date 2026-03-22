use std::fmt::Debug;

use futures_util::SinkExt;
use iced::event::Event;
use iced::futures::channel::mpsc;
use iced::keyboard::{self, key};
use iced::widget::operation;
use iced::{Element, Subscription, Task, stream};
use tokio::sync::broadcast;

use crate::modal::{Modal, ModalAdapt};
use crate::page::{Page, PageAdapt, PageMessage};
use crate::util::Smuggle;
use crate::{modal, page};

pub struct State {
    page: Page,
    modal: Option<Modal>,
    app_sink: broadcast::Sender<AppMessage>,
    app_stream: broadcast::Receiver<AppMessage>,
}

impl State {
    fn boot() -> (Self, Task<AppMessage>) {
        let (app_sink, app_stream) = broadcast::channel(64);
        let (page, task) = page::login::Page::new(Init {
            app_sink: app_sink.clone(),
        });
        (
            Self {
                page: page.into(),
                modal: None,
                app_sink,
                app_stream,
            },
            task.map(PageMessage::from).map(Into::into),
        )
    }
}

#[derive(Debug, Clone)]
pub struct Init {
    pub app_sink: broadcast::Sender<AppMessage>,
}

impl Init {
    pub fn from_state(state: &State) -> Self {
        Self {
            app_sink: state.app_sink.clone(),
        }
    }
}

pub fn boot() -> (State, Task<AppMessage>) {
    State::boot()
}

pub fn subscription(state: &State) -> Subscription<AppMessage> {
    Subscription::batch([
        iced::event::listen().map(AppMessage::Event),
        Subscription::run_with(
            Smuggle::new("app-message-sub", state.app_stream.resubscribe()),
            |stream| {
                let mut stream = stream.take();
                stream::channel(64, |mut s: mpsc::Sender<AppMessage>| async move {
                    while let Ok(msg) = stream.recv().await {
                        s.send(msg).await.ok();
                    }
                })
            },
        ),
        state.page.adapt_subscription().map(Into::into),
        state
            .modal
            .as_ref()
            .map_or_else(Subscription::none, ModalAdapt::adapt_subscription)
            .map(Into::into),
    ])
}

pub fn update(state: &mut State, message: AppMessage) -> iced::Task<AppMessage> {
    match message {
        AppMessage::PageMessage(message) => state.page.adapt_update(message).map(Into::into),
        AppMessage::ModalMessage(message) => state
            .modal
            .as_mut()
            .map_or_else(Task::none, |modal| modal.adapt_update(message))
            .map(Into::into),
        AppMessage::SwitchPage(mut page) => {
            let (page, task) = page(Init::from_state(state));
            state.page = page;
            task.map(Into::into)
        }
        AppMessage::Event(event) => match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Tab),
                modifiers,
                ..
            }) => {
                if modifiers.shift() {
                    operation::focus_previous()
                } else {
                    operation::focus_next()
                }
            }
            _ => Task::none(),
        },
    }
}

pub fn view(state: &State) -> Element<'_, AppMessage> {
    state.page.adapt_view().map(Into::into)
}

pub trait ViewLike<M> {
    type Message: 'static + Into<M> + FromOrPanic<M> + Send + Debug;

    fn subscription(&self) -> Subscription<Self::Message>;
    fn update(&mut self, message: Self::Message) -> Task<Self::Message>;
    fn view(&self) -> Element<'_, Self::Message>;
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    PageMessage(page::PageMessage),
    ModalMessage(modal::ModalMessage),
    Event(Event),
    SwitchPage(Box<dyn page::BootPageFn>),
}

impl From<page::PageMessage> for AppMessage {
    fn from(value: page::PageMessage) -> Self {
        Self::PageMessage(value)
    }
}

impl From<modal::ModalMessage> for AppMessage {
    fn from(value: modal::ModalMessage) -> Self {
        Self::ModalMessage(value)
    }
}

pub trait IntoOrPanic<T> {
    fn into_or_panic(self) -> T;
}

pub trait FromOrPanic<T> {
    fn from_or_panic(value: T) -> Self;
}

impl<A, B: FromOrPanic<A>> IntoOrPanic<B> for A {
    fn into_or_panic(self) -> B {
        B::from_or_panic(self)
    }
}
