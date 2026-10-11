//! The parsing gate: every fixture, parsed, asserted.
//!
//! The fixtures are the ones the Python client of this surface ships with, so
//! these are its assertions — what the redesign must keep saying afterwards.

use std::path::PathBuf;

use serde_json::Value;

use super::channel::{ChannelDetailPage, ChannelPage};
use super::player::{PlayerNextPage, PlayerPage, PlayerState};
use super::playlist::{PlaylistPage, PlaylistVideoComponent};
use super::search_result::{
    ChannelComponent, PlaylistComponent, SearchResultComponent, SearchResultPage, VideoComponent,
};
use super::CommentsPage;
use crate::model::Image;

fn fixture(name: &str) -> Value {
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "tests", "fixtures", name]
        .iter()
        .collect();
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("{} is not json: {error}", path.display()))
}

fn sizes(thumbnails: &[Image]) -> Vec<(i64, i64)> {
    thumbnails
        .iter()
        .map(|thumbnail| (thumbnail.width, thumbnail.height))
        .collect()
}

fn video_component(component: &SearchResultComponent) -> &VideoComponent {
    match component {
        SearchResultComponent::Video(component) => component,
        other => panic!("expected a video row, got {other:?}"),
    }
}

fn playlist_component(component: &SearchResultComponent) -> &PlaylistComponent {
    match component {
        SearchResultComponent::Playlist(component) => component,
        other => panic!("expected a playlist row, got {other:?}"),
    }
}

// -- components ------------------------------------------------------------

#[test]
fn a_channel_renderer_is_read() {
    let component = ChannelComponent::from_json(&fixture("channel_renderer.json")).unwrap();

    assert_eq!(component.id, "UCZRcA_nO3Kt-SOml2mY8D9g");
    assert_eq!(component.title, "test");
    assert_eq!(component.url, "https://youtube.com/@tutorialismic");
    assert_eq!(component.subscribers_count_text, "264K subscribers");
    assert!(component
        .description_snippet
        .as_deref()
        .unwrap()
        .starts_with("Get your own difficulty faces!"));
    assert_eq!(sizes(&component.thumbnails), vec![(88, 88), (176, 176)]);
}

#[test]
fn a_channel_renderer_that_is_not_one_is_an_error() {
    let invalid = serde_json::json!({ "not_a_channel_renderer": {} });

    assert!(ChannelComponent::from_json(&invalid).is_err());
}

#[test]
fn a_playlist_lockup_is_read() {
    let component = PlaylistComponent::from_json(&fixture("playlist_lockup.json")).unwrap();

    assert_eq!(component.id, "PL_DD814i0-aIqCZe6S78WvbFvDB3E5y4z");
    assert_eq!(component.title, "[LOL 롤] 리그 오브 레전드");
    assert_eq!(
        component.url,
        "https://youtube.com/playlist?list=PL_DD814i0-aIqCZe6S78WvbFvDB3E5y4z"
    );
    assert_eq!(
        component.channel_id.as_deref(),
        Some("UCBvApW8Oem0ZaYEH9wIja5Q")
    );
    assert_eq!(
        component.channel_name.as_deref(),
        Some("메가게임 - Mega Game")
    );
    assert_eq!(sizes(&component.thumbnails), vec![(480, 270)]);
}

#[test]
fn a_lockup_of_another_content_type_is_an_error() {
    let invalid = serde_json::json!({
        "lockupViewModel": { "contentType": "LOCKUP_CONTENT_TYPE_VIDEO" }
    });

    assert!(PlaylistComponent::from_json(&invalid).is_err());
}

#[test]
fn a_playlist_video_lockup_is_read() {
    let component =
        PlaylistVideoComponent::from_json(&fixture("playlist_video_lockup.json"), 0).unwrap();

    assert_eq!(component.id, "6sFi_F3DJX4");
    assert_eq!(
        component.title,
        "세상을 구하기 위해 떠나는 공룡서버 ㄷㄷㄷㄷㄷㄷㄷ"
    );
    assert_eq!(component.url, "https://youtube.com/watch?v=6sFi_F3DJX4");
    assert_eq!(component.index, 0);
    assert_eq!(component.owner_text, "각별");
    assert_eq!(component.length_text, "21:34");
    assert_eq!(
        sizes(&component.thumbnails),
        vec![(168, 94), (196, 110), (246, 138), (336, 188)]
    );
}

#[test]
fn a_video_renderer_is_read() {
    let component = VideoComponent::from_json(&fixture("video_renderer.json")).unwrap();

    assert_eq!(component.id, "F6EldAIgOMc");
    assert_eq!(
        component.title,
        "Wonderful OLED TEST 16K Video | Dolby Vision HDR 120FPS (8K/4K Video TV)"
    );
    assert!(component
        .url
        .starts_with("https://youtube.com/watch?v=F6EldAIgOMc"));
    assert!(!component.is_shorts);
    assert_eq!(component.published_time_text.as_deref(), Some("1 day ago"));
    assert_eq!(component.length_text.as_deref(), Some("1:01:16"));
    assert_eq!(component.view_count_text.as_deref(), Some("20,782 views"));
    assert_eq!(
        component.channel_url,
        "https://youtube.com/@12KVIDEOULTRAHD."
    );
    assert_eq!(component.channel_id, "UCe4A97DMN-5zPaxnS-TqN_w");
    assert_eq!(sizes(&component.thumbnails), vec![(360, 202), (720, 404)]);
}

#[test]
fn a_shorts_renderer_is_read_as_a_short() {
    let component = VideoComponent::from_json(&fixture("shorts_video_renderer.json")).unwrap();

    assert_eq!(component.id, "plKc6fXYMTE");
    assert!(component
        .url
        .starts_with("https://youtube.com/shorts/plKc6fXYMTE"));
    assert!(component.is_shorts);
    assert_eq!(component.channel_id, "UCBSem0KlPhoPK5M8cZWCS4w");
}

// -- pages -----------------------------------------------------------------

#[test]
fn a_channel_page_is_read() {
    let page = ChannelPage::from_json(&fixture("channel_page.json"))
        .unwrap()
        .unwrap();

    assert_eq!(page.id, "UCDrAR1OWC2MD4s0JLetN0MA");
    assert_eq!(page.title, "각별");
    assert_eq!(page.handle_name, "@각별");
    assert_eq!(
        page.url,
        "https://www.youtube.com/channel/UCDrAR1OWC2MD4s0JLetN0MA"
    );
    assert_eq!(page.description, "마인크래프트 합니다.");
    assert_eq!(page.subscriber_count_text, "573K subscribers");
    assert_eq!(page.video_count_text, "422 videos");
    assert_eq!(page.avatar_thumbnails.len(), 1);
    assert_eq!(page.banners.len(), 6);
    assert!(page.is_family_safe);
    assert_eq!(page.tags.len(), 2);
    assert!(page.continuation_token.len() > 50);
}

#[test]
fn a_channel_page_without_a_banner_is_still_a_page() {
    let page = ChannelPage::from_json(&fixture("channel_page_no_banner.json"))
        .unwrap()
        .unwrap();

    assert_eq!(page.title, "각별");
    assert!(page.banners.is_empty());
}

#[test]
fn a_channel_detail_page_is_read() {
    let page = ChannelDetailPage::from_json(&fixture("channel_detail_page.json")).unwrap();

    assert_eq!(page.id, "UCg7rkxrTnIhiHEpXY1ec9NA");
    assert_eq!(page.url, "http://www.youtube.com/@sleepground");
    assert_eq!(page.country, "대한민국");
    assert_eq!(page.subscriber_count_text, "구독자 238만명");
    assert_eq!(page.view_count_text, "조회수 4,372,270,606회");
    assert_eq!(page.joined_date_text, "가입일: 2014. 5. 23.");
    assert_eq!(page.video_count_text, "동영상 4,514개");
    assert_eq!(page.links.len(), 3);
}

#[test]
fn a_comments_page_is_read() {
    let page = CommentsPage::from_json(&fixture("continuation_comments_page.json")).unwrap();

    assert_eq!(page.comments.len(), 20);
    assert!(page.continuation_token.as_deref().unwrap().len() > 50);

    let first = &page.comments[0];
    assert_eq!(first.author_display_name, "@허오키");
    assert!(first.comment.starts_with("89299440분 저거 169.9년임ㅋㅋ"));
}

#[test]
fn a_search_result_page_is_read() {
    let page = SearchResultPage::from_json(&fixture("search_result_page.json")).unwrap();

    assert_eq!(page.components.len(), 30);
    assert!(page
        .components
        .iter()
        .all(|component| matches!(component, SearchResultComponent::Video(_))));
    assert!(page.continuation_token.as_deref().unwrap().len() > 50);

    let first = video_component(&page.components[0]);
    assert_eq!(first.id, "OegyYwm6rqE");
    assert!(first
        .title
        .starts_with("【BGM】Top 50 Most Popular Songs by NCS"));
    assert_eq!(first.view_count_text.as_deref(), Some("9,553,170 views"));
    assert_eq!(first.channel_id, "UCwwtUfy0-CqN50HfaFDzL0w");
}

#[test]
fn a_search_result_continuation_page_mixes_rows() {
    let page =
        SearchResultPage::from_json(&fixture("search_result_continuation_page.json")).unwrap();

    assert_eq!(page.components.len(), 20);
    assert_eq!(
        page.components
            .iter()
            .filter(|component| matches!(component, SearchResultComponent::Video(_)))
            .count(),
        19
    );
    assert_eq!(
        page.components
            .iter()
            .filter(|component| matches!(component, SearchResultComponent::Playlist(_)))
            .count(),
        1
    );
    assert!(page.continuation_token.as_deref().unwrap().len() > 50);
}

#[test]
fn a_playlist_search_page_is_read() {
    let page = SearchResultPage::from_json(&fixture("search_result_playlist_page.json")).unwrap();

    assert_eq!(page.components.len(), 20);
    assert!(page
        .components
        .iter()
        .all(|component| matches!(component, SearchResultComponent::Playlist(_))));

    let first = playlist_component(&page.components[0]);
    assert_eq!(first.id, "PL_DD814i0-aIqCZe6S78WvbFvDB3E5y4z");
    assert_eq!(first.title, "[LOL 롤] 리그 오브 레전드");
    assert_eq!(first.channel_name.as_deref(), Some("메가게임 - Mega Game"));
}

#[test]
fn a_nested_search_result_page_is_flattened() {
    // a shelf holds nested rows: videos and one playlist lockup
    let page = SearchResultPage::from_json(&fixture("nested_search_result.json")).unwrap();

    assert_eq!(page.components.len(), 14);
    assert_eq!(
        page.components
            .iter()
            .filter(|component| matches!(component, SearchResultComponent::Video(_)))
            .count(),
        13
    );
    assert!(page.continuation_token.as_deref().unwrap().len() > 50);

    let first = video_component(&page.components[0]);
    assert_eq!(first.id, "kijpcUv-b8M");
    assert_eq!(first.title, "Queen - Somebody To Love (Official Video)");
}

#[test]
fn a_playlist_page_is_read() {
    let page = PlaylistPage::from_json(&fixture("playlist_page.json"))
        .unwrap()
        .unwrap();

    assert_eq!(page.id, "UUDrAR1OWC2MD4s0JLetN0MA");
    assert_eq!(page.title, "Uploads from 각별");
    assert_eq!(page.owner_text, "각별");
    assert_eq!(page.owner_id, "UCDrAR1OWC2MD4s0JLetN0MA");
    assert_eq!(page.view_count_text, "1,692,286 views");
    assert_eq!(page.videos.len(), 100);
    assert!(page.continuation_token.as_deref().unwrap().len() > 50);

    let first = &page.videos[0];
    assert_eq!(first.id, "6sFi_F3DJX4");
    assert_eq!(first.index, 0);
    assert_eq!(first.owner_text, "각별");
}

#[test]
fn a_playlist_continuation_page_ends_the_playlist() {
    let page = PlaylistPage::from_json(&fixture("playlist_continuation_page.json"))
        .unwrap()
        .unwrap();

    assert_eq!(page.videos.len(), 100);
    assert!(page.continuation_token.is_none());

    let first = &page.videos[0];
    assert_eq!(first.id, "EGSk5OrgAzU");
    assert_eq!(first.index, 0);
}

#[test]
fn a_video_next_page_carries_the_comment_token() {
    let page = PlayerNextPage::from_json(&fixture("player_next_page.json")).unwrap();

    assert!(page.comment_continuation_token.len() > 50);
}

// -- player pages ----------------------------------------------------------

#[test]
fn a_player_page_is_read() {
    // the fixture carries videoDetails + microformat but no playabilityStatus
    let (page, availability) = PlayerPage::from_json(&fixture("player_page.json")).unwrap();

    let page = page.expect("the fixture is a complete player page");
    assert_eq!(availability.state, PlayerState::Unknown);
    assert_eq!(availability.reason, None);
    assert_eq!(page.id, "z0GKGpObgPY");
    assert_eq!(page.title, "Harry Styles - Sign of the Times (Audio)");
    assert_eq!(page.url, "https://www.youtube.com/watch?v=z0GKGpObgPY");
    assert_eq!(page.length_seconds, 342);
    assert_eq!(page.view_count, 177_836_495);
    assert!(page.description.starts_with("Official audio for"));
    assert_eq!(page.channel_id, "UCbOCbp5gXL8jigIBZLqMPrw");
    assert_eq!(page.channel_name, "HarryStylesVEVO");
    assert!(!page.is_live_content);
    assert!(page.is_family_safe);
    assert_eq!(page.category, "Music");
    assert_eq!(page.thumbnails.len(), 5);
}

#[test]
fn a_player_page_without_a_microformat_still_reports_its_state() {
    // the pot-free clients answer OK but omit microformat
    let (page, availability) = PlayerPage::from_json(&fixture("player_page_ok.json")).unwrap();

    assert!(page.is_none());
    assert_eq!(availability.state, PlayerState::Ok);
    assert_eq!(availability.reason, None);
}

#[test]
fn a_player_page_of_an_unavailable_video_reports_error() {
    let (page, availability) =
        PlayerPage::from_json(&fixture("player_page_unavailable.json")).unwrap();

    assert!(page.is_none());
    assert_eq!(availability.state, PlayerState::Error);
    assert_eq!(
        availability.reason.as_deref(),
        Some("This video is unavailable")
    );
}

#[test]
fn a_player_page_of_a_gated_video_reports_login_required() {
    let (page, availability) = PlayerPage::from_json(&fixture("player_page_login.json")).unwrap();

    assert!(page.is_none());
    assert_eq!(availability.state, PlayerState::LoginRequired);
    assert!(availability.reason.unwrap().contains("not a bot"));
}

#[test]
fn a_player_page_of_a_live_recording_reports_unplayable() {
    let (page, availability) =
        PlayerPage::from_json(&fixture("player_page_live_recording.json")).unwrap();

    assert!(page.is_none());
    assert_eq!(availability.state, PlayerState::Unplayable);
    assert_eq!(
        availability.reason.as_deref(),
        Some("This live stream recording is not available.")
    );
}

#[test]
fn a_player_page_of_a_members_only_video_is_partial_but_reports_its_state() {
    let (page, availability) = PlayerPage::from_json(&fixture("player_page_members.json")).unwrap();

    assert!(page.is_none());
    assert_eq!(availability.state, PlayerState::Unplayable);
    assert_eq!(
        availability.reason.as_deref(),
        Some(
            "Join this channel from your computer or mobile app to get access \
             to members-only content like this video."
        )
    );
}

#[test]
fn a_player_page_of_unrelated_data_is_an_error() {
    assert!(PlayerPage::from_json(&serde_json::json!({ "unrelated": 1 })).is_err());
}

// -- pages of nothing ------------------------------------------------------

#[test]
fn pages_of_unrelated_data_are_empty_rather_than_errors() {
    let page = SearchResultPage::from_json(&serde_json::json!({ "unrelated": 1 })).unwrap();
    assert!(page.components.is_empty());
    assert!(page.continuation_token.is_none());

    let page = SearchResultPage::from_json(&serde_json::json!({})).unwrap();
    assert!(page.components.is_empty());

    let page = CommentsPage::from_json(&serde_json::json!({})).unwrap();
    assert!(page.comments.is_empty());
    assert!(page.continuation_token.is_none());
}

#[test]
fn the_state_tokens_are_known() {
    assert_eq!(PlayerState::from_token("OK"), PlayerState::Ok);
    assert_eq!(
        PlayerState::from_token("LOGIN_REQUIRED"),
        PlayerState::LoginRequired
    );
    assert_eq!(
        PlayerState::from_token("LIVE_STREAM_OFFLINE"),
        PlayerState::LiveStreamOffline
    );
    assert_eq!(PlayerState::from_token("NOPE"), PlayerState::Unknown);
}
