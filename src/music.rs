use std::path::PathBuf;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
#[derive(Clone, Default)]
pub struct MusicFilter {
    /// Root dir is not a `PathBuf` so we can easily plug it into egui's text input
    pub root_dir: String,
    pub query: String,
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
#[derive(Clone, Default)]
pub struct MusicList {
    pub loading: bool,
    pub songs: Vec<PathBuf>,
}

pub fn spawn_worker(
    filter_receiver: Receiver<MusicFilter>,
    list_sender: Sender<MusicList>
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut music_list = MusicList::default();

        while let Ok(filter) = filter_receiver.recv() {
            music_list.loading = true;
            list_sender.send(music_list.clone()).unwrap();
            music_list.songs.clear();

            for entry in std::fs::read_dir(&filter.root_dir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();

                if path.extension() != Some("mp3".as_ref()) {
                    continue;
                }

                let filename = path.strip_prefix(&filter.root_dir).unwrap();
                if filter.query.trim() != "" &&
                    !filename.to_string_lossy().to_lowercase().contains(&filter.query.to_lowercase()) {
                        continue;
                }

                music_list.songs.push(filename.to_owned());
            }

            list_sender.send(music_list.clone()).unwrap();
        }
    })
}
