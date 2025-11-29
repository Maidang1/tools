mod app;
mod renderer;
mod event_handler;
mod navigation;
mod state;

pub use app::App;

#[derive(Debug)]
pub enum UiError {
    IoError(std::io::Error),
    TerminalError(String),
}

impl From<std::io::Error> for UiError {
    fn from(error: std::io::Error) -> Self {
        UiError::IoError(error)
    }
}

impl std::fmt::Display for UiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UiError::IoError(e) => write!(f, "IO Error: {}", e),
            UiError::TerminalError(e) => write!(f, "Terminal Error: {}", e),
        }
    }
}

impl std::error::Error for UiError {}