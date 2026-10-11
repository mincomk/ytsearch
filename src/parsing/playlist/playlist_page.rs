//! Parsing rules carried over from yspy (see NOTICE).

use serde_json::Value;

use crate::error::{Error, Result};
use crate::jpath;
use crate::json::{
    get_by_path, get_by_path_or, list_at, list_at_or, str_at, string_at, to_youtube_url,
};
use crate::model::Image;
use crate::parsing::playlist::PlaylistVideoComponent;

/// `yspy.data_parsing.playlist.playlist_page.PlaylistPage`
#[derive(Debug, Clone)]
pub struct PlaylistPage {
    pub id: String,
    pub title: String,
    pub url: String,
    pub thumbnails: Vec<Image>,
    pub owner_text: String,
    pub owner_url: String,
    pub owner_id: String,
    pub view_count_text: String,
    pub videos: Vec<PlaylistVideoComponent>,
    pub continuation_token: Option<String>,
}

impl PlaylistPage {
    /// `PlaylistPage._has_error_alert(raw_data)`
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

    /// `PlaylistPage.from_json(raw_data)`.
    ///
    /// A playlist that does not exist comes back with an error alert instead of
    /// sidebar data — `Ok(None)` is that case.
    pub fn from_json(raw_data: &Value) -> Result<Option<Self>> {
        let parsed = (|| -> Result<Self> {
            let primary_info_renderer = get_by_path(
                raw_data,
                jpath!(
                    "sidebar playlistSidebarRenderer items",
                    0,
                    "playlistSidebarPrimaryInfoRenderer"
                ),
            )?;
            let secondary_info_renderer = get_by_path(
                raw_data,
                jpath!(
                    "sidebar playlistSidebarRenderer items",
                    1,
                    "playlistSidebarSecondaryInfoRenderer"
                ),
            )?;

            // video items come as playlistVideoRenderer on the first page and as
            // lockupViewModel on a continuation page
            let continuation_page_items_path = jpath!(
                "onResponseReceivedActions",
                0,
                "appendContinuationItemsAction continuationItems"
            );
            let first_page_items_path = jpath!(
                "contents twoColumnBrowseResultsRenderer tabs",
                0,
                "tabRenderer content sectionListRenderer contents",
                0,
                "itemSectionRenderer contents"
            );

            let video_items: &Vec<Value> = match list_at_or(raw_data, continuation_page_items_path)
            {
                Some(items) => items,
                None => list_at(raw_data, first_page_items_path)?,
            };

            let video_components: Vec<PlaylistVideoComponent> = video_items
                .iter()
                .enumerate()
                .filter_map(|(index, raw_video_data)| {
                    PlaylistVideoComponent::from_json(raw_video_data, index as i64).ok()
                })
                .collect();

            let continuation_page_token_path = jpath!(
                "onResponseReceivedActions",
                0,
                "appendContinuationItemsAction continuationItems",
                -1,
                "continuationItemRenderer continuationEndpoint continuationCommand token"
            );
            let first_page_token_path = jpath!(
                "contents twoColumnBrowseResultsRenderer tabs",
                0,
                "tabRenderer content sectionListRenderer contents",
                0,
                "itemSectionRenderer contents",
                -1,
                "continuationItemRenderer continuationEndpoint continuationCommand token"
            );

            let continuation_token = get_by_path_or(raw_data, continuation_page_token_path)
                .and_then(Value::as_str)
                .map(str::to_string)
                .or_else(|| {
                    get_by_path_or(raw_data, first_page_token_path)
                        .and_then(Value::as_str)
                        .map(str::to_string)
                });

            let title = str_at(primary_info_renderer, jpath!("title runs", 0, "text"))?;

            Ok(PlaylistPage {
                id: string_at(
                    primary_info_renderer,
                    jpath!(
                        "title runs",
                        0,
                        "navigationEndpoint watchEndpoint playlistId"
                    ),
                )?,
                title: title.to_string(),
                url: to_youtube_url(str_at(
                    primary_info_renderer,
                    jpath!(
                        "title runs",
                        0,
                        "navigationEndpoint commandMetadata webCommandMetadata url"
                    ),
                )?)?,
                thumbnails: Image::list_from_json(get_by_path(
                    primary_info_renderer,
                    jpath!("thumbnailRenderer playlistVideoThumbnailRenderer thumbnail thumbnails"),
                )?)?,
                owner_text: string_at(
                    secondary_info_renderer,
                    jpath!("videoOwner videoOwnerRenderer title runs", 0, "text"),
                )?,
                owner_url: format!(
                    "https://youtube.com{}",
                    string_at(
                        secondary_info_renderer,
                        jpath!(
                            "videoOwner videoOwnerRenderer title runs",
                            0,
                            "navigationEndpoint browseEndpoint canonicalBaseUrl"
                        ),
                    )?
                ),
                owner_id: string_at(
                    secondary_info_renderer,
                    jpath!(
                        "videoOwner videoOwnerRenderer title runs",
                        0,
                        "navigationEndpoint browseEndpoint browseId"
                    ),
                )?,
                view_count_text: string_at(
                    primary_info_renderer,
                    jpath!("stats", 1, "simpleText"),
                )?,
                videos: video_components,
                continuation_token,
            })
        })();

        match parsed {
            Ok(page) => Ok(Some(page)),
            Err(_) if PlaylistPage::has_error_alert(raw_data) => Ok(None),
            Err(_) => Err(Error::parse("Given data is not a playlist page data")),
        }
    }
}
