use std::path::Path;
use std::fs::File;
use std::io::BufReader;

use rodio::{Decoder, OutputStream, Sink};
// use rodio::DeviceTrait;
// use rodio::cpal::traits::HostTrait;

#[derive(Default)]
pub struct Player {
    sink: Option<Sink>,
}

impl Player {
    pub fn new() -> Self {
        Player::default()
    }

    pub fn is_playing(&self) -> bool {
        self.sink.as_ref().map(|s| !s.is_paused()).unwrap_or(false)
    }

    // Symphonia player works? /home/andrew/src/Symphonia/symphonia-play/src/output.rs
    // Try pipewire-jack: https://github.com/RustAudio/cpal/issues/554

    // TODO return an error and show a popup
    pub fn play(&mut self, path: &Path) {
        // let host = ::rodio::cpal::default_host();
        // let device = host.output_devices().unwrap().find(|d| d.name().unwrap() == "pulse").unwrap();
        // println!("{:?}", device.name());

        // Get a output stream handle to the default physical sound device
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        // let (_stream, stream_handle) = OutputStream::try_from_device(&device).unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        // Load a sound from a file
        let file = BufReader::new(File::open(path).unwrap());

        // Decode that sound file into a source
        let source = Decoder::new(file).unwrap();

        // Play the sound directly on the device
        sink.append(source);

        self.sink = Some(sink);
    }

    pub fn pause(&self) {
        self.sink.as_ref().map(|s| s.pause());
    }
}
