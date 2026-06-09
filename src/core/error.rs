use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("mpv init error: {0}")]
    MpvInit(String),

    #[error("mpv command failed: {0}")]
    MpvCommand(String),

    #[error("yt-dlp failed: {0}")]
    YtdlFailed(String),

    #[error("config path error: {0}")]
    ConfigPath(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;