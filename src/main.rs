mod app;
mod extensions;
mod macros;
mod modal;
mod page;
mod styling;
mod tasks;
mod util;
mod worker;

fn main() -> Result<(), iced_winit::Error> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter("iced_wgpu=error,iced_matrix_client=debug")
        .pretty()
        .init();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let _enter = rt.enter();
    iced_winit::run(app::IcedMatrixClient)
}
