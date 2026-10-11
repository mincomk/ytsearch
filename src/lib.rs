//! YouTube search, videos, channels, playlists and comments — no API key, no
//! YouTube Data API.
//!
//! ```no_run
//! # #[cfg(feature = "blocking")]
//! # fn run() -> Result<(), ytsearch::Error> {
//! use ytsearch::{Client, VideoId};
//!
//! let yt = Client::new()?;
//!
//! let mut pages = yt.search_pages("minecraft");
//! while let Some(page) = pages.next_page()? {
//!     for result in &page.results {
//!         println!("{}", result.title());
//!     }
//! }
//!
//! let id = "6sFi_F3DJX4".parse::<VideoId>()?;
//! let video = yt.video(&id)?;
//! println!("{}", video.title);
//! # Ok(())
//! # }
//! ```
//!
//! # Shape of the API
//!
//! * A [`Client`] (or [`AsyncClient`]) owns the HTTP transport and the default
//!   [`Locale`], so no call has to thread either of them through.
//! * Ids are types — [`VideoId`], [`ChannelId`], [`PlaylistId`],
//!   [`ChannelHandle`] — parsed once with the validation the endpoint needs,
//!   including "id or URL" inputs, so the string sniffing happens at the edge.
//! * Anything that continues is a cursor: [`SearchPages`](blocking::SearchPages),
//!   [`PlaylistPages`](blocking::PlaylistPages),
//!   [`CommentPages`](blocking::CommentPages) — `next()` returns
//!   `Ok(None)` when the feed ends, and the continuation token itself is
//!   nobody else's business.
//! * Options are builders ([`SearchQuery`]) rather than nested `Option`s.
//! * Counts, durations and dates are typed (`u64`, [`std::time::Duration`],
//!   `chrono::DateTime`) instead of raw page text.
//!
//! # Features
//!
//! | feature | what it compiles | entry point |
//! | --- | --- | --- |
//! | `blocking` (default) | the synchronous surface | [`Client`] |
//! | `async` | the asynchronous surface | [`AsyncClient`] |
//!
//! They are independent: `default-features = false, features = ["async"]` gives
//! you only the async surface.
//!
//! ```no_run
//! # #[cfg(feature = "async")]
//! # async fn run() -> Result<(), ytsearch::Error> {
//! use ytsearch::{AsyncClient, VideoId};
//!
//! let yt = AsyncClient::new()?;
//! let video = yt.video(&"6sFi_F3DJX4".parse::<VideoId>()?).await?;
//! println!("{} {} views", video.title, video.view_count.unwrap_or(0));
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]

#[cfg(all(not(feature = "blocking"), not(feature = "async")))]
compile_error!(
    "ytsearch needs a transport: enable the default `blocking` feature, the `async` feature, or both"
);

mod error;
mod ids;
mod json;
mod locale;
mod model;
mod parsing;
mod request;
mod step;

#[cfg(feature = "async")]
pub mod asynchronous;
#[cfg(feature = "blocking")]
pub mod blocking;

pub use crate::error::{Error, Result};
pub use crate::ids::{ChannelHandle, ChannelId, ChannelInput, PlaylistId, VideoId};
pub use crate::json::PathError;
pub use crate::locale::{Language, Locale, Region, SearchMode};
pub use crate::model::{
    Channel, ChannelDetail, ChannelLink, ChannelResult, Comment, CommentPage, Image, Images,
    PlaylistItem, PlaylistPage, PlaylistResult, SearchPage, SearchQuery, SearchResult, Suggestion,
    SuggestionItem, Video, VideoResult, VideoState,
};

#[cfg(feature = "async")]
pub use crate::asynchronous::AsyncClient;
#[cfg(feature = "blocking")]
pub use crate::blocking::Client;

/// The version of this crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
