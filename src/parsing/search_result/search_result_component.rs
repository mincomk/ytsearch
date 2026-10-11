//! Parsing rules carried over from yspy (see NOTICE).
//!
//! Python needs a common base class for `list[SearchResultComponent]`; Rust
//! needs an enum, so this is the enum the page parser hands back and the api
//! layer matches on.

use super::{ChannelComponent, PlaylistComponent, VideoComponent};

/// One parsed search result row.
#[derive(Debug, Clone)]
pub enum SearchResultComponent {
    Video(VideoComponent),
    Channel(ChannelComponent),
    Playlist(PlaylistComponent),
}

impl SearchResultComponent {
    /// `yspy.api.search.Search._component_key` — the key the localized and the
    /// english page are paired by (their rows are ranked differently).
    pub fn component_key(&self) -> String {
        match self {
            SearchResultComponent::Video(component) => format!("video:{}", component.id),
            SearchResultComponent::Channel(component) => format!("channel:{}", component.id),
            SearchResultComponent::Playlist(component) => format!("playlist:{}", component.id),
        }
    }

    /// The channel component, when this is one.
    pub fn as_channel(&self) -> Option<&ChannelComponent> {
        match self {
            SearchResultComponent::Channel(component) => Some(component),
            _ => None,
        }
    }
}

impl From<VideoComponent> for SearchResultComponent {
    fn from(component: VideoComponent) -> Self {
        SearchResultComponent::Video(component)
    }
}

impl From<ChannelComponent> for SearchResultComponent {
    fn from(component: ChannelComponent) -> Self {
        SearchResultComponent::Channel(component)
    }
}

impl From<PlaylistComponent> for SearchResultComponent {
    fn from(component: PlaylistComponent) -> Self {
        SearchResultComponent::Playlist(component)
    }
}
