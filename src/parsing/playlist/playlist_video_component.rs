//! Parsing rules carried over from yspy (see NOTICE).

use serde_json::Value;

use crate::error::{Error, Result};
use crate::jpath;
use crate::json::{get_by_path, string_at};
use crate::model::Image;

/// `yspy.data_parsing.playlist.playlist_video_component.PlaylistVideoComponent`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistVideoComponent {
    pub id: String,
    pub title: String,
    pub url: String,
    pub thumbnails: Vec<Image>,
    pub index: i64,
    pub owner_text: String,
    pub length_text: String,
}

impl PlaylistVideoComponent {
    /// `PlaylistVideoComponent.from_json(raw_data, index=index)`
    pub fn from_json(raw_data: &Value, index: i64) -> Result<Self> {
        let inner_data = raw_data
            .get("lockupViewModel")
            .ok_or_else(|| Error::parse("Given json data is not a playlist video lockup data"))?;

        let metadata = get_by_path(inner_data, jpath!("metadata lockupMetadataViewModel"))
            .map_err(|_| Error::parse("Given json data is not a playlist video lockup data"))?;

        let length_text = string_at(
            inner_data,
            jpath!(
                "contentImage thumbnailViewModel overlays",
                0,
                "thumbnailBottomOverlayViewModel badges",
                0,
                "thumbnailBadgeViewModel text"
            ),
        )
        .map_err(|_| Error::parse("Given json data is not a playlist video lockup data"))?;

        if inner_data.get("contentType").and_then(Value::as_str)
            != Some("LOCKUP_CONTENT_TYPE_VIDEO")
        {
            return Err(Error::parse("Given lockup data is not a video type data"));
        }

        Ok(PlaylistVideoComponent {
            id: string_at(inner_data, jpath!("contentId"))?,
            title: string_at(metadata, jpath!("title content"))?,
            url: format!(
                "https://youtube.com/watch?v={}",
                string_at(inner_data, jpath!("contentId"))?
            ),
            thumbnails: Image::list_from_json(get_by_path(
                inner_data,
                jpath!("contentImage thumbnailViewModel image sources"),
            )?)?,
            index,
            owner_text: string_at(
                metadata,
                jpath!(
                    "metadata contentMetadataViewModel metadataRows",
                    0,
                    "metadataParts",
                    0,
                    "text content"
                ),
            )?,
            length_text,
        })
    }
}
