//! Parsing rules carried over from yspy (see NOTICE).

use serde_json::{Map, Value};

use crate::error::{Error, Result};

/// `yspy.data_parsing.suggestion_data.find_by_type` for the three shapes the
/// suggestion payload mixes on one line.
fn find_str(elements: &[Value]) -> Option<&str> {
    elements.iter().find_map(Value::as_str)
}

fn find_object(elements: &[Value]) -> Option<&Map<String, Value>> {
    elements.iter().find_map(Value::as_object)
}

fn find_list(elements: &[Value]) -> Option<&Vec<Value>> {
    elements.iter().find_map(Value::as_array)
}

fn metadata_str(metadata: Option<&Map<String, Value>>, key: &str) -> Option<String> {
    metadata
        .and_then(|metadata| metadata.get(key))
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// `yspy.data_parsing.suggestion_data.SuggestionElement`
///
/// yspy annotates `preview_image_url` as an `Image` but assigns the
/// raw `zai` value, which is a url string — so it is an `Option<String>` here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestionElement {
    pub text: String,
    pub preview_image_url: Option<String>,
    pub footer: Option<String>,
    pub external_url: Option<String>,
}

impl SuggestionElement {
    /// `SuggestionElement.from_json(raw_data)`
    pub fn from_json(raw_data: &[Value]) -> Result<Self> {
        let text = find_str(raw_data)
            .ok_or_else(|| Error::parse("The given data is not a suggestion element data"))?;

        let metadata = find_object(raw_data);

        Ok(SuggestionElement {
            text: text.to_string(),
            preview_image_url: metadata_str(metadata, "zai"),
            footer: metadata_str(metadata, "zaf"),
            external_url: metadata_str(metadata, "zaq"),
        })
    }
}

/// `yspy.data_parsing.suggestion_data.SuggestionData`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestionData {
    pub query: String,
    pub suggestions: Vec<SuggestionElement>,
}

impl SuggestionData {
    /// `SuggestionData.unwrap_parentheses(raw_text)` — the completion endpoint
    /// answers a jsonp body, so the payload is cut out of its parentheses.
    pub fn unwrap_parentheses(raw_text: &str) -> Result<Vec<Value>> {
        let start = raw_text
            .find('(')
            .ok_or_else(|| Error::parse("The given data is not a suggestions data"))?;

        // raw_text[raw_text.index('(') + 1 : -1] — the last *character* is dropped
        let end = raw_text
            .char_indices()
            .last()
            .map(|(index, _)| index)
            .unwrap_or(0);

        if end <= start + 1 {
            return Err(Error::parse("The given data is not a suggestions data"));
        }

        Ok(serde_json::from_str(&raw_text[start + 1..end])?)
    }

    /// `SuggestionData.from_json(raw_data)`
    pub fn from_json(raw_data: &[Value]) -> Result<Self> {
        let query = find_str(raw_data)
            .ok_or_else(|| Error::parse("The given data is not a suggestions data"))?;

        let elements = find_list(raw_data)
            .ok_or_else(|| Error::parse("The given data is not a suggestions data"))?;

        let mut suggestions = Vec::with_capacity(elements.len());
        for element in elements {
            let element = element
                .as_array()
                .ok_or_else(|| Error::parse("The given data is not a suggestions data"))?;
            suggestions.push(SuggestionElement::from_json(element)?);
        }

        Ok(SuggestionData {
            query: query.to_string(),
            suggestions,
        })
    }
}
