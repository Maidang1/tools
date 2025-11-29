use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event as CEvent, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Gauge, Paragraph};
use tracing::{error, info};
use walkdir::WalkDir;

mod ui;
mod player;
mod common;

use common::{AppEvent, AppCommand, PlaybackStatus, Track};

struct App {
    tracks: Vec<Track>,
    selected: usize,
    playing: Option<usize>,
    status: PlaybackStatus,
    position: Duration,
    total: Option<Duration>,
    volume: f32,
    cmd_tx: Sender<AppCommand>,
    evt_rx: Receiver<AppEvent>,
    last_tick: Instant,
}

impl App {
    fn new(tracks: Vec<Track>, cmd_tx: Sender<AppCommand>, evt_rx: Receiver<AppEvent>) -> Self {
        Self {
            tracks,
            selected: 0,
            playing: None,
            status: PlaybackStatus::Stopped,
            position: Duration::from_secs(0),
            total: None,
            volume: 1.0,
            cmd_tx,
            evt_rx,
            last_tick: Instant::now(),
        }
    }
}

fn scan_directory(dir: PathBuf) -> Vec<Track> {
    let mut out = Vec::new();
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ok = matches!(ext.to_lowercase().as_str(), "mp3" | "flac" | "ogg" | "wav");
            if ok {
                out.push(Track { id: out.len() as u64, path: path.to_path_buf(), duration: None, title: path.file_name().and_then(|s| s.to_str()).map(|s| s.to_string()) });
            }
        }
    }
    out
}

fn run() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();
    let cwd = std::env::current_dir()?;
    let tracks = scan_directory(cwd);
    let (cmd_tx, cmd_rx) = mpsc::channel();
    let (evt_tx, evt_rx) = mpsc::channel();
    let _player_handle = player::start(cmd_rx, evt_tx)?;

    enable_raw_mode()?;
    let mut terminal = ui::init_terminal()?;
    let tick_rate = Duration::from_millis(200);

    let mut app = App::new(tracks, cmd_tx.clone(), evt_rx);

    loop {
        while let Ok(evt) = app.evt_rx.try_recv() {
            match evt {
                AppEvent::TrackStarted { index, duration } => {
                    app.playing = Some(index);
                    app.status = PlaybackStatus::Playing;
                    app.position = Duration::from_secs(0);
                    app.total = duration;
                }
                AppEvent::Progress { position } => {
                    app.position = position;
                }
                AppEvent::TrackEnded => {
                    if let Some(i) = app.playing {
                        let next = if i + 1 < app.tracks.len() { i + 1 } else { i };
                        app.selected = next;
                        let t = app.tracks[next].clone();
                        app.cmd_tx.send(AppCommand::Play { index: next, path: t.path }).ok();
                    }
                }
                AppEvent::Error { message } => {
                    error!("{}", message);
                    app.status = PlaybackStatus::Stopped;
                }
            }
        }

        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(3), Constraint::Min(5), Constraint::Length(3)].as_ref()).split(size);
            let top = Paragraph::new(match app.playing { Some(i) => format!("正在播放: {}", app.tracks[i].title.clone().unwrap_or_else(|| app.tracks[i].path.display().to_string())), None => "未播放".to_string() }).block(Block::default().borders(Borders::ALL).title("当前"));
            f.render_widget(top, chunks[0]);

            let items: Vec<ListItem> = app.tracks.iter().enumerate().map(|(i, t)| {
                let text = t.title.clone().unwrap_or_else(|| t.path.display().to_string());
                let prefix = if Some(i) == app.playing { "▶ " } else { "  " };
                ListItem::new(format!("{}{}", prefix, text))
            }).collect();
            let list = List::new(items).block(Block::default().borders(Borders::ALL).title("曲目")).highlight_symbol("> ");
            f.render_stateful_widget(list, chunks[1], &mut ui::list_state(app.selected));

            let progress = match app.total { Some(total) => {
                let ratio = app.position.as_secs_f64() / total.as_secs_f64();
                Gauge::default().block(Block::default().borders(Borders::ALL).title("进度")).gauge_style(Style::default().fg(Color::Cyan)).ratio(ratio.clamp(0.0, 1.0)).label(format!("{}/{}", fmt_d(app.position), fmt_d(total)))
            } None => Gauge::default().block(Block::default().borders(Borders::ALL).title("进度")).gauge_style(Style::default().fg(Color::Cyan)).ratio(0.0).label("--/--") };
            f.render_widget(progress, chunks[2]);
        })?;

        let timeout = tick_rate.saturating_sub(app.last_tick.elapsed());
        if event::poll(timeout)? {
            if let CEvent::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Down | KeyCode::Char('j') => {
                            if app.selected + 1 < app.tracks.len() { app.selected += 1; }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if app.selected > 0 { app.selected -= 1; }
                        }
                        KeyCode::Enter => {
                            let t = app.tracks[app.selected].clone();
                            app.cmd_tx.send(AppCommand::Play { index: app.selected, path: t.path }).ok();
                        }
                        KeyCode::Char(' ') => {
                            app.cmd_tx.send(AppCommand::TogglePlayPause).ok();
                        }
                        KeyCode::Char(']') => {
                            let next = if app.selected + 1 < app.tracks.len() { app.selected + 1 } else { app.selected };
                            app.selected = next;
                            let t = app.tracks[next].clone();
                            app.cmd_tx.send(AppCommand::Play { index: next, path: t.path }).ok();
                        }
                        KeyCode::Char('[') => {
                            let prev = if app.selected > 0 { app.selected - 1 } else { 0 };
                            app.selected = prev;
                            let t = app.tracks[prev].clone();
                            app.cmd_tx.send(AppCommand::Play { index: prev, path: t.path }).ok();
                        }
                        KeyCode::Char('+') => {
                            app.volume = (app.volume + 0.05).min(2.0);
                            app.cmd_tx.send(AppCommand::SetVolume(app.volume)).ok();
                        }
                        KeyCode::Char('-') => {
                            app.volume = (app.volume - 0.05).max(0.0);
                            app.cmd_tx.send(AppCommand::SetVolume(app.volume)).ok();
                        }
                        _ => {}
                    }
                }
            }
        }
        app.last_tick = Instant::now();
    }

    disable_raw_mode()?;
    ui::restore_terminal(&mut terminal)?;
    Ok(())
}

fn fmt_d(d: Duration) -> String { format!("{:02}:{:02}", d.as_secs() / 60, d.as_secs() % 60) }

fn main() {
    if let Err(e) = run() { eprintln!("{}", e); }
}
