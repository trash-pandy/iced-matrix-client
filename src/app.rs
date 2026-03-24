use std::fmt::Debug;

use futures_util::SinkExt;
use iced::Length::Shrink;
use iced::event::Event;
use iced::futures::channel::mpsc;
use iced::keyboard::{self, key};
use iced::widget::{Container, Stack, operation};
use iced::{Element, Executor, Font, Program, Renderer, Subscription, Task, Theme, stream, window};
use tokio::sync::broadcast;

use crate::extensions::PushMaybe;
use crate::modal::{Modal, ModalAdapt, ModalMessage};
use crate::page::{Page, PageAdapt, PageMessage};
use crate::util::Smuggle;
use crate::{modal, page, styling};

#[derive(Debug, Clone)]
pub enum AppMessage {
    PageMessage(page::PageMessage),
    ModalMessage(modal::ModalMessage),
    Event(Event),
    SwitchPage(Box<dyn page::BootPageFn>),
    OpenModal(Box<dyn modal::BootModalFn>),
    CloseModal,
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

pub struct State {
    page: Page,
    modal: Option<Modal>,
    app_sink: broadcast::Sender<AppMessage>,
    app_stream: broadcast::Receiver<AppMessage>,
}

impl State {
    pub fn boot() -> (Self, Task<AppMessage>) {
        let (app_sink, app_stream) = broadcast::channel(64);
        let (page, task) = page::login::Page::new(AppMessenger {
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
pub struct AppMessenger {
    app_sink: broadcast::Sender<AppMessage>,
}

impl AppMessenger {
    pub fn from_state(state: &State) -> Self {
        Self {
            app_sink: state.app_sink.clone(),
        }
    }

    pub fn switch_page<
        IntoPage: Into<Page>,
        IntoMessage: 'static + Into<PageMessage> + Sync + Send,
        IntoBootPageFn: 'static + FnMut(Self) -> (IntoPage, Task<IntoMessage>) + Sync + Send + Clone,
    >(
        &self,
        mut page: IntoBootPageFn,
    ) {
        self.app_sink
            .send(AppMessage::SwitchPage(Box::new(move |init| {
                let (p, t) = page(init);
                (p.into(), t.map(Into::into))
            })))
            .ok();
    }

    pub fn open_modal<
        P: Into<Modal>,
        M: 'static + Into<ModalMessage> + Sync + Send,
        F: 'static + FnMut(Self) -> (P, Task<M>) + Sync + Send + Clone,
    >(
        &self,
        mut modal: F,
    ) {
        self.app_sink
            .send(AppMessage::OpenModal(Box::new(move |init| {
                let (m, t) = modal(init);
                (m.into(), t.map(Into::into))
            })))
            .ok();
    }

    pub fn close_modal(&self) {
        self.app_sink.send(AppMessage::CloseModal).ok();
    }
}

pub struct IcedMatrixClient;
impl Program for IcedMatrixClient {
    type Executor = TokioHandleExecutor;
    type Message = AppMessage;
    type Renderer = Renderer;
    type State = State;
    type Theme = Theme;

    fn theme(&self, _state: &Self::State, _window: window::Id) -> Option<Self::Theme> {
        Some(styling::get_app_theme())
    }

    fn name() -> &'static str {
        "iced-matrix-client"
    }

    fn title(&self, _state: &Self::State, _window: window::Id) -> String {
        Self::name().to_string()
    }

    fn settings(&self) -> iced::Settings {
        iced::Settings {
            id: Some("iced-matrix-client".to_owned()),
            fonts: vec![
                include_bytes!("../assets/UbuntuSans/BoldItalic.ttf").into(),
                include_bytes!("../assets/UbuntuSans/Bold.ttf").into(),
                include_bytes!("../assets/UbuntuSans/ExtraBoldItalic.ttf").into(),
                include_bytes!("../assets/UbuntuSans/ExtraBold.ttf").into(),
                include_bytes!("../assets/UbuntuSans/ExtraLightItalic.ttf").into(),
                include_bytes!("../assets/UbuntuSans/ExtraLight.ttf").into(),
                include_bytes!("../assets/UbuntuSans/Italic.ttf").into(),
                include_bytes!("../assets/UbuntuSans/LightItalic.ttf").into(),
                include_bytes!("../assets/UbuntuSans/Light.ttf").into(),
                include_bytes!("../assets/UbuntuSans/MediumItalic.ttf").into(),
                include_bytes!("../assets/UbuntuSans/Medium.ttf").into(),
                include_bytes!("../assets/UbuntuSans/Regular.ttf").into(),
                include_bytes!("../assets/UbuntuSans/SemiBoldItalic.ttf").into(),
                include_bytes!("../assets/UbuntuSans/SemiBold.ttf").into(),
                include_bytes!("../assets/UbuntuSans/ThinItalic.ttf").into(),
                include_bytes!("../assets/UbuntuSans/Thin.ttf").into(),
            ],
            default_font: Font::with_name("Ubuntu Sans"),
            ..Default::default()
        }
    }

    fn window(&self) -> Option<window::Settings> {
        Some(window::Settings::default())
    }

    fn boot(&self) -> (Self::State, iced::Task<Self::Message>) {
        State::boot()
    }

    fn subscription(&self, state: &Self::State) -> iced::Subscription<Self::Message> {
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

    fn update(&self, state: &mut Self::State, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            AppMessage::PageMessage(message) => state.page.adapt_update(message).map(Into::into),
            AppMessage::ModalMessage(message) => state
                .modal
                .as_mut()
                .map_or_else(Task::none, |modal| modal.adapt_update(message))
                .map(Into::into),
            AppMessage::SwitchPage(mut page) => {
                let (page, task) = page(AppMessenger::from_state(state));
                state.page = page;
                task.map(Into::into)
            }
            AppMessage::OpenModal(mut modal) => {
                let (modal, task) = modal(AppMessenger::from_state(state));
                state.modal = Some(modal);
                task.map(Into::into)
            }
            AppMessage::CloseModal => {
                state.modal = None;
                Task::none()
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

    fn view<'a>(
        &self,
        state: &'a Self::State,
        _window: window::Id,
    ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer> {
        Stack::new()
            .push(state.page.adapt_view().map(Into::into))
            .push_maybe(
                state
                    .modal
                    .as_ref()
                    .map(|modal| Container::new(modal.adapt_view().map(Into::into)).center(Shrink)),
            )
            .into()
    }
}

pub struct TokioHandleExecutor(tokio::runtime::Handle);

impl Executor for TokioHandleExecutor {
    fn new() -> Result<Self, iced::futures::io::Error>
    where
        Self: Sized,
    {
        Ok(Self(tokio::runtime::Handle::current()))
    }

    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        self.0.spawn(future);
    }

    fn block_on<T>(&self, future: impl Future<Output = T>) -> T {
        self.0.block_on(future)
    }
}

pub trait ViewLike<M> {
    type Message: 'static + Into<M> + FromOrPanic<M> + Send + Debug;

    fn subscription(&self) -> Subscription<Self::Message>;
    fn update(&mut self, message: Self::Message) -> Task<Self::Message>;
    fn view(&self) -> Element<'_, Self::Message>;
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
