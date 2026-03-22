mod app;
mod extensions;
mod macros;
mod modal;
mod page;
mod styling;
mod tasks;
mod util;
mod worker;

use iced::{Element, Executor, Font, Program, Renderer, Theme, window};

use crate::styling::APP_THEME;

fn main() -> Result<(), iced_winit::Error> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter("iced_wgpu=error,iced_matrix_client=debug")
        .pretty()
        .init();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let _enter = rt.enter();
    iced_winit::run(IcedMatrixClient)
}

struct IcedMatrixClient;
impl Program for IcedMatrixClient {
    type Executor = TokioHandleExecutor;
    type Message = app::AppMessage;
    type Renderer = Renderer;
    type State = app::State;
    type Theme = Theme;

    fn theme(&self, _state: &Self::State, _window: window::Id) -> Option<Self::Theme> {
        Some(APP_THEME)
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
        app::boot()
    }

    fn subscription(&self, state: &Self::State) -> iced::Subscription<Self::Message> {
        app::subscription(state)
    }

    fn update(&self, state: &mut Self::State, message: Self::Message) -> iced::Task<Self::Message> {
        app::update(state, message)
    }

    fn view<'a>(
        &self,
        state: &'a Self::State,
        _window: window::Id,
    ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer> {
        app::view(state)
    }
}

struct TokioHandleExecutor(tokio::runtime::Handle);

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
