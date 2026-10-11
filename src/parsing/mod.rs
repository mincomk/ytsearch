//! Page parsing: raw json into plain structs.
//!
//! Nothing here does I/O and nothing here knows about a client, a locale or a
//! cursor. These types are internal — the public surface is [`crate::model`],
//! which is built from them. The paths and the rules are the ones yspy uses;
//! see NOTICE.

pub(crate) mod channel;
pub(crate) mod counts;
pub(crate) mod player;
pub(crate) mod playlist;
pub(crate) mod search_result;

pub(crate) mod comments_page;
pub(crate) mod suggestion_data;

pub(crate) use comments_page::CommentsPage;

#[cfg(test)]
mod tests;
