//! Layout calculation and responsive logic for the music player UI.
//!
//! This module provides the layout management system that calculates and manages
//! the positioning of all UI regions based on terminal size. It implements a
//! responsive design that adapts to different terminal dimensions.
//!
//! # Layout Structure
//!
//! The layout follows a three-layer vertical structure:
//! 1. **Top**: Now Playing area (fixed height: 3-5 lines)
//! 2. **Middle**: Main content area (flexible height)
//!    - Track list (left, 55% width)
//!    - Visualization (right, 45% width, hidden in compact mode)
//! 3. **Bottom**: Playback controls (fixed height: 4-5 lines)
//! 4. **Bottom-most**: Status bar (fixed height: 1 line)
//!
//! # Responsive Behavior
//!
//! - **Width < 80 columns**: Compact mode - hides visualization, track list takes full width
//! - **Height < 20 lines**: Reduces fixed heights to fit more content
//!
//! # Example
//!
//! ```no_run
//! use ratatui::prelude::*;
//! use tools_rs::ui::layout::LayoutManager;
//!
//! let terminal_size = Rect::new(0, 0, 100, 30);
//! let manager = LayoutManager::new(terminal_size);
//! let layout = manager.calculate_layout();
//!
//! // Use layout.now_playing, layout.track_list, etc. for rendering
//! ```

use ratatui::prelude::*;

/// Represents the calculated layout areas for all UI regions
#[derive(Debug, Clone)]
pub struct AppLayout {
    /// Top area displaying current playing track information
    pub now_playing: Rect,
    /// Middle-left area showing the track list
    pub track_list: Rect,
    /// Middle-right area showing visualization (None in compact mode)
    pub visualization: Option<Rect>,
    /// Bottom area with playback controls and progress bar
    pub playback_control: Rect,
    /// Bottom-most area showing status and keyboard shortcuts
    pub status_bar: Rect,
}

/// Manages layout calculation and responsive behavior
pub struct LayoutManager {
    terminal_size: Rect,
}

impl LayoutManager {
    /// Creates a new LayoutManager with the given terminal size
    pub fn new(size: Rect) -> Self {
        Self {
            terminal_size: size,
        }
    }

    /// Calculates the complete layout based on terminal size
    pub fn calculate_layout(&self) -> AppLayout {
        let size = self.terminal_size;
        
        // Define fixed heights for different regions
        let now_playing_height = if size.height < 20 { 3 } else { 5 };
        let playback_control_height = if size.height < 20 { 4 } else { 5 };
        let status_bar_height = 1;
        
        // Calculate main vertical layout (three layers)
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(now_playing_height),      // Top: Now Playing
                Constraint::Min(5),                          // Middle: Main content (flexible)
                Constraint::Length(playback_control_height), // Bottom: Playback controls
                Constraint::Length(status_bar_height),       // Bottom-most: Status bar
            ])
            .split(size);
        
        let now_playing = vertical_chunks[0];
        let middle_area = vertical_chunks[1];
        let playback_control = vertical_chunks[2];
        let status_bar = vertical_chunks[3];
        
        // Handle middle area: split horizontally if not in compact mode
        let (track_list, visualization) = if self.is_compact_mode() {
            // Compact mode: track list takes full width, no visualization
            (middle_area, None)
        } else {
            // Normal mode: split middle area horizontally
            // Track list: 55%, Visualization: 45%
            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(55), // Track list
                    Constraint::Percentage(45), // Visualization
                ])
                .split(middle_area);
            
            (horizontal_chunks[0], Some(horizontal_chunks[1]))
        };
        
        AppLayout {
            now_playing,
            track_list,
            visualization,
            playback_control,
            status_bar,
        }
    }

    /// Determines if the UI should be in compact mode (width < 80)
    pub fn is_compact_mode(&self) -> bool {
        self.terminal_size.width < 80
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // **Feature: modern-player-layout, Property 1: 布局三区域结构**
    // **Validates: Requirements 1.1, 1.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_layout_three_regions(width in 20u16..200u16, height in 10u16..100u16) {
            let size = Rect::new(0, 0, width, height);
            let manager = LayoutManager::new(size);
            let layout = manager.calculate_layout();
            
            // Verify three main vertical regions exist
            let now_playing = layout.now_playing;
            let middle = layout.track_list;
            let playback = layout.playback_control;
            let status = layout.status_bar;
            
            // Check regions are ordered from top to bottom
            prop_assert!(now_playing.y == 0, "Now playing should start at top");
            prop_assert!(middle.y >= now_playing.y + now_playing.height, 
                "Middle area should be below now playing");
            prop_assert!(playback.y >= middle.y + middle.height, 
                "Playback control should be below middle area");
            prop_assert!(status.y >= playback.y + playback.height,
                "Status bar should be below playback control");
            
            // Check regions don't overlap
            prop_assert!(now_playing.y + now_playing.height <= middle.y,
                "Now playing and middle should not overlap");
            prop_assert!(middle.y + middle.height <= playback.y,
                "Middle and playback should not overlap");
            prop_assert!(playback.y + playback.height <= status.y,
                "Playback and status should not overlap");
            
            // Check all regions fit within terminal
            prop_assert!(status.y + status.height <= height,
                "All regions should fit within terminal height");
        }
    }

    // **Feature: modern-player-layout, Property 7: 中部区域水平分割**
    // **Validates: Requirements 4.1**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_middle_area_horizontal_split(width in 80u16..200u16, height in 20u16..100u16) {
            let size = Rect::new(0, 0, width, height);
            let manager = LayoutManager::new(size);
            let layout = manager.calculate_layout();
            
            // When width >= 80, middle area should be split horizontally
            prop_assert!(layout.visualization.is_some(), 
                "Visualization should exist when width >= 80");
            
            if let Some(viz) = layout.visualization {
                let track_list = layout.track_list;
                
                // Both should be in the same vertical position (side by side)
                prop_assert_eq!(track_list.y, viz.y, 
                    "Track list and visualization should be at same vertical position");
                prop_assert_eq!(track_list.height, viz.height,
                    "Track list and visualization should have same height");
                
                // Track list should be on the left, visualization on the right
                prop_assert!(track_list.x < viz.x,
                    "Track list should be left of visualization");
                prop_assert!(track_list.x + track_list.width <= viz.x,
                    "Track list and visualization should not overlap");
            }
        }
    }

    // **Feature: modern-player-layout, Property 8: 曲目列表宽度比例**
    // **Validates: Requirements 4.2**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_track_list_width_ratio(width in 80u16..200u16, height in 20u16..100u16) {
            let size = Rect::new(0, 0, width, height);
            let manager = LayoutManager::new(size);
            let layout = manager.calculate_layout();
            
            if let Some(viz) = layout.visualization {
                let track_list = layout.track_list;
                let total_width = track_list.width + viz.width;
                let track_ratio = (track_list.width as f32) / (total_width as f32);
                
                // Track list should occupy 50-60% of middle area width
                prop_assert!((0.50..=0.60).contains(&track_ratio),
                    "Track list width should be 50-60% of middle area, got {:.2}%", 
                    track_ratio * 100.0);
            }
        }
    }

    // **Feature: modern-player-layout, Property 9: 可视化区宽度比例**
    // **Validates: Requirements 4.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_visualization_width_ratio(width in 80u16..200u16, height in 20u16..100u16) {
            let size = Rect::new(0, 0, width, height);
            let manager = LayoutManager::new(size);
            let layout = manager.calculate_layout();
            
            if let Some(viz) = layout.visualization {
                let track_list = layout.track_list;
                let total_width = track_list.width + viz.width;
                let viz_ratio = (viz.width as f32) / (total_width as f32);
                
                // Visualization should occupy 40-50% of middle area width
                prop_assert!((0.40..=0.50).contains(&viz_ratio),
                    "Visualization width should be 40-50% of middle area, got {:.2}%", 
                    viz_ratio * 100.0);
            }
        }
    }

    // **Feature: modern-player-layout, Property 15: 小高度终端布局适配**
    // **Validates: Requirements 8.2**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_small_height_terminal_adaptation(width in 20u16..200u16, height in 10u16..20u16) {
            let size = Rect::new(0, 0, width, height);
            let manager = LayoutManager::new(size);
            let layout = manager.calculate_layout();
            
            // Calculate total height used by all regions
            let total_used = layout.now_playing.height 
                + layout.track_list.height 
                + layout.playback_control.height 
                + layout.status_bar.height;
            
            // Total height should not exceed available height
            prop_assert!(total_used <= height,
                "Total layout height ({}) should not exceed terminal height ({})", 
                total_used, height);
            
            // Verify all regions are within bounds
            prop_assert!(layout.now_playing.y + layout.now_playing.height <= height);
            prop_assert!(layout.track_list.y + layout.track_list.height <= height);
            prop_assert!(layout.playback_control.y + layout.playback_control.height <= height);
            prop_assert!(layout.status_bar.y + layout.status_bar.height <= height);
        }
    }

    // **Feature: modern-player-layout, Property 14: 状态栏位于底部**
    // **Validates: Requirements 7.1**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_status_bar_at_bottom(width in 20u16..200u16, height in 10u16..100u16) {
            let size = Rect::new(0, 0, width, height);
            let manager = LayoutManager::new(size);
            let layout = manager.calculate_layout();
            
            // Status bar should be at the bottom-most position
            let status_bar = layout.status_bar;
            
            // Status bar height should be exactly 1
            prop_assert_eq!(status_bar.height, 1,
                "Status bar height should be 1, got {}", status_bar.height);
            
            // Status bar should be the last element (at the bottom)
            // Its bottom edge should be at or near the terminal height
            prop_assert_eq!(status_bar.y + status_bar.height, height,
                "Status bar should be at bottom: y={}, height={}, terminal_height={}", 
                status_bar.y, status_bar.height, height);
            
            // All other regions should be above the status bar
            prop_assert!(layout.now_playing.y < status_bar.y,
                "Now playing should be above status bar");
            prop_assert!(layout.track_list.y < status_bar.y,
                "Track list should be above status bar");
            prop_assert!(layout.playback_control.y < status_bar.y,
                "Playback control should be above status bar");
        }
    }
}
