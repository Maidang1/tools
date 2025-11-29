//! Theme and styling system for the music player UI.
//!
//! This module provides a configurable color theme system that defines
//! the visual appearance of the music player interface. It includes
//! predefined color schemes and style helper methods.
//!
//! # Design Philosophy
//!
//! The theme system follows modern UI design principles:
//! - High contrast for readability
//! - Consistent color usage across components
//! - Visual feedback through color changes
//! - Accessibility-friendly color combinations
//!
//! # Example
//!
//! ```
//! use tools_rs::ui::theme::Theme;
//!
//! let theme = Theme::default();
//! let title_style = theme.style_title();
//! let highlight_style = theme.style_highlight();
//! ```

use ratatui::prelude::*;

/// Color theme configuration for the UI
#[derive(Debug, Clone)]
pub struct Theme {
    /// Primary color for main UI elements
    pub primary: Color,
    /// Secondary color for supporting elements
    #[allow(dead_code)]
    pub secondary: Color,
    /// Accent color for highlights and emphasis
    #[allow(dead_code)]
    pub accent: Color,
    /// Main text color
    #[allow(dead_code)]
    pub text: Color,
    /// Dimmed text color for less important information
    #[allow(dead_code)]
    pub text_dim: Color,
    /// Border color
    pub border: Color,
    /// Highlight color for selected items
    #[allow(dead_code)]
    pub highlight: Color,
    /// Progress bar color
    #[allow(dead_code)]
    pub progress: Color,
}

impl Default for Theme {
    /// Creates a default theme with modern color scheme
    fn default() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Blue,
            accent: Color::Magenta,
            text: Color::White,
            text_dim: Color::Gray,
            border: Color::DarkGray,
            highlight: Color::Yellow,
            progress: Color::Green,
        }
    }
}

impl Theme {
    /// Returns a style for title text
    pub fn style_title(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    /// Returns a style for regular text
    #[allow(dead_code)]
    pub fn style_text(&self) -> Style {
        Style::default().fg(self.text)
    }

    /// Returns a style for highlighted/selected items
    #[allow(dead_code)]
    pub fn style_highlight(&self) -> Style {
        Style::default()
            .fg(self.highlight)
            .bg(self.secondary)
            .add_modifier(Modifier::BOLD)
    }

    /// Returns a style for borders
    pub fn style_border(&self) -> Style {
        Style::default().fg(self.border)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_default_colors() {
        let theme = Theme::default();
        
        // Verify all colors are set according to requirements 9.1 and 9.2
        assert_eq!(theme.primary, Color::Cyan);
        assert_eq!(theme.secondary, Color::Blue);
        assert_eq!(theme.accent, Color::Magenta);
        assert_eq!(theme.text, Color::White);
        assert_eq!(theme.text_dim, Color::Gray);
        assert_eq!(theme.border, Color::DarkGray);
        assert_eq!(theme.highlight, Color::Yellow);
        assert_eq!(theme.progress, Color::Green);
    }

    #[test]
    fn test_style_title() {
        let theme = Theme::default();
        let style = theme.style_title();
        
        assert_eq!(style.fg, Some(Color::Cyan));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_style_text() {
        let theme = Theme::default();
        let style = theme.style_text();
        
        assert_eq!(style.fg, Some(Color::White));
    }

    #[test]
    fn test_style_highlight() {
        let theme = Theme::default();
        let style = theme.style_highlight();
        
        assert_eq!(style.fg, Some(Color::Yellow));
        assert_eq!(style.bg, Some(Color::Blue));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_style_border() {
        let theme = Theme::default();
        let style = theme.style_border();
        
        assert_eq!(style.fg, Some(Color::DarkGray));
    }

    #[test]
    fn test_theme_clone() {
        let theme1 = Theme::default();
        let theme2 = theme1.clone();
        
        assert_eq!(theme1.primary, theme2.primary);
        assert_eq!(theme1.secondary, theme2.secondary);
        assert_eq!(theme1.accent, theme2.accent);
    }
}
