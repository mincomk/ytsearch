//! Parsing rules carried over from yspy (see NOTICE).

use serde_json::Value;

use crate::error::{Error, Result};
use crate::jpath;
use crate::json::{bool_at, get_by_path, get_by_path_or, get_by_path_or_else, str_at, string_at};
use crate::model::Image;

/// `yspy.data_parsing.channel.channel_page.ChannelPage`
#[derive(Debug, Clone)]
pub struct ChannelPage {
    pub id: String,
    pub title: String,
    pub handle_name: String,
    pub url: String,
    pub description: String,
    pub avatar_thumbnails: Vec<Image>,
    pub banners: Vec<Image>,
    pub subscriber_count_text: String,
    pub video_count_text: String,
    pub is_family_safe: bool,
    pub tags: Vec<String>,
    pub continuation_token: String,
}

impl ChannelPage {
    /// `ChannelPage._has_error_alert(raw_data)`
    pub fn has_error_alert(raw_data: &Value) -> bool {
        let alerts = match get_by_path_or(raw_data, jpath!("alerts")).and_then(Value::as_array) {
            Some(alerts) => alerts,
            None => return false,
        };

        alerts.iter().any(|alert| {
            get_by_path_or(alert, jpath!("alertRenderer type")).and_then(Value::as_str)
                == Some("ERROR")
        })
    }

    /// `ChannelPage.from_json(raw_data)`.
    ///
    /// A channel that is unavailable (e.g. does not exist) comes back with an
    /// error alert instead of channel metadata — `Ok(None)` is that case.
    pub fn from_json(raw_data: &Value) -> Result<Option<Self>> {
        let parsed = (|| -> Result<Self> {
            let header_data = get_by_path(raw_data, jpath!("header"))?;
            let channel_metadata =
                get_by_path(raw_data, jpath!("metadata channelMetadataRenderer"))?;
            let microformat_data =
                get_by_path(raw_data, jpath!("microformat microformatDataRenderer"))?;

            let tags = get_by_path(microformat_data, jpath!("tags"))?
                .as_array()
                .ok_or(crate::json::PathError::NotAList)?
                .iter()
                .map(|tag| {
                    tag.as_str()
                        .map(str::to_string)
                        .ok_or(crate::json::PathError::NotAString)
                })
                .collect::<std::result::Result<Vec<String>, _>>()?;

            Ok(ChannelPage {
                id: string_at(channel_metadata, jpath!("externalId"))?,
                title: string_at(channel_metadata, jpath!("title"))?,
                handle_name: str_at(
                    header_data,
                    jpath!(
                        "pageHeaderRenderer content pageHeaderViewModel metadata \
                         contentMetadataViewModel metadataRows",
                        0,
                        "metadataParts",
                        0,
                        "text content"
                    ),
                )?
                .to_string(),
                url: string_at(microformat_data, jpath!("urlCanonical"))?,
                description: string_at(channel_metadata, jpath!("description"))?,
                avatar_thumbnails: Image::list_from_json(get_by_path(
                    channel_metadata,
                    jpath!("avatar thumbnails"),
                )?)?,
                banners: Image::list_from_json_or(get_by_path_or_else(
                    Some(header_data),
                    jpath!(
                        "pageHeaderRenderer content pageHeaderViewModel banner \
                         imageBannerViewModel image sources"
                    ),
                ))?,
                subscriber_count_text: str_at(
                    header_data,
                    jpath!(
                        "pageHeaderRenderer content pageHeaderViewModel metadata \
                         contentMetadataViewModel metadataRows",
                        1,
                        "metadataParts",
                        0,
                        "text content"
                    ),
                )?
                .to_string(),
                video_count_text: str_at(
                    header_data,
                    jpath!(
                        "pageHeaderRenderer content pageHeaderViewModel metadata \
                         contentMetadataViewModel metadataRows",
                        1,
                        "metadataParts",
                        1,
                        "text content"
                    ),
                )?
                .to_string(),
                is_family_safe: bool_at(channel_metadata, jpath!("isFamilySafe"))?,
                tags,
                continuation_token: str_at(
                    header_data,
                    jpath!(
                        "pageHeaderRenderer content pageHeaderViewModel description \
                         descriptionPreviewViewModel rendererContext commandContext onTap \
                         innertubeCommand showEngagementPanelEndpoint engagementPanel \
                         engagementPanelSectionListRenderer content sectionListRenderer contents",
                        0,
                        "itemSectionRenderer contents",
                        0,
                        "continuationItemRenderer continuationEndpoint continuationCommand token"
                    ),
                )?
                .to_string(),
            })
        })();

        match parsed {
            Ok(page) => Ok(Some(page)),
            Err(_) if ChannelPage::has_error_alert(raw_data) => Ok(None),
            Err(_) => Err(Error::parse("Given data is not a channel page data")),
        }
    }
}
