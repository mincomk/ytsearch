//! Parsing rules carried over from yspy (see NOTICE).

use serde_json::Value;

use crate::error::{Error, Result};
use crate::jpath;
use crate::json::{get_by_path, get_by_path_or, str_at_or, string_at, string_at_or};
use crate::model::Image;

/// one
/// `lockupViewModel` row of playlist type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistComponent {
    pub id: String,
    pub title: String,
    pub url: String,
    pub thumbnails: Vec<Image>,
    pub channel_id: Option<String>,
    pub channel_name: Option<String>,
}

impl PlaylistComponent {
    /// `PlaylistComponent.from_json(raw_data)`
    pub fn from_json(raw_data: &Value) -> Result<Self> {
        let lockup_data = raw_data
            .get("lockupViewModel")
            .ok_or_else(|| Error::parse("Given json data is not a playlist lockup data"))?;

        if lockup_data.get("contentType").and_then(Value::as_str)
            != Some("LOCKUP_CONTENT_TYPE_PLAYLIST")
        {
            return Err(Error::parse(
                "Given lockup data is not a playlist type data",
            ));
        }

        let parsed = (|| -> Result<PlaylistComponent> {
            let metadata_rows = get_by_path(
                lockup_data,
                jpath!(
                    "metadata lockupMetadataViewModel metadata \
                     contentMetadataViewModel metadataRows"
                ),
            )?;

            let mut channel_id: Option<String> = None;
            let mut channel_name: Option<String> = None;

            'rows: for row in metadata_rows
                .as_array()
                .ok_or(crate::json::PathError::NotAList)?
            {
                let parts = match row.get("metadataParts").and_then(Value::as_array) {
                    Some(parts) => parts,
                    None => continue,
                };

                for part in parts {
                    let command_runs = match get_by_path_or(part, jpath!("text commandRuns")) {
                        Some(runs) => match runs.as_array() {
                            Some(runs) => runs,
                            None => continue,
                        },
                        None => continue,
                    };

                    for command_run in command_runs {
                        let browse_id = str_at_or(
                            command_run,
                            jpath!("onTap innertubeCommand browseEndpoint browseId"),
                        );
                        if let Some(browse_id) = browse_id {
                            if browse_id.starts_with("UC") {
                                channel_id = Some(browse_id.to_string());
                                channel_name = string_at_or(part, jpath!("text content"));
                                break 'rows;
                            }
                        }
                    }
                }
            }

            Ok(PlaylistComponent {
                id: string_at(lockup_data, jpath!("contentId"))?,
                title: string_at(
                    lockup_data,
                    jpath!("metadata lockupMetadataViewModel title content"),
                )?,
                url: format!(
                    "https://youtube.com/playlist?list={}",
                    string_at(lockup_data, jpath!("contentId"))?
                ),
                thumbnails: Image::list_from_json(get_by_path(
                    lockup_data,
                    jpath!(
                        "contentImage collectionThumbnailViewModel primaryThumbnail \
                         thumbnailViewModel image sources"
                    ),
                )?)?,
                channel_id,
                channel_name,
            })
        })();

        parsed.map_err(|_| Error::parse("Given json data is not a playlist lockup data"))
    }
}
