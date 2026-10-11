//! Parsing rules carried over from yspy (see NOTICE).

use serde_json::Value;

use crate::error::{Error, Result};
use crate::jpath;
use crate::json::string_at;

/// `yspy.data_parsing.player.player_next_page.PlayerNextPage`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerNextPage {
    pub comment_continuation_token: String,
}

impl PlayerNextPage {
    /// `PlayerNextPage.from_json(raw_data)`
    pub fn from_json(raw_data: &Value) -> Result<Self> {
        let comment_continuation_token = string_at(
            raw_data,
            jpath!(
                "engagementPanels",
                0,
                "engagementPanelSectionListRenderer content sectionListRenderer",
                "contents",
                0,
                "itemSectionRenderer",
                "contents",
                0,
                "continuationItemRenderer continuationEndpoint continuationCommand token"
            ),
        )
        .map_err(|_| Error::parse("The given data is not a video continuation page"))?;

        Ok(PlayerNextPage {
            comment_continuation_token,
        })
    }
}
