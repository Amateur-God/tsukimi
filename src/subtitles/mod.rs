pub mod opensubtitles;
pub mod provider;
pub mod subdl;
pub mod dialog;

pub use dialog::{
    SubtitleSearchDialog,
    preferred_subtitle_language_code,
};
