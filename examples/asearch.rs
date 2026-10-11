//! `cargo run --example asearch` — the async client, with a locale and the
//! hops that follow from a result.

use ytsearch::{AsyncClient, CommentPage, Language, Locale, Region, VideoId};

#[tokio::main]
async fn main() -> Result<(), ytsearch::Error> {
    let korean = Locale::new(Some(Language::Korean), Some(Region::SouthKorea));
    let yt = AsyncClient::new()?.with_locale(korean);

    let mut pages = yt.search_pages("마인크래프트");
    if let Some(page) = pages.next_page().await? {
        println!("{} results", page.results.len());
        for video in page.videos().take(3) {
            println!("  {} | {}", video.id, video.title);
        }
    }

    // a handle is not an id: this costs the extra resolution request
    let channel = yt.channel("@sleepground").await?;
    println!(
        "{} — {:?} subscribers, {:?} videos",
        channel.title, channel.subscriber_count, channel.video_count
    );

    // the about page hangs off the token the channel page carried
    let detail = yt.channel_detail(&channel).await?;
    println!(
        "joined {:?} | {:?} views | {} links",
        detail.joined,
        detail.view_count,
        detail.links.len()
    );

    let id = "6sFi_F3DJX4".parse::<VideoId>()?;
    let mut comments = yt.comment_pages(&id);
    if let Some(CommentPage { comments }) = comments.next_page().await? {
        println!("{} comments", comments.len());
        if let Some(first) = comments.first() {
            println!("  {}: {}", first.author, first.text);
        }
    }

    for suggestion in yt.suggestions("마인").await?.items.iter().take(3) {
        println!("  suggestion: {}", suggestion.text);
    }

    Ok(())
}
