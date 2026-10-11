//! Thumbnails.

use std::fmt;

use serde_json::Value;

use crate::error::Result;
use crate::jpath;
use crate::json::{i64_at, string_at};

/// One image of a thumbnail set.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Image {
    /// The image URL, as the page gives it (YouTube thumbnails are often
    /// protocol-relative, so this is a string and not a `Url`).
    pub url: String,
    pub width: i64,
    pub height: i64,
}

impl Image {
    /// `width * height`.
    pub fn area(&self) -> i64 {
        self.width * self.height
    }

    pub(crate) fn from_json(raw: &Value) -> Result<Self> {
        Ok(Image {
            url: string_at(raw, jpath!("url"))?,
            width: i64_at(raw, jpath!("width"))?,
            height: i64_at(raw, jpath!("height"))?,
        })
    }

    pub(crate) fn list_from_json(raw: &Value) -> Result<Vec<Self>> {
        let list = raw.as_array().ok_or(crate::json::PathError::NotAList)?;
        list.iter().map(Image::from_json).collect()
    }

    pub(crate) fn list_from_json_or(raw: Option<&Value>) -> Result<Vec<Self>> {
        match raw {
            Some(raw) => Image::list_from_json(raw),
            None => Ok(Vec::new()),
        }
    }
}

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}x{})", self.url, self.width, self.height)
    }
}

/// The useful pick out of a thumbnail set.
///
/// ```
/// # use ytsearch::{Image, Images};
/// let thumbnails = vec![
///     Image { url: "small.jpg".into(), width: 120, height: 90 },
///     Image { url: "big.jpg".into(), width: 1280, height: 720 },
/// ];
/// assert_eq!(thumbnails.best_url(), Some("big.jpg"));
/// ```
pub trait Images {
    /// The image with the largest area.
    fn best(&self) -> Option<&Image>;

    /// The url of the image with the largest area.
    fn best_url(&self) -> Option<&str>;
}

impl Images for [Image] {
    fn best(&self) -> Option<&Image> {
        self.iter().max_by_key(|image| image.area())
    }

    fn best_url(&self) -> Option<&str> {
        self.best().map(|image| image.url.as_str())
    }
}
