//! Audio playback engine for the music player.
//!
//! This module manages audio playback using the rodio library. It runs in a
//! separate thread and communicates with the main UI thread via channels.
//!
//! # Architecture
//!
//! The player operates on a command-event pattern:
//! - Receives `AppCommand` messages to control playback
//! - Sends `AppEvent` messages to notify the UI of state changes
//!
//! # Thread Safety
//!
//! The player runs in its own thread to avoid blocking the UI. All communication
//! is done through thread-safe channels.

use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::Result;
use rodio::{Decoder, OutputStream, Sink, Source};

use crate::common::{AppCommand, AppEvent};

/// Starts the audio player in a separate thread.
///
/// This function spawns a new thread that handles audio playback. The thread
/// listens for commands via `cmd_rx` and sends events via `evt_tx`.
///
/// # Arguments
///
/// * `cmd_rx` - Channel receiver for receiving playback commands
/// * `evt_tx` - Channel sender for sending playback events
///
/// # Returns
///
/// Returns a `JoinHandle` for the spawned player thread.
///
/// # Errors
///
/// Returns an error if the audio output stream cannot be initialized.
///
/// # Example
///
/// ```no_run
/// use std::sync::mpsc;
/// use tools_rs::player;
///
/// let (cmd_tx, cmd_rx) = mpsc::channel();
/// let (evt_tx, evt_rx) = mpsc::channel();
/// let handle = player::start(cmd_rx, evt_tx).unwrap();
/// ```
pub fn start(cmd_rx: Receiver<AppCommand>, evt_tx: Sender<AppEvent>) -> Result<JoinHandle<()>> {
    let handle = thread::spawn(move || {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let mut sink: Option<Sink> = None;
        let mut volume = 1.0f32;
        let mut start_at: Option<Instant> = None;
        let mut paused_acc = Duration::from_secs(0);

        loop {
            while let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    AppCommand::Play { index, path } => {
                        if let Some(s) = sink.take() { s.stop(); }
                        let file = match File::open(&path) { Ok(f) => f, Err(_) => { let _ = evt_tx.send(AppEvent::Error { message: format!("无法打开文件: {}", path.display()) }); continue; } };
                        let source = match Decoder::new(BufReader::new(file)) { Ok(s) => s, Err(_) => { let _ = evt_tx.send(AppEvent::Error { message: format!("不支持的格式: {}", path.display()) }); continue; } };
                        let duration = source.total_duration();
                        let s = Sink::try_new(&stream_handle).unwrap();
                        s.set_volume(volume);
                        s.append(source);
                        sink = Some(s);
                        start_at = Some(Instant::now());
                        paused_acc = Duration::from_secs(0);
                        let _ = evt_tx.send(AppEvent::TrackStarted { index, duration });
                    }
                    AppCommand::TogglePlayPause => {
                        if let Some(s) = &sink {
                            if s.is_paused() {
                                s.play();
                                if let Some(t0) = start_at { start_at = Some(Instant::now() - (t0.elapsed() - paused_acc)); }
                            } else {
                                s.pause();
                                if let Some(t0) = start_at { paused_acc = t0.elapsed(); }
                            }
                        }
                    }
                    AppCommand::SetVolume(v) => {
                        volume = v;
                        if let Some(s) = &sink { s.set_volume(volume); }
                    }
                }
            }

            if let Some(s) = &sink {
                if s.empty() {
                    let _ = evt_tx.send(AppEvent::TrackEnded);
                    sink = None;
                    start_at = None;
                    paused_acc = Duration::from_secs(0);
                } else if let Some(t0) = start_at {
                    let p = t0.elapsed();
                    let _ = evt_tx.send(AppEvent::Progress { position: p });
                }
            }

            thread::sleep(Duration::from_millis(200));
        }
    });
    Ok(handle)
}
