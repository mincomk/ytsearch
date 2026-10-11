//! Everything that can go wrong.

use crate::json::PathError;
use crate::model::VideoState;

/// The result type of every fallible call in this crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Everything that can go wrong.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The input was neither a video id nor a URL carrying one.
    #[error("not a video id or video url: {0:?}")]
    InvalidVideoId(String),

    /// The input was neither a channel id nor a URL or a handle carrying one.
    #[error("not a channel id or channel url: {0:?}")]
    InvalidChannelId(String),

    /// The input was neither a playlist id nor a URL carrying one.
    #[error("not a playlist id or playlist url: {0:?}")]
    InvalidPlaylistId(String),

    /// The channel page came back with an error alert.
    #[error("the channel does not exist")]
    ChannelNotFound,

    /// The playlist page came back with an error alert.
    #[error("the playlist does not exist")]
    PlaylistNotFound,

    /// The video exists but cannot be watched — the state says why.
    #[error("the video is not available ({0})")]
    VideoUnavailable(VideoState),

    /// The response was not the shape it should have been.
    #[error("the response could not be parsed: {0}")]
    Parse(String),

    /// A json path did not resolve.
    #[error(transparent)]
    Path(#[from] PathError),

    /// The request itself failed.
    #[error(transparent)]
    Http(#[from] reqwest::Error),

    /// A response body was not json.
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// A URL could not be parsed or built.
    #[error(transparent)]
    Url(#[from] url::ParseError),
}

impl Error {
    /// A parsing failure, with the message the page parser wanted to give.
    pub(crate) fn parse(message: impl Into<String>) -> Self {
        Error::Parse(message.into())
    }
}
