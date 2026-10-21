//! The words the recent lead of the finder screen carries.
//!
//! The lead itself is a shared [`termrock_shell::FinderLead`]: the screen owns
//! the layout and the ink, the app owns the words.

pub(crate) const RECENT: &str = "Recent here";
pub(crate) const RECENT_EMPTY: &str = "nothing used here yet";
pub(crate) const EXPLORE: &str = "Tasks   Git   Files   Disk   Services   System";
