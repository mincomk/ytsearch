//! The request sequences, as state machines with no I/O in them.
//!
//! Everything that takes more than one request — and everything whose next
//! request depends on the last response — is written once, here. The blocking
//! and the async client are then two thin drivers over these machines, and the
//! machines themselves run against a scripted transport in the tests.

use serde_json::Value;

use crate::error::Result;
use crate::ids::VideoId;
use crate::locale::Locale;
use crate::model::VideoState;
use crate::parsing::player::{PlayerNextPage, PlayerPage, PlayerState};
use crate::parsing::CommentsPage;
use crate::request::{
    comments_request, player_request, video_next_request, Request, PLAYER_FALLBACK_CLIENTS,
};

/// A step either asks for another request, or is finished.
pub(crate) enum Progress<S: Step> {
    /// Send this request, then hand its response to the state.
    Send(Request, S),
    /// The machine is done.
    Done(S::Output),
}

/// A machine that turns responses into its next move.
pub(crate) trait Step: Sized {
    /// What the machine produces when it is done.
    type Output;

    /// Take one response: ask for the next request, or finish.
    fn advance(self, response: Value) -> Result<Progress<Self>>;
}

/// Drive a machine to its end, sending every request it asks for.
#[cfg_attr(not(feature = "blocking"), allow(dead_code))]
pub(crate) fn drive<S: Step>(
    mut progress: Progress<S>,
    mut send: impl FnMut(Request) -> Result<Value>,
) -> Result<S::Output> {
    loop {
        progress = match progress {
            Progress::Done(output) => return Ok(output),
            Progress::Send(request, state) => state.advance(send(request)?)?,
        };
    }
}

/// The async twin of [`drive`].
#[cfg(feature = "async")]
pub(crate) async fn drive_async<S, F, Fut>(
    mut progress: Progress<S>,
    mut send: F,
) -> Result<S::Output>
where
    S: Step,
    F: FnMut(Request) -> Fut,
    Fut: std::future::Future<Output = Result<Value>>,
{
    loop {
        progress = match progress {
            Progress::Done(output) => return Ok(output),
            Progress::Send(request, state) => state.advance(send(request).await?)?,
        };
    }
}

/// Start a machine that reads a page in the requested locale — and in English
/// too, when the locale is not English.
///
/// Only the English rendering abbreviates a count (`573K subscribers`), so the
/// pair is fetched together and handed over as a pair: the first is what the
/// caller shows, the second is what the counts are read from.
pub(crate) fn localized(
    request: Request,
    locale: Locale,
    english: impl FnOnce() -> Request,
) -> Progress<Localized> {
    let english = locale.needs_english_page().then(english);

    Progress::Send(request, Localized::new(english))
}

/// The machine [`localized`] starts.
pub(crate) struct Localized {
    english: Option<Request>,
    first: Option<Value>,
}

impl Localized {
    fn new(english: Option<Request>) -> Self {
        Localized {
            english,
            first: None,
        }
    }
}

impl Step for Localized {
    /// The requested page and the English page — the same value twice when the
    /// locale is English.
    type Output = (Value, Value);

    fn advance(mut self, response: Value) -> Result<Progress<Self>> {
        match self.first.take() {
            None => match self.english.take() {
                Some(request) => {
                    self.first = Some(response);
                    Ok(Progress::Send(request, self))
                }
                None => Ok(Progress::Done((response.clone(), response))),
            },
            Some(first) => Ok(Progress::Done((first, response))),
        }
    }
}

/// What the availability probe came back with.
pub(crate) enum PlayerOutcome {
    /// The video is watchable.
    Available(Box<PlayerPage>),
    /// It is not, and this is why.
    Unavailable(VideoState),
}

/// Start the availability probe.
pub(crate) fn video_probe(id: &VideoId, locale: Locale) -> Progress<VideoProbe> {
    let probe = VideoProbe::new(id.clone(), locale);
    let request = request(&probe);

    Progress::Send(request, probe)
}

fn request(probe: &VideoProbe) -> Request {
    match probe.stage {
        ProbeStage::Primary => player_request(probe.id.as_str(), probe.locale, None),
        ProbeStage::Fallback(index) => player_request(
            probe.id.as_str(),
            Locale::ENGLISH,
            PLAYER_FALLBACK_CLIENTS.get(index),
        ),
    }
}

/// The availability probe: the WEB player page, then one pot-free client after
/// another until one of them gives a verdict worth reporting.
pub(crate) struct VideoProbe {
    id: VideoId,
    locale: Locale,
    stage: ProbeStage,
}

#[derive(Clone, Copy)]
enum ProbeStage {
    Primary,
    Fallback(usize),
}

impl VideoProbe {
    fn new(id: VideoId, locale: Locale) -> Self {
        VideoProbe {
            id,
            locale,
            stage: ProbeStage::Primary,
        }
    }
}

impl Step for VideoProbe {
    type Output = PlayerOutcome;

    fn advance(mut self, response: Value) -> Result<Progress<Self>> {
        let (page, availability) = PlayerPage::from_json(&response)?;

        match self.stage {
            ProbeStage::Primary => {
                if let Some(page) = page {
                    return Ok(Progress::Done(PlayerOutcome::Available(Box::new(page))));
                }

                // the page is unavailable: ask a pot-free client what happened
                if PLAYER_FALLBACK_CLIENTS.is_empty() {
                    return Ok(Progress::Done(PlayerOutcome::Unavailable(
                        VideoState::Unavailable,
                    )));
                }

                self.stage = ProbeStage::Fallback(0);
                Ok(Progress::Send(request(&self), self))
            }
            ProbeStage::Fallback(index) => {
                // a gate ("the page needs to be reloaded") is not a verdict
                let meaningful = availability.state != PlayerState::Unknown
                    && availability.reason.as_deref() != Some("The page needs to be reloaded.");
                if meaningful {
                    return Ok(Progress::Done(PlayerOutcome::Unavailable(
                        VideoState::from_availability(&availability),
                    )));
                }

                match PLAYER_FALLBACK_CLIENTS.get(index + 1) {
                    Some(_) => {
                        self.stage = ProbeStage::Fallback(index + 1);
                        Ok(Progress::Send(request(&self), self))
                    }
                    // every client gated: keep the last verdict
                    None => Ok(Progress::Done(PlayerOutcome::Unavailable(
                        VideoState::from_availability(&availability),
                    ))),
                }
            }
        }
    }
}

/// Start the comment read: the watch page carries the token the comment thread
/// hangs off, then the comments themselves.
pub(crate) fn comments_first(id: &VideoId) -> Progress<CommentsFirst> {
    let machine = CommentsFirst {
        stage: CommentsStage::Next,
    };

    Progress::Send(video_next_request(id.as_str()), machine)
}

/// The machine [`comments_first`] starts.
pub(crate) struct CommentsFirst {
    stage: CommentsStage,
}

enum CommentsStage {
    Next,
    Comments,
}

impl Step for CommentsFirst {
    type Output = CommentsPage;

    fn advance(mut self, response: Value) -> Result<Progress<Self>> {
        match self.stage {
            CommentsStage::Next => {
                let next = PlayerNextPage::from_json(&response)?;
                self.stage = CommentsStage::Comments;

                Ok(Progress::Send(
                    comments_request(&next.comment_continuation_token),
                    self,
                ))
            }
            CommentsStage::Comments => Ok(Progress::Done(CommentsPage::from_json(&response)?)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::{json, Value};

    use super::*;
    use crate::error::Error;

    fn fixture(name: &str) -> Value {
        let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "tests", "fixtures", name]
            .iter()
            .collect();

        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    /// A transport that answers from a script, and remembers what it was asked.
    struct Script {
        responses: std::vec::IntoIter<Value>,
        sent: Vec<Request>,
    }

    impl Script {
        fn new(responses: Vec<Value>) -> Self {
            Script {
                responses: responses.into_iter(),
                sent: Vec::new(),
            }
        }

        fn send(&mut self, request: Request) -> Result<Value> {
            self.sent.push(request);
            self.responses
                .next()
                .ok_or_else(|| Error::parse("the script ran out of responses"))
        }

        fn client_name(&self, index: usize) -> String {
            self.sent[index].body.as_ref().unwrap()["context"]["client"]["clientName"]
                .as_str()
                .unwrap()
                .to_string()
        }

        fn headers(&self, index: usize) -> Vec<(String, String)> {
            self.sent[index].headers.clone()
        }
    }

    fn id() -> VideoId {
        "6sFi_F3DJX4".parse().unwrap()
    }

    #[test]
    fn an_available_video_costs_one_request() {
        let mut script = Script::new(vec![fixture("player_page.json")]);

        let outcome = drive(video_probe(&id(), Locale::NONE), |request| {
            script.send(request)
        })
        .expect("the probe succeeds");

        assert!(matches!(outcome, PlayerOutcome::Available(_)));
        assert_eq!(script.sent.len(), 1);
        assert_eq!(script.client_name(0), "WEB");
    }

    #[test]
    fn an_unavailable_video_is_probed_again_with_a_pot_free_client() {
        let mut script = Script::new(vec![
            fixture("player_page_unavailable.json"),
            fixture("player_page_unavailable.json"),
        ]);

        let outcome = drive(video_probe(&id(), Locale::NONE), |request| {
            script.send(request)
        })
        .expect("the probe succeeds");

        match outcome {
            PlayerOutcome::Unavailable(state) => assert_eq!(state, VideoState::Unavailable),
            _ => panic!("the video is unavailable"),
        }

        assert_eq!(script.sent.len(), 2);
        assert_eq!(script.client_name(0), "WEB");
        assert_eq!(script.client_name(1), "VISIONOS");
        assert!(script
            .headers(1)
            .contains(&("X-YouTube-Client-Name".to_string(), "101".to_string())));
    }

    #[test]
    fn a_gated_probe_walks_every_fallback_and_keeps_the_last_verdict() {
        // a verdict that is only a gate: a known state, but nothing to report
        let gated = json!({
            "playabilityStatus": {
                "status": "LOGIN_REQUIRED",
                "reason": "The page needs to be reloaded."
            }
        });

        let mut script = Script::new(vec![
            fixture("player_page_unavailable.json"),
            gated.clone(),
            gated.clone(),
            gated.clone(),
            gated,
        ]);

        let outcome = drive(video_probe(&id(), Locale::NONE), |request| {
            script.send(request)
        })
        .expect("the probe succeeds");

        match outcome {
            PlayerOutcome::Unavailable(state) => assert_eq!(state, VideoState::LoginRequired),
            _ => panic!("a gated video is not available"),
        }

        // the WEB client, then all four fallbacks, in order
        assert_eq!(script.sent.len(), 5);
        assert_eq!(script.client_name(1), "VISIONOS");
        assert_eq!(script.client_name(2), "TVHTML5");
        assert_eq!(script.client_name(3), "MWEB");
        assert_eq!(script.client_name(4), "ANDROID_VR");
    }

    #[test]
    fn a_localized_page_is_fetched_twice_unless_it_is_english() {
        let page = json!({"page": 1});

        let mut script = Script::new(vec![page.clone(), page.clone()]);
        let (localized_page, english_page) = drive(
            localized(
                crate::request::search_request("minecraft", None, Locale::NONE, None),
                Locale::NONE,
                || crate::request::search_request("minecraft", None, Locale::ENGLISH, None),
            ),
            |request| script.send(request),
        )
        .unwrap();

        assert_eq!(localized_page, page);
        assert_eq!(english_page, page);
        assert_eq!(script.sent.len(), 2);
        assert_eq!(
            script.sent[1].body.as_ref().unwrap()["context"]["client"]["hl"],
            json!("en")
        );

        let mut script = Script::new(vec![page.clone()]);
        let (localized_page, english_page) = drive(
            localized(
                crate::request::search_request("minecraft", None, Locale::ENGLISH, None),
                Locale::ENGLISH,
                || unreachable!("an English request needs no second page"),
            ),
            |request| script.send(request),
        )
        .unwrap();

        assert_eq!(localized_page, page);
        assert_eq!(english_page, page);
        assert_eq!(script.sent.len(), 1);
    }

    #[test]
    fn comments_are_entered_through_the_watch_page() {
        let mut script = Script::new(vec![
            fixture("player_next_page.json"),
            fixture("continuation_comments_page.json"),
        ]);

        let page = drive(comments_first(&id()), |request| script.send(request)).unwrap();

        assert_eq!(page.comments.len(), 20);
        assert_eq!(script.sent.len(), 2);
        assert!(script.sent[1].body.as_ref().unwrap()["continuation"].is_string());
    }

    #[test]
    fn a_transport_error_stops_the_machine() {
        let mut script = Script::new(vec![]);

        let result = drive(video_probe(&id(), Locale::NONE), |request| {
            script.send(request)
        });

        assert!(result.is_err());
    }
}
