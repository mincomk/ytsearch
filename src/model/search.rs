//! Search: the query, the page, and the three kinds of result.

use std::collections::HashMap;
use std::time::Duration;

use crate::ids::{ChannelId, PlaylistId, VideoId};
use crate::locale::{Locale, SearchMode};
use crate::model::Image;
use crate::parsing::counts::{parse_duration, parse_subscriber_count, parse_view_count};
use crate::parsing::search_result::{
    ChannelComponent, PlaylistComponent, SearchResultComponent, SearchResultPage, VideoComponent,
};

/// What to search for.
///
/// ```
/// # use ytsearch::{SearchMode, SearchQuery};
/// let query = SearchQuery::new("minecraft").mode(SearchMode::Video);
/// assert_eq!(query.query, "minecraft");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery {
    pub query: String,
    /// `None` searches everything; `Some(mode)` filters.
    pub mode: Option<SearchMode>,
    /// `None` uses the client's locale.
    pub locale: Option<Locale>,
}

impl SearchQuery {
    /// A query with no filter and the client's locale.
    pub fn new(query: impl Into<String>) -> Self {
        SearchQuery {
            query: query.into(),
            mode: None,
            locale: None,
        }
    }

    /// Filter the search to one kind of result.
    pub fn mode(mut self, mode: SearchMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Search in a different locale than the client's.
    pub fn locale(mut self, locale: Locale) -> Self {
        self.locale = Some(locale);
        self
    }
}

impl From<&str> for SearchQuery {
    fn from(query: &str) -> Self {
        SearchQuery::new(query)
    }
}

impl From<String> for SearchQuery {
    fn from(query: String) -> Self {
        SearchQuery::new(query)
    }
}

impl From<(&str, SearchMode)> for SearchQuery {
    fn from((query, mode): (&str, SearchMode)) -> Self {
        SearchQuery::new(query).mode(mode)
    }
}

impl From<&SearchQuery> for SearchQuery {
    fn from(query: &SearchQuery) -> Self {
        query.clone()
    }
}

/// One page of search results.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SearchPage {
    pub results: Vec<SearchResult>,
}

impl SearchPage {
    /// Pair this page's rows with the English page's.
    ///
    /// The two pages rank their rows differently (ads drop, ranking shifts), so
    /// rows are paired by their own id rather than by position.
    pub(crate) fn from_parsed(page: &SearchResultPage, english: &SearchResultPage) -> Self {
        let english_by_key: HashMap<String, &SearchResultComponent> = english
            .components
            .iter()
            .map(|component| (component.component_key(), component))
            .collect();

        let results = page
            .components
            .iter()
            .map(|component| {
                let counterpart = english_by_key.get(&component.component_key()).copied();
                SearchResult::from_component(component, counterpart)
            })
            .collect();

        SearchPage { results }
    }

    /// Every video result, in page order.
    pub fn videos(&self) -> impl Iterator<Item = &VideoResult> {
        self.results.iter().filter_map(SearchResult::as_video)
    }

    /// Every channel result, in page order.
    pub fn channels(&self) -> impl Iterator<Item = &ChannelResult> {
        self.results.iter().filter_map(SearchResult::as_channel)
    }

    /// Every playlist result, in page order.
    pub fn playlists(&self) -> impl Iterator<Item = &PlaylistResult> {
        self.results.iter().filter_map(SearchResult::as_playlist)
    }
}

/// One row of a search result page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchResult {
    Video(VideoResult),
    Channel(ChannelResult),
    Playlist(PlaylistResult),
}

impl SearchResult {
    fn from_component(
        component: &SearchResultComponent,
        english: Option<&SearchResultComponent>,
    ) -> Self {
        match component {
            SearchResultComponent::Video(component) => {
                SearchResult::Video(VideoResult::from_component(component))
            }
            SearchResultComponent::Channel(component) => {
                let english = english.and_then(SearchResultComponent::as_channel);
                SearchResult::Channel(ChannelResult::from_component(component, english))
            }
            SearchResultComponent::Playlist(component) => {
                SearchResult::Playlist(PlaylistResult::from_component(component))
            }
        }
    }

    /// The row's id.
    pub fn id(&self) -> &str {
        match self {
            SearchResult::Video(result) => result.id.as_str(),
            SearchResult::Channel(result) => result.id.as_str(),
            SearchResult::Playlist(result) => result.id.as_str(),
        }
    }

    /// The row's title.
    pub fn title(&self) -> &str {
        match self {
            SearchResult::Video(result) => &result.title,
            SearchResult::Channel(result) => &result.title,
            SearchResult::Playlist(result) => &result.title,
        }
    }

    /// The video result, when this row is one.
    pub fn as_video(&self) -> Option<&VideoResult> {
        match self {
            SearchResult::Video(result) => Some(result),
            _ => None,
        }
    }

    /// The channel result, when this row is one.
    pub fn as_channel(&self) -> Option<&ChannelResult> {
        match self {
            SearchResult::Channel(result) => Some(result),
            _ => None,
        }
    }

    /// The playlist result, when this row is one.
    pub fn as_playlist(&self) -> Option<&PlaylistResult> {
        match self {
            SearchResult::Playlist(result) => Some(result),
            _ => None,
        }
    }
}

/// A video row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoResult {
    pub id: VideoId,
    pub title: String,
    pub url: String,
    pub thumbnails: Vec<Image>,
    /// A Short — the page links it to the shorts player.
    pub is_shorts: bool,
    /// How long ago it was published, as the page words it (`1 day ago`).
    pub published_time_text: Option<String>,
    pub length: Option<Duration>,
    pub view_count: Option<u64>,
    pub channel_id: ChannelId,
    pub channel_url: String,
}

impl VideoResult {
    fn from_component(component: &VideoComponent) -> Self {
        VideoResult {
            id: VideoId::new_unchecked(component.id.clone()),
            title: component.title.clone(),
            url: component.url.clone(),
            thumbnails: component.thumbnails.clone(),
            is_shorts: component.is_shorts,
            published_time_text: component.published_time_text.clone(),
            length: parse_duration(component.length_text.as_deref()),
            view_count: parse_view_count(component.view_count_text.as_deref()),
            channel_id: ChannelId::new_unchecked(component.channel_id.clone()),
            channel_url: component.channel_url.clone(),
        }
    }
}

/// A channel row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelResult {
    pub id: ChannelId,
    pub title: String,
    pub url: String,
    pub thumbnails: Vec<Image>,
    pub description: Option<String>,
    pub subscriber_count: Option<u64>,
}

impl ChannelResult {
    fn from_component(component: &ChannelComponent, english: Option<&ChannelComponent>) -> Self {
        ChannelResult {
            id: ChannelId::new_unchecked(component.id.clone()),
            title: component.title.clone(),
            url: component.url.clone(),
            thumbnails: component.thumbnails.clone(),
            description: component.description_snippet.clone(),
            subscriber_count: english
                .and_then(|english| parse_subscriber_count(Some(&english.subscribers_count_text))),
        }
    }
}

/// A playlist row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistResult {
    pub id: PlaylistId,
    pub title: String,
    pub url: String,
    pub thumbnails: Vec<Image>,
    pub channel_id: Option<ChannelId>,
    pub channel_name: Option<String>,
}

impl PlaylistResult {
    fn from_component(component: &PlaylistComponent) -> Self {
        PlaylistResult {
            id: PlaylistId::new_unchecked(component.id.clone()),
            title: component.title.clone(),
            url: component.url.clone(),
            thumbnails: component.thumbnails.clone(),
            channel_id: component
                .channel_id
                .as_ref()
                .map(|id| ChannelId::new_unchecked(id.clone())),
            channel_name: component.channel_name.clone(),
        }
    }
}
