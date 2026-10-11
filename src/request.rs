//! What to send: endpoints, payloads and headers, as plain data.
//!
//! No I/O happens here. A [`Request`] is what the step machines hand to a
//! transport, and it is the same request the Python client of this surface
//! sends — same endpoints, same key, same client context.

use std::sync::LazyLock;

use serde_json::Value;
use url::Url;

use crate::json::JsonMap;
use crate::json_map;
use crate::locale::Locale;

const BROWSE_API_URL: &str = "https://www.youtube.com/youtubei/v1/browse";
const SEARCH_API_URL: &str = "https://www.youtube.com/youtubei/v1/search";
const PLAYER_API_URL: &str = "https://www.youtube.com/youtubei/v1/player";
const NEXT_API_URL: &str = "https://www.youtube.com/youtubei/v1/next";
const RESOLVE_URL: &str = "https://www.youtube.com/youtubei/v1/navigation/resolve_url";
const SUGGESTION_API_URL: &str = "https://clients1.google.com/complete/search";

/// The public innertube key every request carries.
const BROWSE_KEY: &str = "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
                          AppleWebKit/537.36 (KHTML, like Gecko) \
                          Chrome/70.0.3538.77 Safari/537.36";

fn base_headers() -> Vec<(String, String)> {
    vec![("User-Agent".to_string(), USER_AGENT.to_string())]
}

/// A pot-free innertube client definition, and the headers it needs.
pub(crate) struct InnertubeClient {
    pub(crate) client: JsonMap,
    pub(crate) headers: Vec<(String, String)>,
}

/// The clients the availability probe falls back to when the WEB client —
/// which needs a `po_token` to reach an OK status — will not answer.
///
/// These still answer the player endpoint with a truthful playability status
/// anonymously, and in English, which is what the availability table reads.
pub(crate) static PLAYER_FALLBACK_CLIENTS: LazyLock<Vec<InnertubeClient>> = LazyLock::new(|| {
    const VISIONOS: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 15_7_3) AppleWebKit/605.1.15 \
                            (KHTML, like Gecko) Version/26.0 Safari/605.1.15";
    const TVHTML5: &str = "Mozilla/5.0 (ChromiumStylePlatform) Cobalt/Version";
    const MWEB: &str = "Mozilla/5.0 (iPad; CPU OS 16_7_10 like Mac OS X) AppleWebKit/605.1.15 \
                        (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.1,gzip(gfe)";
    const ANDROID_VR: &str = "com.google.android.apps.youtube.vr.oculus/1.65.10 (Linux; U; \
                              Android 12L; eureka-user Build/SQ3A.220605.009.A1) gzip";

    vec![
        InnertubeClient {
            client: json_map! {
                "clientName" => "VISIONOS",
                "clientVersion" => "1.02",
                "userAgent" => VISIONOS,
                "deviceMake" => "Apple",
                "deviceModel" => "RealityDevice17,1",
                "osName" => "visionOS",
                "osVersion" => "26.5.23O471",
            },
            headers: vec![
                ("User-Agent".to_string(), VISIONOS.to_string()),
                ("X-YouTube-Client-Name".to_string(), "101".to_string()),
                ("X-YouTube-Client-Version".to_string(), "1.02".to_string()),
            ],
        },
        InnertubeClient {
            client: json_map! {
                "clientName" => "TVHTML5",
                "clientVersion" => "5.20260707",
                "userAgent" => TVHTML5,
            },
            headers: vec![
                ("User-Agent".to_string(), TVHTML5.to_string()),
                ("X-YouTube-Client-Name".to_string(), "7".to_string()),
                (
                    "X-YouTube-Client-Version".to_string(),
                    "5.20260707".to_string(),
                ),
            ],
        },
        InnertubeClient {
            client: json_map! {
                "clientName" => "MWEB",
                "clientVersion" => "2.20260708.05.00",
                "userAgent" => MWEB,
            },
            headers: vec![
                ("User-Agent".to_string(), MWEB.to_string()),
                ("X-YouTube-Client-Name".to_string(), "2".to_string()),
                (
                    "X-YouTube-Client-Version".to_string(),
                    "2.20260708.05.00".to_string(),
                ),
            ],
        },
        InnertubeClient {
            client: json_map! {
                "clientName" => "ANDROID_VR",
                "clientVersion" => "1.65.10",
                "userAgent" => ANDROID_VR,
                "deviceMake" => "Oculus",
                "deviceModel" => "Quest 3",
                "osName" => "Android",
                "osVersion" => "12L",
            },
            headers: vec![
                ("User-Agent".to_string(), ANDROID_VR.to_string()),
                ("X-YouTube-Client-Name".to_string(), "28".to_string()),
                (
                    "X-YouTube-Client-Version".to_string(),
                    "1.65.10".to_string(),
                ),
            ],
        },
    ]
});

/// The HTTP verb of a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Method {
    Get,
    Post,
}

impl Method {
    /// The reqwest method.
    pub(crate) fn as_reqwest(self) -> reqwest::Method {
        match self {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
        }
    }
}

/// One request, ready to send.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Request {
    pub(crate) method: Method,
    pub(crate) url: String,
    /// `None` for a request that carries no json body.
    pub(crate) body: Option<Value>,
    pub(crate) headers: Vec<(String, String)>,
}

impl Request {
    fn url(endpoint: &str, query: &[(&str, &str)]) -> String {
        let mut url = Url::parse(endpoint).expect("endpoint urls are literals");
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("key", BROWSE_KEY);
            for (key, value) in query {
                pairs.append_pair(key, value);
            }
        }
        url.to_string()
    }

    fn post(endpoint: &str, payload: Value) -> Self {
        Request {
            method: Method::Post,
            url: Request::url(endpoint, &[]),
            body: Some(payload),
            headers: base_headers(),
        }
    }

    fn post_as(endpoint: &str, payload: Value, headers: Vec<(String, String)>) -> Self {
        Request {
            method: Method::Post,
            url: Request::url(endpoint, &[]),
            body: Some(payload),
            headers,
        }
    }
}

/// The innertube client context every payload carries.
fn payload(locale: Locale, client: Option<&JsonMap>, params: JsonMap) -> Value {
    let mut client = match client {
        Some(client) => client.clone(),
        None => json_map! {
            "clientName" => "WEB",
            "clientVersion" => "2.20210224.06.00",
            "newVisitorCookie" => true,
        },
    };

    if let Some(language) = locale.language {
        client.insert(
            "hl".to_string(),
            Value::String(language.as_str().to_string()),
        );
    }
    if let Some(region) = locale.region {
        client.insert("gl".to_string(), Value::String(region.as_str().to_string()));
    }

    let mut body = JsonMap::new();
    body.insert(
        "context".to_string(),
        serde_json::json!({
            "client": client,
            "user": { "lockedSafetyMode": false },
        }),
    );
    for (key, value) in params {
        body.insert(key, value);
    }

    Value::Object(body)
}

/// A search, first page or continuation.
pub(crate) fn search_request(
    query: &str,
    search_mode: Option<crate::SearchMode>,
    locale: Locale,
    continuation: Option<&str>,
) -> Request {
    let mut params = match continuation {
        Some(token) => json_map! { "continuation" => token },
        None => json_map! { "query" => query },
    };
    if let Some(mode) = search_mode {
        params.insert(
            "params".to_string(),
            Value::String(mode.as_params().to_string()),
        );
    }

    Request::post(SEARCH_API_URL, payload(locale, None, params))
}

/// A player page, as the WEB client or as one of the fallback clients.
pub(crate) fn player_request(
    video_id: &str,
    locale: Locale,
    client: Option<&InnertubeClient>,
) -> Request {
    let params = json_map! {
        "videoId" => video_id,
        "contentCheckOk" => true,
        "racyCheckOk" => true,
    };
    let body = payload(locale, client.map(|client| &client.client), params);

    match client {
        Some(client) => {
            let mut headers = base_headers();
            headers.extend(client.headers.iter().cloned());
            Request::post_as(PLAYER_API_URL, body, headers)
        }
        None => Request::post(PLAYER_API_URL, body),
    }
}

/// Resolving a `@handle` or a channel URL to a channel id.
pub(crate) fn resolve_channel_request(input: &str) -> Request {
    let full_url = if input.starts_with("http") {
        input.to_string()
    } else if input.starts_with('@') {
        format!("https://www.youtube.com/{input}")
    } else {
        format!("https://www.youtube.com{input}")
    };

    Request::post(
        RESOLVE_URL,
        payload(Locale::NONE, None, json_map! { "url" => full_url }),
    )
}

/// A channel page.
pub(crate) fn channel_request(channel_id: &str, locale: Locale) -> Request {
    Request::post(
        BROWSE_API_URL,
        payload(locale, None, json_map! { "browseId" => channel_id }),
    )
}

/// A channel's about page, by its continuation token.
pub(crate) fn channel_detail_request(token: &str, locale: Locale) -> Request {
    Request::post(
        BROWSE_API_URL,
        payload(locale, None, json_map! { "continuation" => token }),
    )
}

/// A playlist page, by a `VL…` browse id.
pub(crate) fn playlist_request(browse_id: &str, locale: Locale) -> Request {
    Request::post(
        BROWSE_API_URL,
        payload(locale, None, json_map! { "browseId" => browse_id }),
    )
}

/// A playlist's next page, by its continuation token.
pub(crate) fn playlist_continuation_request(token: &str, locale: Locale) -> Request {
    Request::post(
        BROWSE_API_URL,
        payload(locale, None, json_map! { "continuation" => token }),
    )
}

/// A video's watch page, which carries the token the comments hang off.
pub(crate) fn video_next_request(video_id: &str) -> Request {
    Request::post(
        NEXT_API_URL,
        payload(Locale::NONE, None, json_map! { "videoId" => video_id }),
    )
}

/// A batch of comments, by their continuation token.
pub(crate) fn comments_request(token: &str) -> Request {
    Request::post(
        NEXT_API_URL,
        payload(Locale::NONE, None, json_map! { "continuation" => token }),
    )
}

/// Search completions.
pub(crate) fn suggestion_request(query: &str, locale: Locale) -> Request {
    let mut query_params: Vec<(&str, &str)> = vec![
        ("q", query),
        ("client", "youtube"),
        ("gs_ri", "youtube"),
        ("ds", "yt"),
    ];

    if let Some(language) = locale.language {
        query_params.push(("hl", language.as_str()));
    }
    if let Some(region) = locale.region {
        query_params.push(("gl", region.as_str()));
    }

    let mut headers = base_headers();
    headers.push(("Accept-Language".to_string(), "en-US,en;q=0.9".to_string()));

    Request {
        method: Method::Get,
        url: Request::url(SUGGESTION_API_URL, &query_params),
        body: None,
        headers,
    }
}
