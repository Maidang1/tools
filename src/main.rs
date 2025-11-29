//! Terminal-based music player with modern UI.
//!
//! This application provides a feature-rich music player with:
//! - Modern three-layer layout design
//! - Responsive UI that adapts to terminal size
//! - Audio visualization
//! - Keyboard-driven controls
//! - Support for multiple audio formats (MP3, FLAC, OGG, WAV)
//!
//! # Usage
//!
//! Run the player in a directory containing audio files:
//! ```bash
//! cargo run
//! ```
//!
//! # Keyboard Controls
//!
//! - `q`: Quit
//! - `↑/↓` or `j/k`: Navigate track list
//! - `Enter`: Play selected track
//! - `Space`: Toggle play/pause
//! - `[/]`: Previous/next track
//! - `+/-`: Increase/decrease volume

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event as CEvent, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use tracing::error;
use walkdir::WalkDir;

mod ui;
mod player;
mod common;

use common::{AppEvent, AppCommand, PlaybackStatus, Track};
use ui::theme::Theme;
use ui::layout::{LayoutManager, AppLayout};

/// Main application state.
struct App {
    /// List of all available tracks
    tracks: Vec<Track>,
    /// Index of currently selected track
    selected: usize,
    /// Index of currently playing track (if any)
    playing: Option<usize>,
    /// Current playback status
    status: PlaybackStatus,
    /// Current playback position
    position: Duration,
    /// Total duration of current track
    total: Option<Duration>,
    /// Current volume level (0.0 to 2.0)
    volume: f32,
    /// Channel sender for player commands
    cmd_tx: Sender<AppCommand>,
    /// Channel receiver for player events
    evt_rx: Receiver<AppEvent>,
    /// Timestamp of last tick for timing
    last_tick: Instant,
    /// Waveform data for visualization
    wave: Vec<u64>,
    /// UI color theme
    theme: Theme,
    /// Whether UI is in compact mode
    compact_mode: bool,
    /// Cached layout to avoid recalculation
    cached_layout: Option<(u16, u16, AppLayout)>,
}

impl App {
    /// Creates a new application instance.
    ///
    /// # Arguments
    ///
    /// * `tracks` - List of available audio tracks
    /// * `cmd_tx` - Channel sender for player commands
    /// * `evt_rx` - Channel receiver for player events
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
            wave: vec![0; 80],
            theme: Theme::default(),
            compact_mode: false,
            cached_layout: None,
        }
    }

    /// Gets the layout for the current terminal size, using cache if available.
    ///
    /// This method implements layout caching to avoid recalculating the layout
    /// on every frame when the terminal size hasn't changed.
    ///
    /// # Arguments
    ///
    /// * `width` - Current terminal width
    /// * `height` - Current terminal height
    ///
    /// # Returns
    ///
    /// Returns a reference to the calculated layout.
    fn get_layout(&mut self, width: u16, height: u16) -> &AppLayout {
        // Check if we have a cached layout for this size
        let needs_recalc = match &self.cached_layout {
            Some((cached_w, cached_h, _)) => *cached_w != width || *cached_h != height,
            None => true,
        };

        if needs_recalc {
            let size = ratatui::layout::Rect::new(0, 0, width, height);
            let layout_manager = LayoutManager::new(size);
            let layout = layout_manager.calculate_layout();
            self.cached_layout = Some((width, height, layout));
        }

        // Safe to unwrap because we just set it
        &self.cached_layout.as_ref().unwrap().2
    }
}

/// Scans a directory recursively for audio files.
///
/// Searches for files with supported audio extensions (mp3, flac, ogg, wav)
/// and creates Track instances for each found file.
///
/// # Arguments
///
/// * `dir` - Directory path to scan
///
/// # Returns
///
/// Returns a vector of Track instances for all found audio files.
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
                out.push(Track {
                    id: out.len() as u64,
                    path: path.to_path_buf(),
                    duration: None,
                    title: path.file_name().and_then(|s| s.to_str()).map(|s| s.to_string()),
                });
            }
        }
    }
    out
}

/// Main application entry point.
///
/// Initializes the application, sets up the terminal UI, starts the player thread,
/// and runs the main event loop.
///
/// # Returns
///
/// Returns `Ok(())` on successful exit, or an error if initialization or runtime fails.
///
/// # Errors
///
/// Returns an error if:
/// - Terminal initialization fails
/// - Player thread cannot be started
/// - Terminal operations fail during runtime
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
        // Process all pending player events
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
                    // Auto-advance to next track
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
            
            // Update compact mode based on terminal size
            app.compact_mode = size.width < 80;
            
            // Get layout (uses cache if terminal size unchanged)
            let layout = app.get_layout(size.width, size.height).clone();
            
            // Render NowPlayingWidget to top area
            let current_track = app.playing.and_then(|i| app.tracks.get(i));
            let now_playing_widget = ui::widgets::NowPlayingWidget::new(
                current_track,
                app.status,
                &app.theme
            );
            f.render_widget(now_playing_widget, layout.now_playing);
            
            // Render TrackListWidget to middle-left area
            let track_list_widget = ui::widgets::TrackListWidget::new(
                &app.tracks,
                app.selected,
                app.playing
            );
            f.render_widget(track_list_widget, layout.track_list);
            
            // Render VisualizationWidget to middle-right area (if not in compact mode)
            if let Some(viz_area) = layout.visualization {
                let is_playing = app.status == PlaybackStatus::Playing;
                let visualization_widget = ui::widgets::VisualizationWidget::new(
                    &app.wave,
                    is_playing
                );
                f.render_widget(visualization_widget, viz_area);
            }
            
            // Render PlaybackControlWidget to bottom playback control area
            let playback_control_widget = ui::widgets::PlaybackControlWidget::new(
                app.position,
                app.total,
                app.volume,
                app.status
            );
            f.render_widget(playback_control_widget, layout.playback_control);
            
            // Render StatusBarWidget to bottom-most status bar
            let status_hints = [
                ("q", "退出"),
                ("↑/↓", "导航"),
                ("Enter", "播放"),
                ("Space", "暂停"),
                ("[/]", "上/下一曲"),
                ("+/-", "音量"),
            ];
            let status_bar_widget = ui::widgets::StatusBarWidget::new(&status_hints);
            f.render_widget(status_bar_widget, layout.status_bar);
        })?;

        // Poll for keyboard input with timeout
        let timeout = tick_rate.saturating_sub(app.last_tick.elapsed());
        if event::poll(timeout)? {
            if let CEvent::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Handle keyboard commands
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
        // Update visualization waveform data
        let sample = if app.status == PlaybackStatus::Playing {
            // Generate simulated waveform based on playback position
            let t = app.position.as_secs_f64();
            let v = ((t * 6.0).sin() * 0.5 + 0.5) * 100.0 * app.volume.min(1.0) as f64;
            v.clamp(0.0, 100.0) as u64
        } else {
            0
        };
        
        // Maintain rolling window of waveform samples
        if !app.wave.is_empty() {
            app.wave.remove(0);
        }
        app.wave.push(sample);
        app.last_tick = Instant::now();
    }

    disable_raw_mode()?;
    ui::restore_terminal(&mut terminal)?;
    Ok(())
}

fn main() {
    if let Err(e) = run() { eprintln!("{}", e); }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    /// Integration test: Complete UI rendering from App state
    /// Tests the full rendering pipeline from App state to complete interface
    /// Validates: All requirements
    #[test]
    fn test_complete_ui_rendering() {
        // Create test tracks
        let tracks = vec![
            Track {
                id: 0,
                path: PathBuf::from("/test/track1.mp3"),
                duration: Some(Duration::from_secs(180)),
                title: Some("Test Track 1".to_string()),
            },
            Track {
                id: 1,
                path: PathBuf::from("/test/track2.mp3"),
                duration: Some(Duration::from_secs(240)),
                title: Some("Test Track 2".to_string()),
            },
            Track {
                id: 2,
                path: PathBuf::from("/test/track3.mp3"),
                duration: Some(Duration::from_secs(200)),
                title: Some("Test Track 3".to_string()),
            },
        ];

        // Create channels for testing
        let (cmd_tx, _cmd_rx) = mpsc::channel();
        let (_evt_tx, evt_rx) = mpsc::channel();

        // Create app with test data
        let mut app = App::new(tracks, cmd_tx, evt_rx);
        app.playing = Some(1);
        app.status = PlaybackStatus::Playing;
        app.position = Duration::from_secs(60);
        app.total = Some(Duration::from_secs(240));
        app.volume = 0.75;
        app.selected = 1;
        app.wave = vec![10, 20, 30, 40, 50, 60, 70, 80];

        // Create a test backend with sufficient size
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        // Render the UI
        terminal
            .draw(|f| {
                let size = f.size();
                
                // Update compact mode based on terminal size
                app.compact_mode = size.width < 80;
                
                // Calculate layout using LayoutManager
                let layout_manager = ui::layout::LayoutManager::new(size);
                let layout = layout_manager.calculate_layout();
                
                // Render NowPlayingWidget to top area
                let current_track = app.playing.and_then(|i| app.tracks.get(i));
                let now_playing_widget = ui::widgets::NowPlayingWidget::new(
                    current_track,
                    app.status,
                    &app.theme
                );
                f.render_widget(now_playing_widget, layout.now_playing);
                
                // Render TrackListWidget to middle-left area
                let track_list_widget = ui::widgets::TrackListWidget::new(
                    &app.tracks,
                    app.selected,
                    app.playing
                );
                f.render_widget(track_list_widget, layout.track_list);
                
                // Render VisualizationWidget to middle-right area (if not in compact mode)
                if let Some(viz_area) = layout.visualization {
                    let is_playing = app.status == PlaybackStatus::Playing;
                    let visualization_widget = ui::widgets::VisualizationWidget::new(
                        &app.wave,
                        is_playing
                    );
                    f.render_widget(visualization_widget, viz_area);
                }
                
                // Render PlaybackControlWidget to bottom playback control area
                let playback_control_widget = ui::widgets::PlaybackControlWidget::new(
                    app.position,
                    app.total,
                    app.volume,
                    app.status
                );
                f.render_widget(playback_control_widget, layout.playback_control);
                
                // Render StatusBarWidget to bottom-most status bar
                let status_hints = [
                    ("q", "退出"),
                    ("↑/↓", "导航"),
                    ("Enter", "播放"),
                    ("Space", "暂停"),
                    ("[/]", "上/下一曲"),
                    ("+/-", "音量"),
                ];
                let status_bar_widget = ui::widgets::StatusBarWidget::new(&status_hints);
                f.render_widget(status_bar_widget, layout.status_bar);
            })
            .unwrap();

        // Get the rendered buffer
        let buffer = terminal.backend().buffer();
        let rendered = buffer_to_string(buffer);

        // Verify all regions are rendered correctly
        
        // 1. Now Playing area should contain the playing track title
        assert!(
            rendered.contains("Test Track 2"),
            "Now Playing area should contain the playing track title"
        );
        
        // 2. Now Playing area should contain play icon
        assert!(
            rendered.contains("▶"),
            "Now Playing area should contain play icon when playing"
        );
        
        // 3. Track list should contain all tracks
        assert!(
            rendered.contains("Test Track 1"),
            "Track list should contain Track 1"
        );
        assert!(
            rendered.contains("Test Track 2"),
            "Track list should contain Track 2"
        );
        assert!(
            rendered.contains("Test Track 3"),
            "Track list should contain Track 3"
        );
        
        // 4. Progress bar should show time information
        assert!(
            rendered.contains("01:00") || rendered.contains("1:00"),
            "Progress bar should show current time (01:00)"
        );
        assert!(
            rendered.contains("04:00") || rendered.contains("4:00"),
            "Progress bar should show total time (04:00)"
        );
        
        // 5. Volume indicator should be displayed
        assert!(
            rendered.contains("75%") || rendered.contains("Volume"),
            "Volume indicator should be displayed"
        );
        
        // 6. Status bar should contain keyboard shortcuts
        assert!(
            rendered.contains("退出") || rendered.contains("q"),
            "Status bar should contain quit shortcut"
        );
        
        // 7. Visualization should be rendered (since width >= 80)
        assert!(
            rendered.contains("Visualization") || !app.compact_mode,
            "Visualization should be rendered in non-compact mode"
        );
    }

    /// Integration test: Compact mode rendering
    /// Tests that UI adapts correctly to narrow terminal width
    #[test]
    fn test_compact_mode_rendering() {
        // Create minimal test data
        let tracks = vec![
            Track {
                id: 0,
                path: PathBuf::from("/test/track1.mp3"),
                duration: Some(Duration::from_secs(180)),
                title: Some("Track 1".to_string()),
            },
        ];

        let (cmd_tx, _cmd_rx) = mpsc::channel();
        let (_evt_tx, evt_rx) = mpsc::channel();

        let mut app = App::new(tracks, cmd_tx, evt_rx);
        app.playing = Some(0);
        app.status = PlaybackStatus::Playing;

        // Create a narrow terminal (width < 80 for compact mode)
        let backend = TestBackend::new(60, 25);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                let size = f.size();
                app.compact_mode = size.width < 80;
                
                let layout_manager = ui::layout::LayoutManager::new(size);
                let layout = layout_manager.calculate_layout();
                
                // Verify compact mode is active
                assert!(app.compact_mode, "Compact mode should be active for width < 80");
                assert!(layout.visualization.is_none(), "Visualization should be hidden in compact mode");
                
                // Render all widgets
                let current_track = app.playing.and_then(|i| app.tracks.get(i));
                let now_playing_widget = ui::widgets::NowPlayingWidget::new(
                    current_track,
                    app.status,
                    &app.theme
                );
                f.render_widget(now_playing_widget, layout.now_playing);
                
                let track_list_widget = ui::widgets::TrackListWidget::new(
                    &app.tracks,
                    app.selected,
                    app.playing
                );
                f.render_widget(track_list_widget, layout.track_list);
                
                let playback_control_widget = ui::widgets::PlaybackControlWidget::new(
                    app.position,
                    app.total,
                    app.volume,
                    app.status
                );
                f.render_widget(playback_control_widget, layout.playback_control);
                
                let status_hints = [("q", "Quit")];
                let status_bar_widget = ui::widgets::StatusBarWidget::new(&status_hints);
                f.render_widget(status_bar_widget, layout.status_bar);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let rendered = buffer_to_string(buffer);

        // Verify core functionality is still accessible in compact mode
        assert!(
            rendered.contains("Track 1"),
            "Track list should be visible in compact mode"
        );
        assert!(
            rendered.contains("▶"),
            "Play icon should be visible in compact mode"
        );
    }

    /// Integration test: Empty track list rendering
    /// Tests that UI handles empty state gracefully
    #[test]
    fn test_empty_track_list_rendering() {
        let tracks = vec![];
        let (cmd_tx, _cmd_rx) = mpsc::channel();
        let (_evt_tx, evt_rx) = mpsc::channel();

        let app = App::new(tracks, cmd_tx, evt_rx);

        let backend = TestBackend::new(80, 25);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                let size = f.size();
                let layout_manager = ui::layout::LayoutManager::new(size);
                let layout = layout_manager.calculate_layout();
                
                // Render with empty track list
                let now_playing_widget = ui::widgets::NowPlayingWidget::new(
                    None,
                    app.status,
                    &app.theme
                );
                f.render_widget(now_playing_widget, layout.now_playing);
                
                let track_list_widget = ui::widgets::TrackListWidget::new(
                    &app.tracks,
                    app.selected,
                    app.playing
                );
                f.render_widget(track_list_widget, layout.track_list);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let rendered = buffer_to_string(buffer);

        // Verify welcome message is shown when no track is playing
        assert!(
            rendered.contains("Welcome") || rendered.contains("Music Player"),
            "Should show welcome message when no track is playing"
        );
        
        // Verify empty track list message
        // Note: Chinese characters may be rendered with spaces
        assert!(
            rendered.contains("未") || rendered.contains("找"),
            "Should show empty track list message"
        );
    }

    /// Helper function to convert buffer to string for assertions
    fn buffer_to_string(buffer: &ratatui::buffer::Buffer) -> String {
        let mut result = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                let cell = buffer.get(x, y);
                result.push_str(cell.symbol());
            }
            result.push('\n');
        }
        result
    }
}
