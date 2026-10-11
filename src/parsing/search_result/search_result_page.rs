//! Parsing rules carried over from yspy (see NOTICE).

use serde_json::Value;

use crate::error::Result;
use crate::jpath;
use crate::json::{get_by_path_or, string_at_or};
use crate::parsing::search_result::{
    ChannelComponent, PlaylistComponent, SearchResultComponent, VideoComponent,
};

/// `yspy.data_parsing.search_result.search_result_page.SearchResultPage`
#[derive(Debug, Clone, Default)]
pub struct SearchResultPage {
    pub components: Vec<SearchResultComponent>,
    pub continuation_token: Option<String>,
}

impl SearchResultPage {
    /// `SearchResultPage.parse_components(raw_data)`.
    ///
    /// Ad rows and half-broken renderers are not search results — they are
    /// dropped silently, exactly like yspy's `except (KeyError,
    /// DataParsingException): return []`.
    pub fn parse_components(raw_data: &Value) -> Vec<SearchResultComponent> {
        if raw_data.get("videoRenderer").is_some() {
            return VideoComponent::from_json(raw_data)
                .map(|component| vec![component.into()])
                .unwrap_or_default();
        }

        if raw_data.get("channelRenderer").is_some() {
            return ChannelComponent::from_json(raw_data)
                .map(|component| vec![component.into()])
                .unwrap_or_default();
        }

        if raw_data.get("lockupViewModel").is_some() {
            return PlaylistComponent::from_json(raw_data)
                .map(|component| vec![component.into()])
                .unwrap_or_default();
        }

        if raw_data.get("shelfRenderer").is_some() {
            let inner_data = get_by_path_or(
                raw_data,
                jpath!("shelfRenderer content verticalListRenderer items"),
            )
            .or_else(|| {
                get_by_path_or(
                    raw_data,
                    jpath!("shelfRenderer content horizontalListRenderer items"),
                )
            });

            let inner_data = match inner_data.and_then(Value::as_array) {
                Some(inner_data) => inner_data,
                None => return Vec::new(),
            };

            return inner_data
                .iter()
                .flat_map(SearchResultPage::parse_components)
                .collect();
        }

        Vec::new()
    }

    /// `SearchResultPage.from_json(raw_data)`
    pub fn from_json(raw_data: &Value) -> Result<Self> {
        let inner_data = get_by_path_or(
            raw_data,
            jpath!(
                "contents twoColumnSearchResultsRenderer primaryContents \
                 sectionListRenderer contents"
            ),
        )
        .or_else(|| {
            get_by_path_or(
                raw_data,
                jpath!(
                    "onResponseReceivedCommands",
                    0,
                    "appendContinuationItemsAction continuationItems"
                ),
            )
        });

        // not a search result page (e.g. the last continuation page) — an empty page
        let inner_data = match inner_data.and_then(Value::as_array) {
            Some(inner_data) => inner_data,
            None => return Ok(SearchResultPage::default()),
        };

        let mut continuation_token: Option<String> = None;
        let mut components: Vec<SearchResultComponent> = Vec::new();

        for element in inner_data {
            if element.get("continuationItemRenderer").is_some() {
                continuation_token = string_at_or(
                    element,
                    jpath!(
                        "continuationItemRenderer continuationEndpoint \
                         continuationCommand token"
                    ),
                );
            } else if element.get("itemSectionRenderer").is_some() {
                let raw_components =
                    match get_by_path_or(element, jpath!("itemSectionRenderer contents")) {
                        Some(raw_components) => raw_components,
                        None => continue,
                    };

                let raw_components = match raw_components.as_array() {
                    Some(raw_components) => raw_components,
                    None => continue,
                };

                for raw_component in raw_components {
                    components.extend(SearchResultPage::parse_components(raw_component));
                }
            }
        }

        Ok(SearchResultPage {
            components,
            continuation_token,
        })
    }
}
