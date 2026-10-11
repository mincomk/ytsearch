//! The counts, lengths and dates YouTube renders as text, read back.

use std::sync::LazyLock;
use std::time::Duration;

use chrono::NaiveDate;
use regex::Regex;

static COUNT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?P<number>\d+(?:[.,]\d+)?)\s*(?P<unit>[kmb]|[a-z]+)?").expect("valid regex")
});

static NO_SUBSCRIBERS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bno\s+subscribers?\b").expect("valid regex"));

static NO_VIEWS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bno\s+views?\b").expect("valid regex"));

static NO_VIDEOS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bno\s+videos?\b").expect("valid regex"));

static JOINED_DATE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([A-Za-z]{3,9})\s+(\d{1,2}),?\s+(\d{4})").expect("valid regex"));

static LENGTH_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:(\d+):)?(\d+):(\d{2})$").expect("valid regex"));

/// A unit, or its first letter, or a plain count.
fn count_unit(unit: &str) -> Option<u64> {
    match unit {
        "k" | "thousand" => Some(1_000),
        "m" | "million" => Some(1_000_000),
        "b" | "billion" => Some(1_000_000_000),
        _ => match unit.chars().next() {
            Some('k') => Some(1_000),
            Some('m') => Some(1_000_000),
            Some('b') => Some(1_000_000_000),
            _ => None,
        },
    }
}

fn parse_count(text: Option<&str>) -> Option<u64> {
    let text = text?;
    if text.is_empty() {
        return None;
    }

    let cleaned = text.replace(',', "");
    let captures = COUNT_PATTERN.captures(&cleaned)?;

    let number: f64 = captures.name("number")?.as_str().parse().ok()?;
    let unit = captures
        .name("unit")
        .map(|unit| unit.as_str().to_lowercase())
        .unwrap_or_default();
    let multiplier = count_unit(&unit).unwrap_or(1) as f64;

    let count = number * multiplier;
    if count.is_sign_negative() {
        return None;
    }

    Some(count as u64)
}

/// `573K subscribers`, `264 thousand subscribers`, `No subscribers`.
pub(crate) fn parse_subscriber_count(text: Option<&str>) -> Option<u64> {
    let text = text?;
    if text.is_empty() {
        return None;
    }
    if NO_SUBSCRIBERS.is_match(text) {
        return Some(0);
    }
    parse_count(Some(text))
}

/// `9,553,170 views`, `조회수 4,372,270,606회`, `No views`.
pub(crate) fn parse_view_count(text: Option<&str>) -> Option<u64> {
    let text = text?;
    if text.is_empty() {
        return None;
    }
    if NO_VIEWS.is_match(text) {
        return Some(0);
    }
    parse_count(Some(text))
}

/// `422 videos`, `No videos`.
pub(crate) fn parse_video_count(text: Option<&str>) -> Option<u64> {
    let text = text?;
    if text.is_empty() {
        return None;
    }
    if NO_VIDEOS.is_match(text) {
        return Some(0);
    }
    parse_count(Some(text))
}

fn month_of(name: &str) -> Option<u32> {
    match name {
        "jan" => Some(1),
        "feb" => Some(2),
        "mar" => Some(3),
        "apr" => Some(4),
        "may" => Some(5),
        "jun" => Some(6),
        "jul" => Some(7),
        "aug" => Some(8),
        "sep" => Some(9),
        "oct" => Some(10),
        "nov" => Some(11),
        "dec" => Some(12),
        _ => None,
    }
}

/// `Joined May 23, 2014` — only the English rendering parses, so a localized one
/// (`가입일: 2014. 5. 23.`) is `None`.
pub(crate) fn parse_joined_date(text: Option<&str>) -> Option<NaiveDate> {
    let text = text?;

    let captures = JOINED_DATE_PATTERN.captures(text)?;
    let month_name = captures.get(1)?.as_str().to_lowercase();
    let month = month_of(month_name.get(..3)?)?;
    let day: u32 = captures.get(2)?.as_str().parse().ok()?;
    let year: i32 = captures.get(3)?.as_str().parse().ok()?;

    NaiveDate::from_ymd_opt(year, month, day)
}

/// `21:34`, `1:01:16` — the video length is locale-independent.
pub(crate) fn parse_length_seconds(length_text: &str) -> Option<u64> {
    let captures = LENGTH_PATTERN.captures(length_text)?;

    let hours = match captures.get(1) {
        Some(hours) => hours.as_str().parse::<u64>().ok()?,
        None => 0,
    };
    let minutes: u64 = captures.get(2)?.as_str().parse().ok()?;
    let seconds: u64 = captures.get(3)?.as_str().parse().ok()?;

    Some(hours * 3600 + minutes * 60 + seconds)
}

/// The length, as a `Duration`.
pub(crate) fn parse_duration(length_text: Option<&str>) -> Option<Duration> {
    Some(Duration::from_secs(parse_length_seconds(length_text?)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_come_back_as_numbers() {
        assert_eq!(
            parse_subscriber_count(Some("573K subscribers")),
            Some(573_000)
        );
        assert_eq!(
            parse_subscriber_count(Some("264K subscribers")),
            Some(264_000)
        );
        assert_eq!(
            parse_subscriber_count(Some("264 thousand subscribers")),
            Some(264_000)
        );
        assert_eq!(
            parse_subscriber_count(Some("1.2M subscribers")),
            Some(1_200_000)
        );
        assert_eq!(parse_subscriber_count(Some("No subscribers")), Some(0));
        assert_eq!(parse_view_count(Some("9,553,170 views")), Some(9_553_170));
        assert_eq!(parse_view_count(Some("1,692,286 views")), Some(1_692_286));
        assert_eq!(
            parse_view_count(Some("조회수 4,372,270,606회")),
            Some(4_372_270_606)
        );
        assert_eq!(parse_view_count(Some("No views")), Some(0));
        assert_eq!(parse_video_count(Some("422 videos")), Some(422));
        assert_eq!(parse_video_count(Some("No videos")), Some(0));
        assert_eq!(parse_subscriber_count(None), None);
    }

    #[test]
    fn dates_and_lengths_come_back_typed() {
        assert_eq!(
            parse_joined_date(Some("Joined May 23, 2014")),
            NaiveDate::from_ymd_opt(2014, 5, 23)
        );
        assert_eq!(parse_joined_date(Some("가입일: 2014. 5. 23.")), None);
        assert_eq!(parse_length_seconds("21:34"), Some(1294));
        assert_eq!(parse_length_seconds("1:01:16"), Some(3676));
        assert_eq!(parse_length_seconds("nope"), None);
        assert_eq!(
            parse_duration(Some("21:34")),
            Some(Duration::from_secs(1294))
        );
    }
}
