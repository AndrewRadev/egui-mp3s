use std::sync::mpsc;

use egui_mp3s::app::Mp3sApp;
use egui_mp3s::music::spawn_worker;

fn main() {
    ::env_logger::init();

    let native_options = eframe::NativeOptions::default();

    let (worker_sender, worker_receiver) = mpsc::channel();
    let (ui_sender,     ui_receiver)     = mpsc::channel();

    let app = Mp3sApp::new(worker_sender, ui_receiver);
    spawn_worker(worker_receiver, ui_sender);

    eframe::run_native(Box::new(app), native_options);
}
