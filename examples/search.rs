//! `cargo run --example search` — the blocking client.

use ytsearch::{Client, PlaylistId, SearchMode, SearchQuery, VideoId};

fn main() -> Result<(), ytsearch::Error> {
    let yt = Client::new()?;

    // a filtered search: the cursor fetches the first page and every page after it
    let mut pages = yt.search_pages(SearchQuery::new("minecraft").mode(SearchMode::Video));
    if let Some(page) = pages.next_page()? {
        println!("first page: {} results", page.results.len());
        for video in page.videos().take(3) {
            println!(
                "  {} | {} | {:?} views",
                video.id, video.title, video.view_count
            );
        }
    }

    // a video, typed and then fetched
    let id = "6sFi_F3DJX4".parse::<VideoId>()?;
    match yt.video(&id) {
        Ok(video) => println!(
            "{}\n  {} | {:?} | {:?}",
            video.title, video.channel_name, video.length, video.view_count
        ),
        Err(ytsearch::Error::VideoUnavailable(state)) => println!("{id} is not available: {state}"),
        Err(error) => return Err(error),
    }

    // a channel's uploads, page by page
    let uploads = "UUDrAR1OWC2MD4s0JLetN0MA".parse::<PlaylistId>()?;
    let mut pages = yt.playlist_pages(&uploads, None);
    if let Some(page) = pages.next_page()? {
        println!(
            "{} — {} items, {} views",
            page.title,
            page.items.len(),
            page.view_count.unwrap_or(0)
        );
        for item in page.items.iter().take(3) {
            println!("  [{}] {} ({:?})", item.index, item.title, item.length);
        }
    }

    Ok(())
}
