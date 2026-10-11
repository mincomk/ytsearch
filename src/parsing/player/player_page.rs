//! Parsing rules carried over from yspy (see NOTICE).

use chrono::{DateTime, FixedOffset};
use serde_json::Value;

use crate::error::{Error, Result};
use crate::jpath;
use crate::json::{bool_at, get_by_path, get_by_path_or, i64_at, string_at};
use crate::model::Image;

/// the `playabilityStatus.status`
/// tokens of a player response.
///
/// yspy's members are spelled `OK`, `UNPLAYABLE`, …; the Rust variants are the
/// same tokens in Rust casing, and [`PlayerState::as_str`] gives the token back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerState {
    /// `PlayerState.OK`
    Ok,
    /// `PlayerState.UNPLAYABLE`
    Unplayable,
    /// `PlayerState.LOGIN_REQUIRED`
    LoginRequired,
    /// `PlayerState.AGE_CHECK_REQUIRED`
    AgeCheckRequired,
    /// `PlayerState.LIVE_STREAM`
    LiveStream,
    /// `PlayerState.LIVE_STREAM_OFFLINE`
    LiveStreamOffline,
    /// `PlayerState.ERROR`
    Error,
    /// `PlayerState.UNKNOWN`
    Unknown,
}

impl PlayerState {
    /// The `playabilityStatus.status` token (`PlayerState::Ok == "OK"`).
    pub const fn as_str(self) -> &'static str {
        match self {
            PlayerState::Ok => "OK",
            PlayerState::Unplayable => "UNPLAYABLE",
            PlayerState::LoginRequired => "LOGIN_REQUIRED",
            PlayerState::AgeCheckRequired => "AGE_CHECK_REQUIRED",
            PlayerState::LiveStream => "LIVE_STREAM",
            PlayerState::LiveStreamOffline => "LIVE_STREAM_OFFLINE",
            PlayerState::Error => "ERROR",
            PlayerState::Unknown => "UNKNOWN",
        }
    }

    /// `PlayerState(status_token)` — a token that is not in the table is
    /// `PlayerState::Unknown` (yspy catches the `ValueError` of its str-enum).
    pub fn from_token(token: &str) -> Self {
        match token {
            "OK" => PlayerState::Ok,
            "UNPLAYABLE" => PlayerState::Unplayable,
            "LOGIN_REQUIRED" => PlayerState::LoginRequired,
            "AGE_CHECK_REQUIRED" => PlayerState::AgeCheckRequired,
            "LIVE_STREAM" => PlayerState::LiveStream,
            "LIVE_STREAM_OFFLINE" => PlayerState::LiveStreamOffline,
            "ERROR" => PlayerState::Error,
            _ => PlayerState::Unknown,
        }
    }
}

impl std::fmt::Display for PlayerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// the availability
/// verdict of a player response: the state, plus the reason the player gave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerAvailability {
    pub state: PlayerState,
    pub reason: Option<String>,
}

impl PlayerAvailability {
    /// `PlayerAvailability(PlayerState.UNKNOWN)`
    pub fn unknown() -> Self {
        PlayerAvailability {
            state: PlayerState::Unknown,
            reason: None,
        }
    }
}

/// `yspy.data_parsing.player.player_page.PlayerPage`
#[derive(Debug, Clone)]
pub struct PlayerPage {
    pub id: String,
    pub title: String,
    pub url: String,
    pub length_seconds: i64,
    pub view_count: i64,
    pub thumbnails: Vec<Image>,
    pub description: String,
    pub channel_id: String,
    pub channel_name: String,
    pub is_live_content: bool,
    pub publish_date: DateTime<FixedOffset>,
    pub upload_date: DateTime<FixedOffset>,
    pub is_family_safe: bool,
    pub category: String,
}

impl PlayerPage {
    /// `PlayerPage._parse_availability(raw_data)`
    pub fn parse_availability(raw_data: &Value) -> PlayerAvailability {
        let playability = match get_by_path_or(raw_data, jpath!("playabilityStatus"))
            .and_then(Value::as_object)
        {
            Some(playability) => playability,
            None => return PlayerAvailability::unknown(),
        };

        let state = playability
            .get("status")
            .and_then(Value::as_str)
            .map(PlayerState::from_token)
            .unwrap_or(PlayerState::Unknown);

        let reason = playability
            .get("reason")
            .and_then(Value::as_str)
            .filter(|reason| !reason.is_empty())
            .map(str::to_string)
            .or_else(|| {
                playability
                    .get("messages")
                    .and_then(Value::as_array)
                    .and_then(|messages| messages.first())
                    .and_then(Value::as_str)
                    .map(str::to_string)
            });

        PlayerAvailability { state, reason }
    }

    /// `PlayerPage.from_json(raw_data)` — the availability verdict is decided
    /// from the `playabilityStatus` alone, so an unavailable video still
    /// reports the reason of its failure instead of raising.
    pub fn from_json(raw_data: &Value) -> Result<(Option<PlayerPage>, PlayerAvailability)> {
        let availability = PlayerPage::parse_availability(raw_data);

        let video_details = match raw_data.get("videoDetails") {
            Some(video_details) => video_details,
            None => {
                if availability.state == PlayerState::Unknown {
                    return Err(Error::parse("Given data is not a player page data"));
                }
                return Ok((None, availability));
            }
        };

        let microformat =
            match get_by_path(raw_data, jpath!("microformat playerMicroformatRenderer")) {
                Ok(microformat) => microformat,
                Err(_) => {
                    if availability.state == PlayerState::Unknown {
                        return Err(Error::parse("Given data is not a player page data"));
                    }
                    return Ok((None, availability));
                }
            };

        let parsed = (|| -> Result<PlayerPage> {
            let publish_date_text = string_at(microformat, jpath!("publishDate"))?;

            Ok(PlayerPage {
                id: string_at(video_details, jpath!("videoId"))?,
                title: string_at(video_details, jpath!("title"))?,
                url: string_at(microformat, jpath!("canonicalUrl"))?,
                length_seconds: i64_at(video_details, jpath!("lengthSeconds"))?,
                view_count: i64_at(video_details, jpath!("viewCount"))?,
                thumbnails: Image::list_from_json(get_by_path(
                    video_details,
                    jpath!("thumbnail thumbnails"),
                )?)?,
                description: string_at(video_details, jpath!("shortDescription"))?,
                channel_id: string_at(video_details, jpath!("channelId"))?,
                channel_name: string_at(microformat, jpath!("ownerChannelName"))?,
                is_live_content: bool_at(video_details, jpath!("isLiveContent"))?,
                publish_date: super::parse_iso_datetime(&publish_date_text)?,
                // yspy reads 'publishDate' for both dates — kept identical here
                upload_date: super::parse_iso_datetime(&publish_date_text)?,
                is_family_safe: bool_at(microformat, jpath!("isFamilySafe"))?,
                category: string_at(microformat, jpath!("category"))?,
            })
        })();

        match parsed {
            Ok(page) => Ok((Some(page), availability)),
            // partial videoDetails (e.g. members-only videos omit view_count) —
            // the availability verdict still carries the reason
            Err(_) => Ok((None, availability)),
        }
    }
}
