use std::path::PathBuf;

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct MusicFilter {
    /// Root dir is not a `PathBuf` so we can easily plug it into egui's text input
    pub root_dir: String,
    pub filter: String,
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct MusicList {
    pub loading: bool,
    pub songs: Vec<PathBuf>,
}
