//! Parsing rules carried over from yspy (see NOTICE).

use serde_json::Value;

use crate::error::{Error, Result};
use crate::jpath;
use crate::json::{get_by_path, string_at};
use crate::parsing::channel::ChannelExternalLinkComponent;

/// `yspy.data_parsing.channel.channel_detail_page.ChannelDetailPage`
#[derive(Debug, Clone)]
pub struct ChannelDetailPage {
    pub id: String,
    pub url: String,
    pub description: String,
    pub country: String,
    pub subscriber_count_text: String,
    pub view_count_text: String,
    pub joined_date_text: String,
    pub video_count_text: String,
    pub links: Vec<ChannelExternalLinkComponent>,
}

impl ChannelDetailPage {
    /// `ChannelDetailPage.from_json(raw_data)`
    pub fn from_json(raw_data: &Value) -> Result<Self> {
        let inner_data = get_by_path(
            raw_data,
            jpath!(
                "onResponseReceivedEndpoints",
                0,
                "appendContinuationItemsAction continuationItems",
                0,
                "aboutChannelRenderer"
            ),
        )
        .map_err(|_| Error::parse("Given data is not a channel detail page (popup ui) data"))?;

        let about_channel_view_data =
            get_by_path(inner_data, jpath!("metadata aboutChannelViewModel"))?;

        let links = get_by_path(about_channel_view_data, jpath!("links"))?
            .as_array()
            .ok_or(crate::json::PathError::NotAList)?
            .iter()
            .map(ChannelExternalLinkComponent::from_json)
            .collect::<Result<Vec<_>>>()?;

        Ok(ChannelDetailPage {
            id: string_at(about_channel_view_data, jpath!("channelId"))?,
            url: string_at(about_channel_view_data, jpath!("canonicalChannelUrl"))?,
            description: string_at(about_channel_view_data, jpath!("description"))?,
            country: string_at(about_channel_view_data, jpath!("country"))?,
            subscriber_count_text: string_at(
                about_channel_view_data,
                jpath!("subscriberCountText"),
            )?,
            view_count_text: string_at(about_channel_view_data, jpath!("viewCountText"))?,
            joined_date_text: string_at(about_channel_view_data, jpath!("joinedDateText content"))?,
            video_count_text: string_at(about_channel_view_data, jpath!("videoCountText"))?,
            links,
        })
    }
}
