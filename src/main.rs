fn main() {
    let app = egui_mp3s::Mp3sApp::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
