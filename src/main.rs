use std::sync::mpsc;

use egui_mp3s::app::Mp3sApp;
use egui_mp3s::music::spawn_worker;

fn main() {
    ::env_logger::init();

    let native_options = eframe::NativeOptions::default();

    let (worker_sender, worker_receiver) = mpsc::channel();
    let (ui_sender,     ui_receiver)     = mpsc::channel();

    spawn_worker(worker_receiver, ui_sender);

    eframe::run_native("Basic Mp3 Viewer", native_options, Box::new(move |creation_context| {
        creation_context.egui_ctx.set_visuals(egui::Visuals::light());

        #[allow(unused_mut)]
        let mut app = Mp3sApp::new(worker_sender, ui_receiver);

        #[cfg(feature = "persistence")]
        if let Some(storage) = creation_context.storage {
            app.load_storage(storage);
        }

        app.refresh_filter();

        Box::new(app)
    }));
}
