#[derive(Debug)]
pub enum GuiApp {
    Music { name: String },
    Bluetooth { name: String, data: Vec<String> },
}

#[derive(Debug)]
pub enum GuiCommand {
    PlayPause,
    UpdateProgress,
    Shuffle,
    Like,
    Back,
    Forward,
    Seek(f32),
    RefreshUI {
        title: Option<String>,
        artist_name: Option<String>,
        album_name: Option<String>,
        album_art: Option<Vec<u8>>,
        duration: Option<f32>,
        is_playing: Option<bool>,
        is_liked: Option<bool>,
        is_shuffle: Option<bool>,
    },
    AppSelect(Vec<GuiApp>),
    AppTray,
}
