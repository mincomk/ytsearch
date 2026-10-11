//! Parsing rules carried over from yspy (see NOTICE).

mod player_next_page;
mod player_page;

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime};

use crate::error::{Error, Result};

pub use player_next_page::PlayerNextPage;
pub use player_page::{PlayerAvailability, PlayerPage, PlayerState};

/// `datetime.fromisoformat(...)` — an RFC 3339 stamp, and the two looser shapes
/// YouTube also answers (`2026-07-18T03:02:32`, `2026-07-18`).
pub(crate) fn parse_iso_datetime(text: &str) -> Result<DateTime<FixedOffset>> {
    const UTC: fn() -> FixedOffset = || FixedOffset::east_opt(0).expect("UTC is a valid offset");

    if let Ok(datetime) = DateTime::parse_from_rfc3339(text) {
        return Ok(datetime);
    }

    if let Ok(naive) = NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S") {
        return Ok(DateTime::from_naive_utc_and_offset(naive, UTC()));
    }

    if let Ok(date) = NaiveDate::parse_from_str(text, "%Y-%m-%d") {
        let naive = date.and_hms_opt(0, 0, 0).expect("midnight is a valid time");
        return Ok(DateTime::from_naive_utc_and_offset(naive, UTC()));
    }

    Err(Error::parse(format!(
        "The given date is not an ISO 8601 date: {text}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_shapes_youtube_answers() {
        assert_eq!(
            parse_iso_datetime("2017-04-07T00:00:03-07:00")
                .unwrap()
                .to_rfc3339(),
            "2017-04-07T00:00:03-07:00"
        );
        assert_eq!(
            parse_iso_datetime("2026-07-18T03:02:32")
                .unwrap()
                .to_rfc3339(),
            "2026-07-18T03:02:32+00:00"
        );
        assert_eq!(
            parse_iso_datetime("2026-07-18").unwrap().to_rfc3339(),
            "2026-07-18T00:00:00+00:00"
        );
        assert!(parse_iso_datetime("가입일: 2014. 5. 23.").is_err());
    }
}
