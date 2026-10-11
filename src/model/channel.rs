//! A channel, and its "about" page.

use std::str::FromStr;

use chrono::NaiveDate;

use crate::ids::{ChannelHandle, ChannelId};
use crate::model::Image;
use crate::parsing::channel::{ChannelDetailPage, ChannelExternalLinkComponent, ChannelPage};
use crate::parsing::counts::{
    parse_joined_date, parse_subscriber_count, parse_video_count, parse_view_count,
};

/// A channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Channel {
    pub id: ChannelId,
    /// The `@handle`, once YouTube has given one.
    pub handle: Option<ChannelHandle>,
    pub title: String,
    pub url: String,
    pub description: String,
    pub avatars: Vec<Image>,
    pub banners: Vec<Image>,
    pub subscriber_count: Option<u64>,
    /// How many videos the channel has, as the about data reports it.
    pub video_count: Option<u64>,
    pub is_family_safe: bool,
    pub tags: Vec<String>,
    /// The token the about page is read with — see
    /// [`Client::channel_detail`](crate::Client::channel_detail).
    pub(crate) about_token: Option<String>,
}

impl Channel {
    pub(crate) fn from_page(page: ChannelPage, english: &ChannelPage) -> Self {
        Channel {
            id: ChannelId::new_unchecked(page.id),
            handle: ChannelHandle::from_str(&page.handle_name).ok(),
            title: page.title,
            url: page.url,
            description: page.description,
            avatars: page.avatar_thumbnails,
            banners: page.banners,
            subscriber_count: parse_subscriber_count(Some(&english.subscriber_count_text)),
            video_count: parse_video_count(Some(&english.video_count_text)),
            is_family_safe: page.is_family_safe,
            tags: page.tags,
            about_token: (!page.continuation_token.is_empty()).then_some(page.continuation_token),
        }
    }
}

/// A channel's about page: when it joined, how much it has been watched, and
/// every link it lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelDetail {
    pub id: ChannelId,
    pub url: String,
    pub description: String,
    pub country: String,
    pub subscriber_count: Option<u64>,
    pub view_count: Option<u64>,
    pub joined: Option<NaiveDate>,
    pub video_count: Option<u64>,
    pub links: Vec<ChannelLink>,
}

impl ChannelDetail {
    pub(crate) fn from_page(page: ChannelDetailPage, english: &ChannelDetailPage) -> Self {
        ChannelDetail {
            id: ChannelId::new_unchecked(page.id),
            url: page.url,
            description: page.description,
            country: page.country,
            subscriber_count: parse_subscriber_count(Some(&english.subscriber_count_text)),
            view_count: parse_view_count(Some(&english.view_count_text)),
            joined: parse_joined_date(Some(&english.joined_date_text)),
            video_count: parse_video_count(Some(&english.video_count_text)),
            links: page.links.into_iter().map(ChannelLink::from).collect(),
        }
    }
}

/// One link on a channel's about page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelLink {
    /// The link's label, as the page shows it.
    pub title: String,
    /// The link's text, which is usually the URL as shown.
    pub text: String,
    /// The URL the link opens.
    pub url: String,
    pub favicons: Vec<Image>,
}

impl From<ChannelExternalLinkComponent> for ChannelLink {
    fn from(link: ChannelExternalLinkComponent) -> Self {
        ChannelLink {
            title: link.title,
            text: link.content,
            url: link.endpoint_url,
            favicons: link.favicons,
        }
    }
}
