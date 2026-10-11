//! Search completions.

use crate::parsing::suggestion_data::{SuggestionData, SuggestionElement};

/// What YouTube suggests for a query — the completion endpoint's answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    /// The query the suggestions were asked for.
    pub query: String,
    pub items: Vec<SuggestionItem>,
}

impl From<SuggestionData> for Suggestion {
    fn from(data: SuggestionData) -> Self {
        Suggestion {
            query: data.query,
            items: data
                .suggestions
                .into_iter()
                .map(SuggestionItem::from)
                .collect(),
        }
    }
}

/// One completion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestionItem {
    /// The completion itself.
    pub text: String,
    /// A preview image, when YouTube offers one.
    pub image_url: Option<String>,
    /// A trailing note, when YouTube offers one.
    pub footer: Option<String>,
    /// The external URL the completion points at, when it points outside.
    pub url: Option<String>,
}

impl From<SuggestionElement> for SuggestionItem {
    fn from(element: SuggestionElement) -> Self {
        SuggestionItem {
            text: element.text,
            image_url: element.preview_image_url,
            footer: element.footer,
            url: element.external_url,
        }
    }
}
