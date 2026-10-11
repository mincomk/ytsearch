//! Parsing rules carried over from yspy (see NOTICE).

use serde_json::Value;

use crate::error::{Error, Result};
use crate::jpath;
use crate::json::{get_by_path, string_at};
use crate::model::Image;

/// `yspy.data_parsing.channel.channel_external_link_component.ChannelExternalLinkComponent`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelExternalLinkComponent {
    pub title: String,
    pub content: String,
    pub endpoint_url: String,
    pub favicons: Vec<Image>,
}

impl ChannelExternalLinkComponent {
    /// `ChannelExternalLinkComponent.from_json(raw_data)`
    pub fn from_json(raw_data: &Value) -> Result<Self> {
        let inner_data = raw_data
            .get("channelExternalLinkViewModel")
            .ok_or_else(|| Error::parse("Given data is not a channel external link api model"))?;

        Ok(ChannelExternalLinkComponent {
            title: string_at(inner_data, jpath!("title content"))?,
            content: string_at(inner_data, jpath!("link content"))?,
            endpoint_url: string_at(
                inner_data,
                jpath!(
                    "link commandRuns",
                    0,
                    "onTap innertubeCommand urlEndpoint url"
                ),
            )?,
            favicons: Image::list_from_json(get_by_path(inner_data, jpath!("favicon sources"))?)?,
        })
    }
}
