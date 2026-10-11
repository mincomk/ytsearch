Youtube
  Search.
  Rust, idiomatically.

Search YouTube, and read videos, channels, playlists and comments, without the
YouTube Data API and without an API key.

A clean rewrite of the same surface as [yspy](https://github.com/ityeri/yspy)
(and of the line-by-line port of it, [ysrs](https://github.com/mincomk/ysrs)) —
same endpoints, same parsing rules, an API designed for Rust.

- **One handle.** A [`Client`] owns the transport and the default locale, so no
  call threads either of them through.
- **Typed ids.** `VideoId`, `ChannelId`, `PlaylistId`, `ChannelHandle` — parsed
  once, at the edge, URLs included.
- **Cursors, not tokens.** `search_pages`, `playlist_pages`, `comment_pages`
  return something you can walk; the continuation token stays inside.
- **Typed data.** Counts are `u64`, lengths are `Duration`, dates are
  `DateTime` — the page's text is read once, not handed on.
- **Sync *or* async, by feature.** Neither surface costs you the other.

## Install

```toml
[dependencies]
ytsearch = "0.1"                       # the blocking client
# ytsearch = { version = "0.1", default-features = false, features = ["async"] }
```

```rust
use ytsearch::{Client, VideoId};

let yt = Client::new()?;

let mut pages = yt.search_pages("minecraft");
while let Some(page) = pages.next_page()? {
    for result in &page.results {
        println!("{}", result.title());
    }
}

let id = "6sFi_F3DJX4".parse::<VideoId>()?;   // an id or any URL carrying one
match yt.video(&id) {
    Ok(video) => println!("{} — {:?} views", video.title, video.view_count),
    Err(ytsearch::Error::VideoUnavailable(state)) => println!("not available: {state}"),
    Err(error) => return Err(error),
}
```

The async twin is the same API with `.await`:

```rust
use ytsearch::{AsyncClient, Language, Locale, Region, VideoId};

let korean = Locale::new(Some(Language::Korean), Some(Region::SouthKorea));
let yt = AsyncClient::new()?.with_locale(korean);

let id = "6sFi_F3DJX4".parse::<VideoId>()?;
let mut comments = yt.comment_pages(&id);
while let Some(page) = comments.next_page().await? {
    for comment in &page.comments {
        println!("{}: {}", comment.author, comment.text);
    }
}
```

## What the API looks like

### Search

```rust
use ytsearch::{SearchMode, SearchQuery};

let query = SearchQuery::new("minecraft")
    .mode(SearchMode::Video)          // or Channel, Playlist, Livestream
    .locale(my_korean_locale);        // this query only; the client's locale otherwise

let mut pages = yt.search_pages(query);
let first = pages.next_page()?;       // Ok(None) once the results run out
```

`SearchPage::videos()`, `channels()` and `playlists()` filter the page; a row is
a `SearchResult` enum with `as_video()` / `as_channel()` / `as_playlist()`, and
every row carries its typed id, so a row can be followed to its own page:

```rust
let video = yt.video(&page.videos().next().unwrap().id)?;
```

Because the client lives in a variable, a chained search is two calls, not one
expression — which is the point: nothing is smuggled through a hidden field.

### Videos and availability

```rust
let state = yt.video_state(&id)?;     // the verdict alone
if state.is_available() { /* ... */ }
```

Unavailable states are `MembersOnly`, `RecordingUnavailable`, `AgeRestricted`,
`BotDetection`, `LoginRequired`, `RegionBlocked`, `CopyrightBlocked`, `Private`,
`RemovedByUploader`, `AccountTerminated`, `RemovedForTos`, `Unavailable`. They
are classified from the player's own reason, and a page that comes back
unavailable is re-probed with pot-free innertube clients in an English locale,
so the reason is a stable one — the same fallback chain as the Python client,
walked one request at a time and only as far as it has to go.

### Channels

```rust
let channel = yt.channel("@sleepground")?;          // a handle: one resolve request
let channel = yt.channel(&channel_id)?;             // an id: none
let channel = yt.channel("UCg7rkxrTnIhiHEpXY1ec9NA")?;   // or a raw id/handle/URL

let detail = yt.channel_detail(&channel)?;          // about page: joined, views, links
```

`channel_detail` takes the fetched channel because the about page needs a token
the channel page carried — data flows in, not out of thin air.

### Playlists and comments

```rust
let playlist = "UUDrAR1OWC2MD4s0JLetN0MA".parse::<PlaylistId>()?;
let mut pages = yt.playlist_pages(&playlist, None);
while let Some(page) = pages.next_page()? {
    println!("page {} of {} items", page.page_index, page.items.len());
    for item in &page.items {
        println!("[{}] {} ({:?})", item.index, item.title, item.length);
    }
}
```

`items` carry their position in the *whole* playlist (`index`), so a walk across
pages stays usable. Comments work the same way through `comment_pages`.

### Suggestions

```rust
for item in yt.suggestions("mine")?.items {
    println!("{}", item.text);
}
```

### Locale

```rust
use ytsearch::{Language, Locale, Region};

let korean = Locale::new(Some(Language::Korean), Some(Region::SouthKorea));
let yt = Client::new()?.with_locale(korean);
```

Counts are only abbreviated in the English rendering (`573K subscribers`), so a
non-English call fetches the English page alongside the requested one and reads
numbers from there — the same rule, and the same two requests, as the Python
client.

## Features

| feature | what it compiles | entry point |
| --- | --- | --- |
| `blocking` (default) | the synchronous surface | `Client` |
| `async` | the asynchronous surface | `AsyncClient` |

They are independent: `default-features = false, features = ["async"]` builds
only the async surface, and neither pulls the other in.

The blocking client owns a runtime of its own (it is `reqwest`'s blocking
client), so **do not use `Client` from inside an async runtime** — use
`AsyncClient` there. This is the reason the two are separate features rather
than one type with two method families.

## How it is put together

```
src/request.rs   endpoints, payloads, headers — plain data, no I/O
src/step.rs      the request sequences as state machines, no I/O
src/parsing/     json into structs, no I/O, no client, no locale
src/model/       what you get back: ids, counts, durations, dates
src/blocking.rs  the blocking driver + its cursors
src/asynchronous.rs  the async driver + its cursors
```

Every operation that takes more than one request — the availability probe, the
English-page pairing, entering the comments through the watch page — is written
once, as a machine that takes a response and says what to do next. The two
clients are thin drivers over those machines, which is why they cannot drift
apart, and why the tests can run the whole sequence against a scripted transport
with no network at all:

```console
$ cargo test
running 41 tests ... ok
```

The parsing is pinned to the fixtures the Python client ships with
(`tests/fixtures/`), so the redesign keeps saying what the original said.

## License

MIT — see [LICENSE](LICENSE), and [NOTICE](NOTICE) for what came from where.
