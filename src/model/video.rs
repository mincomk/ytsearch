//! A video, and the verdict on whether it can be watched.

use std::fmt;
use std::time::Duration;

use chrono::{DateTime, FixedOffset};

use crate::ids::{ChannelId, VideoId};
use crate::model::Image;
use crate::parsing::player::{PlayerAvailability, PlayerPage, PlayerState};

/// Why a video can or cannot be watched.
///
/// The verdict is classified from the player's own playability reason, and a
/// page that comes back unavailable is re-probed once with pot-free innertube
/// clients in an English locale so the reason is a stable one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum VideoState {
    /// Watchable.
    Ok,
    /// Members-only content.
    MembersOnly,
    /// A live stream whose recording is gone.
    RecordingUnavailable,
    /// Age-restricted.
    AgeRestricted,
    /// YouTube wants a signed-in human.
    BotDetection,
    /// Sign-in required.
    LoginRequired,
    /// Blocked in this region.
    RegionBlocked,
    /// Blocked on copyright grounds.
    CopyrightBlocked,
    /// Private.
    Private,
    /// Removed by the uploader.
    RemovedByUploader,
    /// The account was terminated.
    AccountTerminated,
    /// Removed for violating the terms.
    RemovedForTos,
    /// Unavailable, for a reason the table does not name.
    Unavailable,
}

impl VideoState {
    /// The state in yspy's spelling (`VideoState::MembersOnly == "MEMBERS_ONLY"`).
    pub const fn as_str(self) -> &'static str {
        match self {
            VideoState::Ok => "OK",
            VideoState::MembersOnly => "MEMBERS_ONLY",
            VideoState::RecordingUnavailable => "RECORDING_UNAVAILABLE",
            VideoState::AgeRestricted => "AGE_RESTRICTED",
            VideoState::BotDetection => "BOT_DETECTION",
            VideoState::LoginRequired => "LOGIN_REQUIRED",
            VideoState::RegionBlocked => "REGION_BLOCKED",
            VideoState::CopyrightBlocked => "COPYRIGHT_BLOCKED",
            VideoState::Private => "PRIVATE",
            VideoState::RemovedByUploader => "REMOVED_BY_UPLOADER",
            VideoState::AccountTerminated => "ACCOUNT_TERMINATED",
            VideoState::RemovedForTos => "REMOVED_FOR_TOS",
            VideoState::Unavailable => "UNAVAILABLE",
        }
    }

    /// Whether the video can be watched — i.e. whether it is
    /// [`VideoState::Ok`].
    pub const fn is_available(self) -> bool {
        matches!(self, VideoState::Ok)
    }

    /// The state a playability verdict means, for the availability table:
    /// available states collapse to [`VideoState::Ok`], and every other state
    /// is split by what the player's reason says.
    pub(crate) fn from_availability(availability: &PlayerAvailability) -> Self {
        if matches!(
            availability.state,
            PlayerState::Ok | PlayerState::LiveStream | PlayerState::Unknown
        ) {
            return VideoState::Ok;
        }

        let reason = availability.reason.as_deref().unwrap_or("");

        match availability.state {
            PlayerState::Unplayable => {
                if reason.contains("members-only") {
                    return VideoState::MembersOnly;
                }
                if reason == "This live stream recording is not available." {
                    return VideoState::RecordingUnavailable;
                }
                if reason.contains("confirm your age") {
                    return VideoState::AgeRestricted;
                }
                // a copyright claim is checked before a plain regional block
                if reason.contains("copyright grounds") {
                    return VideoState::CopyrightBlocked;
                }
                if reason.contains("in your country") {
                    return VideoState::RegionBlocked;
                }
                VideoState::Unavailable
            }
            PlayerState::LoginRequired => {
                if reason.contains("not a bot") {
                    return VideoState::BotDetection;
                }
                if reason.contains("confirm your age") {
                    return VideoState::AgeRestricted;
                }
                VideoState::LoginRequired
            }
            PlayerState::AgeCheckRequired => VideoState::AgeRestricted,
            PlayerState::Error => {
                if reason.contains("private") {
                    return VideoState::Private;
                }
                if reason.contains("removed by the uploader") {
                    return VideoState::RemovedByUploader;
                }
                if reason.contains("terminated") {
                    return VideoState::AccountTerminated;
                }
                if reason.contains("Community Guidelines") {
                    return VideoState::RemovedForTos;
                }
                VideoState::Unavailable
            }
            _ => VideoState::Unavailable,
        }
    }
}

impl fmt::Display for VideoState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A video.
#[derive(Debug, Clone, PartialEq)]
pub struct Video {
    pub id: VideoId,
    pub title: String,
    pub url: String,
    /// `None` for a stream with no length of its own (a live one, say).
    pub length: Option<Duration>,
    pub view_count: Option<u64>,
    pub thumbnails: Vec<Image>,
    pub description: String,
    pub channel_id: ChannelId,
    pub channel_name: String,
    pub is_live: bool,
    pub published_at: DateTime<FixedOffset>,
    pub uploaded_at: DateTime<FixedOffset>,
    pub is_family_safe: bool,
    pub category: String,
}

impl Video {
    pub(crate) fn from_page(page: PlayerPage) -> Self {
        let length = if page.length_seconds > 0 {
            Some(Duration::from_secs(page.length_seconds as u64))
        } else {
            None
        };

        Video {
            id: VideoId::new_unchecked(page.id),
            title: page.title,
            url: page.url,
            length,
            view_count: u64::try_from(page.view_count).ok(),
            thumbnails: page.thumbnails,
            description: page.description,
            channel_id: ChannelId::new_unchecked(page.channel_id),
            channel_name: page.channel_name,
            is_live: page.is_live_content,
            published_at: page.publish_date,
            uploaded_at: page.upload_date,
            is_family_safe: page.is_family_safe,
            category: page.category,
        }
    }
}
