use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::Result;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use tracing::info;

use crate::common::{AppCommand, AppEvent};

pub fn start(cmd_rx: Receiver<AppCommand>, evt_tx: Sender<AppEvent>) -> Result<JoinHandle<()>> {
    let handle = thread::spawn(move || {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let mut sink: Option<Sink> = None;
        let mut volume = 1.0f32;
        let mut start_at: Option<Instant> = None;
        let mut paused_acc = Duration::from_secs(0);
        let mut total: Option<Duration> = None;
        let mut current_index: Option<usize> = None;

        loop {
            while let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    AppCommand::Play { index, path } => {
                        if let Some(s) = sink.take() { s.stop(); }
                        let file = match File::open(&path) { Ok(f) => f, Err(_) => { let _ = evt_tx.send(AppEvent::Error { message: format!("无法打开文件: {}", path.display()) }); continue; } };
                        let source = match Decoder::new(BufReader::new(file)) { Ok(s) => s, Err(_) => { let _ = evt_tx.send(AppEvent::Error { message: format!("不支持的格式: {}", path.display()) }); continue; } };
                        total = source.total_duration();
                        let s = Sink::try_new(&stream_handle).unwrap();
                        s.set_volume(volume);
                        s.append(source);
                        sink = Some(s);
                        start_at = Some(Instant::now());
                        paused_acc = Duration::from_secs(0);
                        current_index = Some(index);
                        let _ = evt_tx.send(AppEvent::TrackStarted { index, duration: total });
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
                    total = None;
                    current_index = None;
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

fn _stream_handle(_h: &OutputStreamHandle) {}
