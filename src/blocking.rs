//! The synchronous client.
//!
//! ```no_run
//! # fn main() -> Result<(), ytsearch::Error> {
//! use std::str::FromStr;
//!
//! use ytsearch::{Client, VideoId};
//!
//! let yt = Client::new()?;
//! let video = yt.video(&VideoId::from_str("6sFi_F3DJX4")?)?;
//! println!("{} — {} views", video.title, video.view_count.unwrap_or(0));
//! # Ok(())
//! # }
//! ```
//!
//! The blocking transport owns a runtime of its own, so a [`Client`] must not be
//! used from inside an async runtime — use
//! [`AsyncClient`](crate::AsyncClient) there. (This is why the two surfaces are
//! separate features: a program that only wants the async one never links the
//! blocking client at all.)

use std::time::Duration;

use reqwest::header::{HeaderName, HeaderValue};
use serde_json::Value;

use crate::error::{Error, Result};
use crate::ids::{ChannelId, ChannelInput, PlaylistId, VideoId};
use crate::jpath;
use crate::json::str_at_or;
use crate::locale::Locale;
use crate::model::{
    Channel, ChannelDetail, CommentPage, PlaylistPage, SearchPage, SearchQuery, Suggestion, Video,
    VideoState,
};
use crate::parsing::channel::{ChannelDetailPage, ChannelPage};
use crate::parsing::playlist::PlaylistPage as ParsedPlaylistPage;
use crate::parsing::search_result::SearchResultPage;
use crate::parsing::suggestion_data::SuggestionData;
use crate::parsing::CommentsPage as ParsedCommentsPage;
use crate::request::{self, Request};
use crate::step::{self, PlayerOutcome};

/// The timeout every request gets unless the builder says otherwise.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// A synchronous client: the transport and the default locale in one handle.
pub struct Client {
    http: reqwest::blocking::Client,
    locale: Locale,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("locale", &self.locale)
            .finish_non_exhaustive()
    }
}

impl Client {
    /// A client with the defaults: a five second timeout, no redirect
    /// following, and [`Locale::NONE`].
    pub fn new() -> Result<Self> {
        ClientBuilder::default().build()
    }

    /// A builder, for a different timeout, redirect policy or locale.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// The locale this client asks in.
    pub fn locale(&self) -> Locale {
        self.locale
    }

    /// The same client, asking in a different locale.
    pub fn with_locale(mut self, locale: Locale) -> Self {
        self.locale = locale;
        self
    }

    /// The pages of a search.
    ///
    /// The first page is fetched by [`SearchPages::next_page`], like every
    /// other page.
    pub fn search_pages(&self, query: impl Into<SearchQuery>) -> SearchPages<'_> {
        let query = query.into();
        let locale = query.locale.unwrap_or(self.locale);

        SearchPages {
            client: self,
            query,
            locale,
            continuation: None,
            finished: false,
        }
    }

    /// A video, by id.
    ///
    /// An unavailable video is an [`Error::VideoUnavailable`] carrying the
    /// state; [`Client::video_state`] asks for the verdict alone.
    pub fn video(&self, id: &VideoId) -> Result<Video> {
        match self.probe(id)? {
            PlayerOutcome::Available(page) => Ok(Video::from_page(*page)),
            PlayerOutcome::Unavailable(state) => Err(Error::VideoUnavailable(state)),
        }
    }

    /// Whether a video can be watched, and if not, why.
    pub fn video_state(&self, id: &VideoId) -> Result<VideoState> {
        Ok(match self.probe(id)? {
            PlayerOutcome::Available(_) => VideoState::Ok,
            PlayerOutcome::Unavailable(state) => state,
        })
    }

    /// A channel, by id, handle or URL.
    ///
    /// A handle costs one extra request: it has to be resolved to an id first.
    pub fn channel<'a>(&self, channel: impl Into<ChannelInput<'a>>) -> Result<Channel> {
        let id = match channel.into() {
            ChannelInput::Id(id) => id.clone(),
            ChannelInput::Handle(handle) => self.resolve_channel_id(handle.as_ref())?,
            ChannelInput::Raw(raw) => match raw.parse::<ChannelId>() {
                Ok(id) => id,
                Err(_) => self.resolve_channel_id(raw)?,
            },
        };

        let request = request::channel_request(id.as_str(), self.locale);
        let (page, english) = step::drive(
            step::localized(request, self.locale, || {
                request::channel_request(id.as_str(), Locale::ENGLISH)
            }),
            |request| self.send(request),
        )?;

        let page = ChannelPage::from_json(&page)?.ok_or(Error::ChannelNotFound)?;
        let english = ChannelPage::from_json(&english)?.unwrap_or_else(|| page.clone());

        Ok(Channel::from_page(page, &english))
    }

    /// A channel's about page.
    ///
    /// The token it needs comes from the channel page itself, so the channel
    /// has to have been fetched first — that is the whole reason this takes a
    /// [`Channel`] and not an id.
    pub fn channel_detail(&self, channel: &Channel) -> Result<ChannelDetail> {
        let token = channel
            .about_token
            .as_deref()
            .ok_or_else(|| Error::parse("this channel page carried no about-page token"))?;

        let (page, english) = step::drive(
            step::localized(
                request::channel_detail_request(token, self.locale),
                self.locale,
                || request::channel_detail_request(token, Locale::ENGLISH),
            ),
            |request| self.send(request),
        )?;

        let page = ChannelDetailPage::from_json(&page)?;
        let english = ChannelDetailPage::from_json(&english).unwrap_or_else(|_| page.clone());

        Ok(ChannelDetail::from_page(page, &english))
    }

    /// The pages of a playlist.
    pub fn playlist_pages(&self, id: &PlaylistId, locale: Option<Locale>) -> PlaylistPages<'_> {
        PlaylistPages {
            client: self,
            id: id.clone(),
            locale: locale.unwrap_or(self.locale),
            continuation: None,
            page_index: 0,
            index_offset: 0,
            finished: false,
        }
    }

    /// The pages of a video's comments.
    pub fn comment_pages(&self, video: &VideoId) -> CommentPages<'_> {
        CommentPages {
            client: self,
            id: video.clone(),
            continuation: None,
            started: false,
            finished: false,
        }
    }

    /// What YouTube suggests for a query.
    pub fn suggestions(&self, query: &str) -> Result<Suggestion> {
        let text = self.send_text(request::suggestion_request(query, self.locale))?;
        let data = SuggestionData::from_json(&SuggestionData::unwrap_parentheses(&text)?)?;

        Ok(Suggestion::from(data))
    }

    fn probe(&self, id: &VideoId) -> Result<PlayerOutcome> {
        step::drive(step::video_probe(id, self.locale), |request| {
            self.send(request)
        })
    }

    fn resolve_channel_id(&self, input: &str) -> Result<ChannelId> {
        let value = self.send(request::resolve_channel_request(input))?;

        str_at_or(&value, jpath!("endpoint browseEndpoint browseId"))
            .and_then(|id| id.parse::<ChannelId>().ok())
            .ok_or(Error::ChannelNotFound)
    }

    fn send(&self, request: Request) -> Result<Value> {
        let response = self
            .http
            .request(request.method.as_reqwest(), &request.url)
            .headers(header_map(&request)?)
            .json_opt(request.body.as_ref())
            .send()?;

        Ok(response.json()?)
    }

    fn send_text(&self, request: Request) -> Result<String> {
        let response = self
            .http
            .request(request.method.as_reqwest(), &request.url)
            .headers(header_map(&request)?)
            .json_opt(request.body.as_ref())
            .send()?;

        Ok(response.text()?)
    }
}

/// A `RequestBuilder` with a body only when there is one.
trait JsonOpt {
    fn json_opt(self, body: Option<&Value>) -> Self;
}

impl JsonOpt for reqwest::blocking::RequestBuilder {
    fn json_opt(self, body: Option<&Value>) -> Self {
        match body {
            Some(body) => self.json(body),
            None => self,
        }
    }
}

fn header_map(request: &Request) -> Result<reqwest::header::HeaderMap> {
    let mut headers = reqwest::header::HeaderMap::new();
    for (name, value) in &request.headers {
        headers.insert(
            HeaderName::from_bytes(name.as_bytes())
                .map_err(|error| Error::parse(format!("invalid header name: {error}")))?,
            HeaderValue::from_str(value)
                .map_err(|error| Error::parse(format!("invalid header value: {error}")))?,
        );
    }

    Ok(headers)
}

/// Builds a [`Client`].
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    timeout: Duration,
    follow_redirects: bool,
    locale: Locale,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        ClientBuilder {
            timeout: DEFAULT_TIMEOUT,
            follow_redirects: false,
            locale: Locale::NONE,
        }
    }
}

impl ClientBuilder {
    /// How long a request may take in total. Default: five seconds.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Whether to follow redirects. Default: no, which is what `httpx` does.
    pub fn follow_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = follow;
        self
    }

    /// The locale every call uses unless the call overrides it.
    pub fn locale(mut self, locale: Locale) -> Self {
        self.locale = locale;
        self
    }

    /// Build the client.
    pub fn build(self) -> Result<Client> {
        let redirect = if self.follow_redirects {
            reqwest::redirect::Policy::limited(10)
        } else {
            reqwest::redirect::Policy::none()
        };

        let http = reqwest::blocking::Client::builder()
            .timeout(self.timeout)
            .redirect(redirect)
            .build()?;

        Ok(Client {
            http,
            locale: self.locale,
        })
    }
}

/// The pages of a search.
pub struct SearchPages<'a> {
    client: &'a Client,
    query: SearchQuery,
    locale: Locale,
    continuation: Option<String>,
    finished: bool,
}

impl SearchPages<'_> {
    /// The next page, or `Ok(None)` when the results run out.
    ///
    /// This is the call that fetches the *first* page, too.
    pub fn next_page(&mut self) -> Result<Option<SearchPage>> {
        if self.finished {
            return Ok(None);
        }

        let (query, mode, locale, continuation) = (
            &self.query.query,
            self.query.mode,
            self.locale,
            self.continuation.clone(),
        );

        let (page, english) = step::drive(
            step::localized(
                request::search_request(query, mode, locale, continuation.as_deref()),
                locale,
                || request::search_request(query, mode, Locale::ENGLISH, continuation.as_deref()),
            ),
            |request| self.client.send(request),
        )?;

        let page = SearchResultPage::from_json(&page)?;
        let english = SearchResultPage::from_json(&english).unwrap_or_else(|_| page.clone());

        self.continuation = page.continuation_token.clone();
        if self.continuation.is_none() {
            self.finished = true;
        }

        Ok(Some(SearchPage::from_parsed(&page, &english)))
    }
}

impl Iterator for SearchPages<'_> {
    type Item = Result<SearchPage>;

    /// [`SearchPages::next_page`], as an iterator — an error ends the walk.
    fn next(&mut self) -> Option<Self::Item> {
        match self.next_page() {
            Ok(Some(page)) => Some(Ok(page)),
            Ok(None) => None,
            Err(error) => {
                self.finished = true;
                Some(Err(error))
            }
        }
    }
}

/// The pages of a playlist.
pub struct PlaylistPages<'a> {
    client: &'a Client,
    id: PlaylistId,
    locale: Locale,
    continuation: Option<String>,
    page_index: u32,
    index_offset: u32,
    finished: bool,
}

impl PlaylistPages<'_> {
    /// The next page, or `Ok(None)` when the playlist runs out.
    pub fn next_page(&mut self) -> Result<Option<PlaylistPage>> {
        if self.finished {
            return Ok(None);
        }

        let first = self.continuation.is_none();
        let (browse_id, locale, continuation) =
            (self.id.browse_id(), self.locale, self.continuation.clone());

        let (page, english) = match continuation.as_deref() {
            Some(token) => step::drive(
                step::localized(
                    request::playlist_continuation_request(token, locale),
                    locale,
                    || request::playlist_continuation_request(token, Locale::ENGLISH),
                ),
                |request| self.client.send(request),
            )?,
            None => step::drive(
                step::localized(
                    request::playlist_request(&browse_id, locale),
                    locale,
                    || request::playlist_request(&browse_id, Locale::ENGLISH),
                ),
                |request| self.client.send(request),
            )?,
        };

        let parsed = ParsedPlaylistPage::from_json(&page)?;
        let parsed = match parsed {
            Some(parsed) => parsed,
            None if first => return Err(Error::PlaylistNotFound),
            None => {
                self.finished = true;
                return Ok(None);
            }
        };

        let english = ParsedPlaylistPage::from_json(&english)?.unwrap_or_else(|| parsed.clone());

        let page = crate::model::PlaylistPage::from_parsed(
            parsed.clone(),
            &english,
            self.page_index,
            self.index_offset,
        );

        self.continuation = parsed.continuation_token.clone();
        self.index_offset += parsed.videos.len() as u32;
        self.page_index += 1;
        if self.continuation.is_none() {
            self.finished = true;
        }

        Ok(Some(page))
    }
}

impl Iterator for PlaylistPages<'_> {
    type Item = Result<PlaylistPage>;

    /// [`PlaylistPages::next_page`], as an iterator — an error ends the walk.
    fn next(&mut self) -> Option<Self::Item> {
        match self.next_page() {
            Ok(Some(page)) => Some(Ok(page)),
            Ok(None) => None,
            Err(error) => {
                self.finished = true;
                Some(Err(error))
            }
        }
    }
}

/// The pages of a video's comments.
pub struct CommentPages<'a> {
    client: &'a Client,
    id: VideoId,
    continuation: Option<String>,
    started: bool,
    finished: bool,
}

impl CommentPages<'_> {
    /// The next batch of comments, or `Ok(None)` when the thread ends.
    pub fn next_page(&mut self) -> Result<Option<CommentPage>> {
        if self.finished {
            return Ok(None);
        }

        let parsed = match self.continuation.clone() {
            Some(token) => {
                let value = self.client.send(request::comments_request(&token))?;
                ParsedCommentsPage::from_json(&value)?
            }
            None if !self.started => {
                let parsed = step::drive(step::comments_first(&self.id), |request| {
                    self.client.send(request)
                })?;
                self.started = true;
                parsed
            }
            None => {
                self.finished = true;
                return Ok(None);
            }
        };

        let (page, continuation) = CommentPage::from_parsed(parsed);
        self.continuation = continuation;
        if self.continuation.is_none() {
            self.finished = true;
        }

        Ok(Some(page))
    }
}

impl Iterator for CommentPages<'_> {
    type Item = Result<CommentPage>;

    /// [`CommentPages::next_page`], as an iterator — an error ends the walk.
    fn next(&mut self) -> Option<Self::Item> {
        match self.next_page() {
            Ok(Some(page)) => Some(Ok(page)),
            Ok(None) => None,
            Err(error) => {
                self.finished = true;
                Some(Err(error))
            }
        }
    }
}
