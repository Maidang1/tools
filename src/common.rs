//! Common data structures and types shared across the application.
//!
//! This module defines the core data types used for communication between
//! the UI, player, and application logic.

use std::path::PathBuf;
use std::time::Duration;

/// Represents a music track with metadata.
#[derive(Clone)]
pub struct Track {
    /// Unique identifier for the track
    #[allow(dead_code)]
    pub id: u64,
    /// File system path to the audio file
    pub path: PathBuf,
    /// Total duration of the track (if available)
    #[allow(dead_code)]
    pub duration: Option<Duration>,
    /// Display title for the track (if available, otherwise use filename)
    pub title: Option<String>,
}

/// Represents the current playback state.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlaybackStatus {
    /// No track is loaded or playing
    Stopped,
    /// Track is loaded but playback is paused
    #[allow(dead_code)]
    Paused,
    /// Track is currently playing
    Playing,
}

/// Commands sent from the UI to the player thread.
pub enum AppCommand {
    /// Start playing a specific track
    Play {
        /// Index of the track in the track list
        index: usize,
        /// Path to the audio file
        path: PathBuf,
    },
    /// Toggle between play and pause states
    TogglePlayPause,
    /// Set the playback volume (0.0 to 2.0, where 1.0 is 100%)
    SetVolume(f32),
}

/// Events sent from the player thread to the UI.
pub enum AppEvent {
    /// A track has started playing
    TrackStarted {
        /// Index of the track that started
        index: usize,
        /// Total duration of the track (if available)
        duration: Option<Duration>,
    },
    /// Playback progress update
    Progress {
        /// Current playback position
        position: Duration,
    },
    /// Current track has finished playing
    TrackEnded,
    /// An error occurred during playback
    Error {
        /// Error message describing what went wrong
        message: String,
    },
}
