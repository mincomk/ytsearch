//! A playlist, one page at a time.

use std::time::Duration;

use crate::ids::{ChannelId, PlaylistId, VideoId};
use crate::model::Image;
use crate::parsing::counts::{parse_duration, parse_view_count};
use crate::parsing::playlist::{PlaylistPage as ParsedPage, PlaylistVideoComponent};

/// One page of a playlist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistPage {
    pub id: PlaylistId,
    pub title: String,
    pub url: String,
    pub thumbnails: Vec<Image>,
    pub owner_id: ChannelId,
    pub owner_name: String,
    pub owner_url: String,
    pub view_count: Option<u64>,
    pub items: Vec<PlaylistItem>,
    /// Which page of the playlist this is, counting from zero.
    pub page_index: u32,
    /// How many items the earlier pages held.
    pub index_offset: u32,
}

impl PlaylistPage {
    pub(crate) fn from_parsed(
        page: ParsedPage,
        english: &ParsedPage,
        page_index: u32,
        index_offset: u32,
    ) -> Self {
        PlaylistPage {
            id: PlaylistId::new_unchecked(page.id),
            title: page.title,
            url: page.url,
            thumbnails: page.thumbnails,
            owner_id: ChannelId::new_unchecked(page.owner_id),
            owner_name: page.owner_text,
            owner_url: page.owner_url,
            view_count: parse_view_count(Some(&english.view_count_text)),
            items: page
                .videos
                .iter()
                .map(|video| PlaylistItem::from_component(video, index_offset))
                .collect(),
            page_index,
            index_offset,
        }
    }
}

/// One video of a playlist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistItem {
    pub id: VideoId,
    pub title: String,
    pub url: String,
    pub thumbnails: Vec<Image>,
    /// The item's position in the whole playlist, not in this page.
    pub index: u32,
    pub owner_name: String,
    pub length: Option<Duration>,
}

impl PlaylistItem {
    pub(crate) fn from_component(component: &PlaylistVideoComponent, index_offset: u32) -> Self {
        let index = (i64::from(index_offset) + component.index).max(0) as u32;

        PlaylistItem {
            id: VideoId::new_unchecked(component.id.clone()),
            title: component.title.clone(),
            url: component.url.clone(),
            thumbnails: component.thumbnails.clone(),
            index,
            owner_name: component.owner_text.clone(),
            length: parse_duration(Some(&component.length_text)),
        }
    }
}
