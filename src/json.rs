//! Reading json by path — the small, I/O-free vocabulary every parser uses.
//!
//! The pages this crate reads are deeply nested and only loosely typed, so the
//! parsers address them by path rather than by deserializing into a schema:
//! `get_by_path(&value, jpath!("metadata channelMetadataRenderer", "title"))`.
//! A path chunk is split on whitespace (one chunk, several keys) and an integer
//! is a list index, counted from the end when negative.

use serde_json::{Map, Value};
use thiserror::Error;
use url::Url;

/// A json object.
pub(crate) type JsonMap = Map<String, Value>;

/// One step of a [`get_by_path`] path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Path {
    /// One or more object keys — the chunk is split on whitespace, so
    /// `Path::Key("title runs")` means `title`, then `runs`.
    Key(&'static str),
    /// A list index; a negative index counts from the end.
    Index(i64),
}

impl From<&'static str> for Path {
    fn from(key: &'static str) -> Self {
        Path::Key(key)
    }
}

impl From<i64> for Path {
    fn from(index: i64) -> Self {
        Path::Index(index)
    }
}

/// Build a `&[Path]` out of string chunks and integer indices:
/// `jpath!("title runs", 0, "text")`.
#[macro_export]
macro_rules! jpath {
    () => {
        (&[] as &[$crate::json::Path])
    };
    ($($element:expr),+ $(,)?) => {
        (&[$($crate::json::Path::from($element)),+] as &[$crate::json::Path])
    };
}

/// Build a json object out of pairs: `json_map! { "query" => "minecraft" }`.
#[macro_export]
macro_rules! json_map {
    () => {
        $crate::json::JsonMap::new()
    };
    ($($key:expr => $value:expr),+ $(,)?) => {{
        let mut map = $crate::json::JsonMap::new();
        $(map.insert($key.to_string(), ::serde_json::json!($value));)+
        map
    }};
}

/// Why a json path did not resolve.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PathError {
    /// The object has no such key.
    #[error("no such key: {0}")]
    MissingKey(String),
    /// The list has no such index.
    #[error("index out of range: {0}")]
    IndexOutOfRange(i64),
    /// A key was read off something that is not an object.
    #[error("expected an object to read key {0}")]
    NotAnObject(String),
    /// An index was read off something that is not a list.
    #[error("expected a list")]
    NotAList,
    /// The value exists but is not a string.
    #[error("expected a string")]
    NotAString,
    /// The value exists but is not a number.
    #[error("expected a number")]
    NotANumber,
    /// The value exists but is not a boolean.
    #[error("expected a boolean")]
    NotABoolean,
}

/// Walk `path` into `data`.
///
/// An empty path is `data` itself.
pub fn get_by_path<'a>(data: &'a Value, path: &[Path]) -> Result<&'a Value, PathError> {
    let mut current = data;

    for element in path {
        match element {
            Path::Key(chunk) => {
                for key in chunk.split_whitespace() {
                    let object = current
                        .as_object()
                        .ok_or_else(|| PathError::NotAnObject(key.to_string()))?;
                    current = object
                        .get(key)
                        .ok_or_else(|| PathError::MissingKey(key.to_string()))?;
                }
            }
            Path::Index(index) => {
                let list = current.as_array().ok_or(PathError::NotAList)?;
                let length = list.len() as i64;
                let resolved = if *index < 0 { length + *index } else { *index };
                if resolved < 0 || resolved >= length {
                    return Err(PathError::IndexOutOfRange(*index));
                }
                current = &list[resolved as usize];
            }
        }
    }

    Ok(current)
}

/// The same, as an `Option`.
pub fn get_by_path_or<'a>(data: &'a Value, path: &[Path]) -> Option<&'a Value> {
    get_by_path(data, path).ok()
}

/// The same, for a value that may itself be absent.
pub fn get_by_path_or_else<'a>(data: Option<&'a Value>, path: &[Path]) -> Option<&'a Value> {
    data.and_then(|data| get_by_path_or(data, path))
}

/// The value as a string, if it is one.
pub fn value_as_str(value: &Value) -> Option<&str> {
    value.as_str()
}

/// A json number, or a string holding one.
pub fn value_as_i64(value: &Value) -> Option<i64> {
    match value {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text.trim().parse::<i64>().ok(),
        _ => None,
    }
}

/// `get_by_path`, where the value must be a string.
pub fn str_at<'a>(data: &'a Value, path: &[Path]) -> Result<&'a str, PathError> {
    get_by_path(data, path)?
        .as_str()
        .ok_or(PathError::NotAString)
}

/// `str_at`, owned.
pub fn string_at(data: &Value, path: &[Path]) -> Result<String, PathError> {
    Ok(str_at(data, path)?.to_string())
}

/// `str_at`, but `None` when the path does not resolve to a string.
pub fn str_at_or<'a>(data: &'a Value, path: &[Path]) -> Option<&'a str> {
    get_by_path_or(data, path).and_then(value_as_str)
}

/// `string_at`, but `None` when the path does not resolve to a string.
pub fn string_at_or(data: &Value, path: &[Path]) -> Option<String> {
    str_at_or(data, path).map(str::to_string)
}

/// `get_by_path`, where the value must be an integer-ish scalar.
pub fn i64_at(data: &Value, path: &[Path]) -> Result<i64, PathError> {
    value_as_i64(get_by_path(data, path)?).ok_or(PathError::NotANumber)
}

/// `get_by_path`, where the value must be a boolean.
pub fn bool_at(data: &Value, path: &[Path]) -> Result<bool, PathError> {
    get_by_path(data, path)?
        .as_bool()
        .ok_or(PathError::NotABoolean)
}

/// `get_by_path`, where the value must be a list.
pub fn list_at<'a>(data: &'a Value, path: &[Path]) -> Result<&'a Vec<Value>, PathError> {
    get_by_path(data, path)?
        .as_array()
        .ok_or(PathError::NotAList)
}

/// `list_at`, but `None` when the path does not resolve to a list.
pub fn list_at_or<'a>(data: &'a Value, path: &[Path]) -> Option<&'a Vec<Value>> {
    get_by_path_or(data, path).and_then(Value::as_array)
}

/// An absolute url kept as it is, a relative one joined onto `https://youtube.com`.
pub fn to_youtube_url(url: &str) -> Result<String, url::ParseError> {
    match Url::parse(url) {
        Ok(parsed) => Ok(parsed.to_string()),
        Err(url::ParseError::RelativeUrlWithoutBase) => {
            let base = Url::parse("https://youtube.com")?;
            Ok(base.join(url)?.to_string())
        }
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_path_reads_keys_lists_and_negative_indices() {
        let data = json!({"a": {"b": [{"text": "hi"}]}});

        assert_eq!(
            get_by_path(&data, jpath!("a b", 0, "text")).unwrap(),
            &json!("hi")
        );
        assert_eq!(
            get_by_path(&data, jpath!("a b", -1, "text")).unwrap(),
            &json!("hi")
        );
        assert_eq!(
            get_by_path(&data, jpath!("a b", 1)).unwrap_err(),
            PathError::IndexOutOfRange(1)
        );
        assert_eq!(
            get_by_path(&data, jpath!("nope")).unwrap_err(),
            PathError::MissingKey("nope".to_string())
        );
        assert_eq!(
            get_by_path(&json!({"a": 1}), jpath!("a b")).unwrap_err(),
            PathError::NotAnObject("b".to_string())
        );
        assert_eq!(get_by_path(&data, jpath!()).unwrap(), &data);
    }

    #[test]
    fn a_relative_youtube_url_is_joined() {
        assert_eq!(
            to_youtube_url("/playlist?list=PL1").unwrap(),
            "https://youtube.com/playlist?list=PL1"
        );
        assert_eq!(
            to_youtube_url("https://www.youtube.com/@sleepground").unwrap(),
            "https://www.youtube.com/@sleepground"
        );
    }
}
