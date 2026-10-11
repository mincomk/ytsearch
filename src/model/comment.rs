//! Comments on a video.

use crate::parsing::comments_page::CommentComponent;
use crate::parsing::CommentsPage;

/// One comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    /// The comment's text, newlines and all.
    pub text: String,
    /// The commenter's display name, as the page shows it (`@허오키`).
    pub author: String,
    /// The commenter's channel URL.
    pub author_url: String,
}

impl From<CommentComponent> for Comment {
    fn from(comment: CommentComponent) -> Self {
        Comment {
            text: comment.comment,
            author: comment.author_display_name,
            author_url: comment.author_url,
        }
    }
}

/// One batch of comments — the cursor that produced it knows how to ask for the
/// next one.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommentPage {
    pub comments: Vec<Comment>,
}

impl CommentPage {
    /// The page and the token that continues it.
    pub(crate) fn from_parsed(page: CommentsPage) -> (Self, Option<String>) {
        let comments = page.comments.into_iter().map(Comment::from).collect();

        (CommentPage { comments }, page.continuation_token)
    }
}
