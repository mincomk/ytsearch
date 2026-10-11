//! The locale a request is made in, and the search modes.

use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};

/// A language, spelled the way the `hl` parameter spells it.
///
/// ```
/// # use ytsearch::Language;
/// assert_eq!(Language::Korean.as_str(), "ko");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Korean,
    English,
    Japanese,
    ChineseSimplified,
    ChineseTraditional,
    Spanish,
    French,
    German,
    Italian,
    Portuguese,
    Russian,
    Vietnamese,
    Thai,
    Indonesian,
    Hindi,
    Arabic,
}

impl Language {
    /// The value YouTube expects (`Language::Korean == "ko"`).
    pub const fn as_str(self) -> &'static str {
        match self {
            Language::Korean => "ko",
            Language::English => "en",
            Language::Japanese => "ja",
            Language::ChineseSimplified => "zh-Hans",
            Language::ChineseTraditional => "zh-Hant",
            Language::Spanish => "es",
            Language::French => "fr",
            Language::German => "de",
            Language::Italian => "it",
            Language::Portuguese => "pt",
            Language::Russian => "ru",
            Language::Vietnamese => "vi",
            Language::Thai => "th",
            Language::Indonesian => "id",
            Language::Hindi => "hi",
            Language::Arabic => "ar",
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Language {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        const LANGUAGES: &[Language] = &[
            Language::Korean,
            Language::English,
            Language::Japanese,
            Language::ChineseSimplified,
            Language::ChineseTraditional,
            Language::Spanish,
            Language::French,
            Language::German,
            Language::Italian,
            Language::Portuguese,
            Language::Russian,
            Language::Vietnamese,
            Language::Thai,
            Language::Indonesian,
            Language::Hindi,
            Language::Arabic,
        ];

        LANGUAGES
            .iter()
            .copied()
            .find(|language| language.as_str() == value.trim())
            .ok_or_else(|| Error::parse(format!("unknown language: {value:?}")))
    }
}

/// A region, spelled the way the `gl` parameter spells it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Region {
    SouthKorea,
    UnitedStates,
    Japan,
    UnitedKingdom,
    Canada,
    France,
    Germany,
    Italy,
    Spain,
    Brazil,
    Mexico,
    Australia,
    Vietnam,
    Thailand,
    Indonesia,
    India,
    Taiwan,
    HongKong,
}

impl Region {
    /// The value YouTube expects (`Region::SouthKorea == "KR"`).
    pub const fn as_str(self) -> &'static str {
        match self {
            Region::SouthKorea => "KR",
            Region::UnitedStates => "US",
            Region::Japan => "JP",
            Region::UnitedKingdom => "GB",
            Region::Canada => "CA",
            Region::France => "FR",
            Region::Germany => "DE",
            Region::Italy => "IT",
            Region::Spain => "ES",
            Region::Brazil => "BR",
            Region::Mexico => "MX",
            Region::Australia => "AU",
            Region::Vietnam => "VN",
            Region::Thailand => "TH",
            Region::Indonesia => "ID",
            Region::India => "IN",
            Region::Taiwan => "TW",
            Region::HongKong => "HK",
        }
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A language and/or a region, as one value.
///
/// [`Locale::NONE`] is the default: no `hl`, no `gl`, and YouTube answers in
/// whatever it thinks the caller is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Locale {
    pub language: Option<Language>,
    pub region: Option<Region>,
}

impl Locale {
    /// No `hl`/`gl` at all.
    pub const NONE: Locale = Locale::new(None, None);

    /// English — the rendering every abbreviated count is read from.
    pub const ENGLISH: Locale = Locale::new(Some(Language::English), None);

    /// A language and a region.
    pub const fn new(language: Option<Language>, region: Option<Region>) -> Self {
        Locale { language, region }
    }

    /// Just a language.
    pub const fn language(language: Language) -> Self {
        Locale::new(Some(language), None)
    }

    /// Just a region.
    pub const fn region(region: Region) -> Self {
        Locale::new(None, Some(region))
    }

    /// The same locale, with the language replaced.
    pub const fn with_language(self, language: Language) -> Self {
        Locale::new(Some(language), self.region)
    }

    /// The same locale, with the region replaced.
    pub const fn with_region(self, region: Region) -> Self {
        Locale::new(self.language, Some(region))
    }

    /// Whether reading this locale's counts needs a second, English request.
    ///
    /// Only the English rendering abbreviates a count (`573K subscribers`), so
    /// every count-bearing call fetches the English page alongside the
    /// requested one unless the locale *is* English.
    pub const fn needs_english_page(self) -> bool {
        !matches!(self.language, Some(Language::English))
    }
}

impl From<Language> for Locale {
    fn from(language: Language) -> Self {
        Locale::language(language)
    }
}

impl From<Region> for Locale {
    fn from(region: Region) -> Self {
        Locale::region(region)
    }
}

/// A search filter.
///
/// ```
/// # use ytsearch::SearchMode;
/// assert_eq!(SearchMode::Video.as_params(), "EgIQAQ%3D%3D");
/// assert_eq!(SearchMode::default(), SearchMode::Video);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SearchMode {
    /// Videos only — the default.
    #[default]
    Video,
    /// Channels only.
    Channel,
    /// Playlists only.
    Playlist,
    /// Live streams only.
    Livestream,
}

impl SearchMode {
    /// The percent-encoded `params` value YouTube filters on.
    pub const fn as_params(self) -> &'static str {
        match self {
            SearchMode::Video => "EgIQAQ%3D%3D",
            SearchMode::Channel => "EgIQAg%3D%3D",
            SearchMode::Playlist => "EgIQAw%3D%3D",
            SearchMode::Livestream => "EgJAAQ%3D%3D",
        }
    }
}

impl fmt::Display for SearchMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            SearchMode::Video => "video",
            SearchMode::Channel => "channel",
            SearchMode::Playlist => "playlist",
            SearchMode::Livestream => "livestream",
        })
    }
}
