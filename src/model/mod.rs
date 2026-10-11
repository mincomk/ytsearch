//! The types the client hands back: plain data, no I/O, no hidden state.

mod channel;
mod comment;
mod image;
mod playlist;
mod search;
mod suggestion;
mod video;

pub use channel::{Channel, ChannelDetail, ChannelLink};
pub use comment::{Comment, CommentPage};
pub use image::{Image, Images};
pub use playlist::{PlaylistItem, PlaylistPage};
pub use search::{
    ChannelResult, PlaylistResult, SearchPage, SearchQuery, SearchResult, VideoResult,
};
pub use suggestion::{Suggestion, SuggestionItem};
pub use video::{Video, VideoState};
