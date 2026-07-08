use gtk::prelude::*;

use crate::{
    ui::widgets::liked::LikedPage,
    ui::input::{
        actions::InputAction,
        focus_manager::FocusManager,
    },
    Window,
};

pub struct LikedNavigator {
    focus: FocusManager,
}

impl Default for LikedNavigator {
    fn default() -> Self {
        Self {
            focus: FocusManager::default(),
        }
    }
}

impl LikedNavigator {
    pub fn register(&self, liked: &LikedPage) {
        self.focus.register_rows(liked.focus_hortu_rows());
    }

    pub fn handle(&self, window: &Window, liked: &LikedPage, action: InputAction) -> bool {
        if self.focus.handle_rows_only(window, action) {
            return true;
        }
        // Re-register if rows were empty on first visit.
        if liked.focus_hortu_rows().iter().any(|r| r.is_visible()) {
            self.register(liked);
            return self.focus.handle_rows_only(window, action);
        }
        false
    }
}
