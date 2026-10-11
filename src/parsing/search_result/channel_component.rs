//! Parsing rules carried over from yspy (see NOTICE).

use serde_json::Value;

use crate::error::{Error, Result};
use crate::jpath;
use crate::json::{get_by_path, string_at, string_at_or};
use crate::model::Image;

/// one
/// `channelRenderer` row of a search result page.
///
/// YouTube puts the `@handle` in `subscriberCountText` and the real subscriber
/// count text in `videoCountText` — the field names no longer match their
/// content, and this reads `videoCountText` like yspy does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelComponent {
    pub id: String,
    pub title: String,
    pub url: String,
    pub thumbnails: Vec<Image>,
    pub description_snippet: Option<String>,
    pub subscribers_count_text: String,
}

impl ChannelComponent {
    /// `ChannelComponent.from_json(raw_data)`
    pub fn from_json(raw_data: &Value) -> Result<Self> {
        let inner_data = raw_data
            .get("channelRenderer")
            .ok_or_else(|| Error::parse("Given json data is not a channel renderer data"))?;

        let web_url = string_at(
            inner_data,
            jpath!("navigationEndpoint commandMetadata webCommandMetadata url"),
        )?;

        Ok(ChannelComponent {
            id: string_at(inner_data, jpath!("channelId"))?,
            title: string_at(inner_data, jpath!("title simpleText"))?,
            url: format!("https://youtube.com{web_url}"),
            thumbnails: Image::list_from_json(get_by_path(
                inner_data,
                jpath!("thumbnail thumbnails"),
            )?)?,
            description_snippet: string_at_or(
                inner_data,
                jpath!("descriptionSnippet runs", 0, "text"),
            ),
            subscribers_count_text: string_at(inner_data, jpath!("videoCountText simpleText"))?,
        })
    }
}
