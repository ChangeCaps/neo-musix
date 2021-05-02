mod app;
mod audio_clip;
mod audio_engine;
mod audio_source;
mod main_panel;
mod top_panel;

fn main() {
    simple_logger::SimpleLogger::from_env().init().unwrap();
    let app = app::App::new();

    eframe::run_native(Box::new(app));
}
