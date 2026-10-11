//! Parsing rules carried over from yspy (see NOTICE).

use serde_json::Value;

use crate::error::{Error, Result};
use crate::jpath;
use crate::json::{get_by_path, string_at, string_at_or};
use crate::model::Image;

/// one
/// `videoRenderer` row of a search result page.
///
/// `length_text` / `view_count_text` are annotated `str` in yspy but are read
/// with `get_by_path_or`, so they are `Option<String>` here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoComponent {
    pub id: String,
    pub title: String,
    pub url: String,
    pub thumbnails: Vec<Image>,
    pub is_shorts: bool,
    pub published_time_text: Option<String>,
    pub length_text: Option<String>,
    pub view_count_text: Option<String>,
    pub channel_url: String,
    pub channel_id: String,
}

impl VideoComponent {
    /// `VideoComponent.from_json(raw_data)`
    pub fn from_json(raw_data: &Value) -> Result<Self> {
        let inner_data = raw_data
            .get("videoRenderer")
            .ok_or_else(|| Error::parse("Given json data is not a video renderer data"))?;

        let web_url = string_at(
            inner_data,
            jpath!("navigationEndpoint commandMetadata webCommandMetadata url"),
        )?;

        // 'reelWatchEndpoint' in inner_data['navigationEndpoint']
        let is_shorts = inner_data
            .get("navigationEndpoint")
            .map(|endpoint| endpoint.get("reelWatchEndpoint").is_some())
            .unwrap_or(false);

        let channel_web_url = string_at(
            inner_data,
            jpath!(
                "ownerText runs",
                0,
                "navigationEndpoint commandMetadata webCommandMetadata url"
            ),
        )?;

        Ok(VideoComponent {
            id: string_at(inner_data, jpath!("videoId"))?,
            title: string_at(inner_data, jpath!("title runs", 0, "text"))?,
            url: format!("https://youtube.com{web_url}"),
            thumbnails: Image::list_from_json(get_by_path(
                inner_data,
                jpath!("thumbnail thumbnails"),
            )?)?,
            is_shorts,
            published_time_text: string_at_or(inner_data, jpath!("publishedTimeText simpleText")),
            length_text: string_at_or(inner_data, jpath!("lengthText simpleText")),
            view_count_text: string_at_or(inner_data, jpath!("viewCountText simpleText")),
            channel_url: format!("https://youtube.com{channel_web_url}"),
            channel_id: string_at(
                inner_data,
                jpath!(
                    "ownerText runs",
                    0,
                    "navigationEndpoint browseEndpoint browseId"
                ),
            )?,
        })
    }
}
