//! Typed ids — parsed once, at the edge, so nothing downstream has to guess.

use std::borrow::Borrow;
use std::fmt;
use std::str::FromStr;

use url::Url;

use crate::error::{Error, Result};

/// A video id: the eleven characters in `watch?v=…`.
///
/// [`FromStr`] accepts the bare id or any URL that carries one in its `v`
/// parameter.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VideoId(String);

impl VideoId {
    /// The length of a bare video id.
    pub const LENGTH: usize = 11;

    /// Wrap a string that is already a video id, without checking it.
    pub fn new_unchecked(id: impl Into<String>) -> Self {
        VideoId(id.into())
    }

    /// The id itself.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The id itself, owned.
    pub fn into_string(self) -> String {
        self.0
    }
}

impl FromStr for VideoId {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        let value = value.trim();

        if value.len() == VideoId::LENGTH
            && value
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Ok(VideoId(value.to_string()));
        }

        if let Ok(url) = Url::parse(value) {
            if let Some((_, id)) = url.query_pairs().find(|(key, _)| key == "v") {
                if !id.is_empty() {
                    return Ok(VideoId(id.into_owned()));
                }
            }
        }

        Err(Error::InvalidVideoId(value.to_string()))
    }
}

impl TryFrom<&str> for VideoId {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        value.parse()
    }
}

impl TryFrom<String> for VideoId {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        value.parse()
    }
}

impl AsRef<str> for VideoId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for VideoId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for VideoId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A channel id: the `UC…` identifier.
///
/// [`FromStr`] accepts the bare id, a `…/channel/UC…` URL, or a URL whose
/// `browseId` is one. A `@handle` is *not* an id and has no id until it is
/// resolved — use [`ChannelHandle`] and
/// [`Client::channel`](crate::Client::channel).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChannelId(String);

impl ChannelId {
    /// Wrap a string that is already a channel id, without checking it.
    pub fn new_unchecked(id: impl Into<String>) -> Self {
        ChannelId(id.into())
    }

    /// The id itself.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The id itself, owned.
    pub fn into_string(self) -> String {
        self.0
    }
}

impl FromStr for ChannelId {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        let value = value.trim();

        if value.starts_with("UC") && value.len() > 2 {
            return Ok(ChannelId(value.to_string()));
        }

        if let Ok(url) = Url::parse(value) {
            if let Some(segment) = url.path_segments().and_then(|segments| {
                segments
                    .into_iter()
                    .find(|segment| segment.starts_with("UC"))
            }) {
                return Ok(ChannelId(segment.to_string()));
            }
            if let Some((_, id)) = url
                .query_pairs()
                .find(|(key, value)| key == "browseId" && value.starts_with("UC"))
            {
                return Ok(ChannelId(id.into_owned()));
            }
        }

        Err(Error::InvalidChannelId(value.to_string()))
    }
}

impl TryFrom<&str> for ChannelId {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        value.parse()
    }
}

impl TryFrom<String> for ChannelId {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        value.parse()
    }
}

impl AsRef<str> for ChannelId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for ChannelId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ChannelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A channel handle such as `@sleepground`, **without** the `@`.
///
/// Resolving a handle to a [`ChannelId`] costs a request, which is why the two
/// are different types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChannelHandle(String);

impl ChannelHandle {
    /// Wrap a string that is already a handle, without checking it.
    pub fn new_unchecked(handle: impl Into<String>) -> Self {
        ChannelHandle(handle.into())
    }

    /// The handle without its `@`.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The handle, owned, without its `@`.
    pub fn into_string(self) -> String {
        self.0
    }
}

impl FromStr for ChannelHandle {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        let value = value.trim();

        let handle = match Url::parse(value) {
            // a url: the handle is its first path segment
            Ok(url) => url
                .path_segments()
                .and_then(|mut segments| segments.next().map(str::to_string))
                .unwrap_or_default()
                .trim_start_matches('@')
                .to_string(),
            Err(_) => value.trim_start_matches('@').to_string(),
        };

        let valid = !handle.is_empty()
            && handle
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-');

        if valid {
            Ok(ChannelHandle(handle))
        } else {
            Err(Error::InvalidChannelId(value.to_string()))
        }
    }
}

impl TryFrom<&str> for ChannelHandle {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        value.parse()
    }
}

impl AsRef<str> for ChannelHandle {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ChannelHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}", self.0)
    }
}

/// A playlist id: `PL…`, `UU…` (a channel's uploads) or `VL…`.
///
/// [`FromStr`] accepts the bare id or any URL carrying one in its `list`
/// parameter.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PlaylistId(String);

impl PlaylistId {
    /// Wrap a string that is already a playlist id, without checking it.
    pub fn new_unchecked(id: impl Into<String>) -> Self {
        PlaylistId(id.into())
    }

    /// The id itself.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The id itself, owned.
    pub fn into_string(self) -> String {
        self.0
    }

    /// The browse id the playlist endpoint takes: `VL` + the id.
    pub(crate) fn browse_id(&self) -> String {
        if self.0.starts_with("VL") {
            self.0.clone()
        } else {
            format!("VL{}", self.0)
        }
    }
}

impl FromStr for PlaylistId {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        let value = value.trim();

        if value.starts_with("PL") || value.starts_with("UU") || value.starts_with("VL") {
            return Ok(PlaylistId(value.to_string()));
        }

        if let Ok(url) = Url::parse(value) {
            if let Some((_, id)) = url.query_pairs().find(|(key, _)| key == "list") {
                if !id.is_empty() {
                    return Ok(PlaylistId(id.into_owned()));
                }
            }
        }

        Err(Error::InvalidPlaylistId(value.to_string()))
    }
}

impl TryFrom<&str> for PlaylistId {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        value.parse()
    }
}

impl TryFrom<String> for PlaylistId {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        value.parse()
    }
}

impl AsRef<str> for PlaylistId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for PlaylistId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PlaylistId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// What a channel fetch accepts.
///
/// ```
/// # use ytsearch::{ChannelHandle, ChannelId, ChannelInput};
/// # use std::str::FromStr;
/// let id = ChannelId::from_str("UCg7rkxrTnIhiHEpXY1ec9NA").unwrap();
/// let handle = ChannelHandle::from_str("@sleepground").unwrap();
/// let from_id: ChannelInput = (&id).into();
/// let from_handle: ChannelInput = (&handle).into();
/// let from_raw: ChannelInput = "@sleepground".into();
/// # let _ = (from_id, from_handle, from_raw);
/// ```
#[derive(Debug, Clone, Copy)]
pub enum ChannelInput<'a> {
    /// A channel id — no resolution needed.
    Id(&'a ChannelId),
    /// A `@handle` — resolved with one extra request.
    Handle(&'a ChannelHandle),
    /// An id, a handle or a URL, worked out at call time.
    Raw(&'a str),
}

impl<'a> From<&'a ChannelId> for ChannelInput<'a> {
    fn from(id: &'a ChannelId) -> Self {
        ChannelInput::Id(id)
    }
}

impl<'a> From<&'a ChannelHandle> for ChannelInput<'a> {
    fn from(handle: &'a ChannelHandle) -> Self {
        ChannelInput::Handle(handle)
    }
}

impl<'a> From<&'a str> for ChannelInput<'a> {
    fn from(raw: &'a str) -> Self {
        ChannelInput::Raw(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn video_ids_parse_ids_and_urls() {
        assert_eq!(
            "6sFi_F3DJX4".parse::<VideoId>().unwrap().as_str(),
            "6sFi_F3DJX4"
        );
        assert_eq!(
            "https://www.youtube.com/watch?v=6sFi_F3DJX4&t=1"
                .parse::<VideoId>()
                .unwrap()
                .as_str(),
            "6sFi_F3DJX4"
        );
        assert!("dQw4w9WgXcQ0".parse::<VideoId>().is_err()); // twelve characters
        assert!("not a video".parse::<VideoId>().is_err());
    }

    #[test]
    fn channel_ids_and_handles_are_different_types() {
        assert_eq!(
            "UCg7rkxrTnIhiHEpXY1ec9NA"
                .parse::<ChannelId>()
                .unwrap()
                .as_str(),
            "UCg7rkxrTnIhiHEpXY1ec9NA"
        );
        assert_eq!(
            "https://www.youtube.com/channel/UCg7rkxrTnIhiHEpXY1ec9NA"
                .parse::<ChannelId>()
                .unwrap()
                .as_str(),
            "UCg7rkxrTnIhiHEpXY1ec9NA"
        );
        // a handle is not an id
        assert!("@sleepground".parse::<ChannelId>().is_err());

        let handle: ChannelHandle = "@sleepground".parse().unwrap();
        assert_eq!(handle.as_str(), "sleepground");
        assert_eq!(handle.to_string(), "@sleepground");
        assert_eq!(
            "https://www.youtube.com/@sleepground"
                .parse::<ChannelHandle>()
                .unwrap()
                .as_str(),
            "sleepground"
        );
    }

    #[test]
    fn playlist_ids_parse_urls_and_normalize_to_a_browse_id() {
        assert_eq!(
            "UUDrAR1OWC2MD4s0JLetN0MA"
                .parse::<PlaylistId>()
                .unwrap()
                .browse_id(),
            "VLUUDrAR1OWC2MD4s0JLetN0MA"
        );
        assert_eq!(
            "VLPL_DD814i0-aIqCZe6S78WvbFvDB3E5y4z"
                .parse::<PlaylistId>()
                .unwrap()
                .browse_id(),
            "VLPL_DD814i0-aIqCZe6S78WvbFvDB3E5y4z"
        );
        assert_eq!(
            "https://www.youtube.com/playlist?list=PL_DD814i0-aIqCZe6S78WvbFvDB3E5y4z"
                .parse::<PlaylistId>()
                .unwrap()
                .as_str(),
            "PL_DD814i0-aIqCZe6S78WvbFvDB3E5y4z"
        );
        assert!("nonsense".parse::<PlaylistId>().is_err());
    }
}
