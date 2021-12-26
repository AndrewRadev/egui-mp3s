use std::sync::mpsc;

use egui_mp3s::app::Mp3sApp;
use egui_mp3s::music::{MusicFilter, MusicList, spawn_worker};

fn main() {
    let native_options = eframe::NativeOptions::default();

    let (filter_sender, filter_receiver) = mpsc::channel::<MusicFilter>();
    let (list_sender,   list_receiver)   = mpsc::channel::<MusicList>();

    let app = Mp3sApp::new(filter_sender, list_receiver);
    spawn_worker(filter_receiver, list_sender);

    eframe::run_native(Box::new(app), native_options);
}
