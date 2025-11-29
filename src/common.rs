use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone)]
pub struct Track {
    pub id: u64,
    pub path: PathBuf,
    pub duration: Option<Duration>,
    pub title: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlaybackStatus {
    Stopped,
    Paused,
    Playing,
}

pub enum AppCommand {
    Play { index: usize, path: PathBuf },
    TogglePlayPause,
    SetVolume(f32),
}

pub enum AppEvent {
    TrackStarted { index: usize, duration: Option<Duration> },
    Progress { position: Duration },
    TrackEnded,
    Error { message: String },
}
