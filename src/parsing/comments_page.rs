//! Parsing rules carried over from yspy (see NOTICE).

use serde_json::Value;

use crate::error::{Error, Result};
use crate::jpath;
use crate::json::{get_by_path, get_by_path_or, string_at};

/// `yspy.data_parsing.comments_page.CommentComponent`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentComponent {
    pub comment: String,
    pub author_url: String,
    pub author_display_name: String,
}

impl CommentComponent {
    /// `CommentComponent.from_json(raw_data)`
    pub fn from_json(raw_data: &Value) -> Result<Self> {
        let comment = string_at(
            raw_data,
            jpath!("payload commentEntityPayload properties content content"),
        )
        .map_err(|_| Error::parse("The given data is not a comment component data"))?;

        let author_url = string_at(
            raw_data,
            jpath!(
                "payload commentEntityPayload author channelCommand innertubeCommand \
                 browseEndpoint canonicalBaseUrl"
            ),
        )
        .map_err(|_| Error::parse("The given data is not a comment component data"))?;

        let author_display_name = string_at(
            raw_data,
            jpath!("payload commentEntityPayload author displayName"),
        )
        .map_err(|_| Error::parse("The given data is not a comment component data"))?;

        Ok(CommentComponent {
            comment,
            author_url: format!("https://youtube.com{author_url}"),
            author_display_name,
        })
    }
}

/// `yspy.data_parsing.comments_page.CommentsPage`
#[derive(Debug, Clone, Default)]
pub struct CommentsPage {
    pub comments: Vec<CommentComponent>,
    pub continuation_token: Option<String>,
}

impl CommentsPage {
    /// `CommentsPage.from_json(raw_data)`
    pub fn from_json(raw_data: &Value) -> Result<Self> {
        let mut comment_components: Vec<CommentComponent> = Vec::new();

        if let Some(raw_components) = get_by_path_or(
            raw_data,
            jpath!("frameworkUpdates entityBatchUpdate mutations"),
        )
        .and_then(Value::as_array)
        {
            for raw_component in raw_components {
                if let Ok(component) = CommentComponent::from_json(raw_component) {
                    comment_components.push(component);
                }
            }
        }

        let reload_token_path = jpath!(
            "onResponseReceivedEndpoints",
            -1,
            "reloadContinuationItemsCommand",
            "continuationItems",
            -1,
            "continuationItemRenderer",
            "continuationEndpoint continuationCommand token"
        );
        let append_token_path = jpath!(
            "onResponseReceivedEndpoints",
            -1,
            "appendContinuationItemAction",
            "continuationItems",
            -1,
            "continuationItemRenderer",
            "continuationEndpoint continuationCommand token"
        );

        let continuation_token = get_by_path(raw_data, reload_token_path)
            .ok()
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                get_by_path(raw_data, append_token_path)
                    .ok()
                    .and_then(Value::as_str)
                    .map(str::to_string)
            });

        Ok(CommentsPage {
            comments: comment_components,
            continuation_token,
        })
    }
}
