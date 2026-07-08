use std::cell::RefCell;

use gtk::prelude::*;

use super::actions::InputAction;
use crate::{
    tv::set_tv_focused,
    ui::widgets::server_action_row::ServerActionRow,
    Window,
};

pub struct PlaceholderNavigator {
    selected_index: RefCell<u32>,
    last_row: RefCell<Option<gtk::ListBoxRow>>,
}

impl Default for PlaceholderNavigator {
    fn default() -> Self {
        Self {
            selected_index: RefCell::new(0),
            last_row: RefCell::new(None),
        }
    }
}

impl PlaceholderNavigator {
    pub fn reset(&self) {
        *self.selected_index.borrow_mut() = 0;
        self.clear_row_focus();
    }

    pub fn select_initial(&self, listbox: &gtk::ListBox) {
        if listbox.observe_children().n_items() > 0 {
            select_row(self, listbox, 0);
        }
    }

    pub fn handle(
        &self,
        window: &Window,
        login_stack: &gtk::Stack,
        listbox: &gtk::ListBox,
        action: InputAction,
    ) -> bool {
        if login_stack.visible_child_name().as_deref() == Some("no-server") {
            return match action {
                InputAction::Activate => {
                    window.new_account();
                    true
                }
                _ => false,
            };
        }

        let count = listbox.observe_children().n_items();
        if count == 0 {
            return match action {
                InputAction::Activate => {
                    window.new_account();
                    true
                }
                _ => false,
            };
        }

        let mut index = *self.selected_index.borrow();
        if index >= count {
            index = count.saturating_sub(1);
        }

        match action {
            InputAction::NavigateDown | InputAction::NavigateRight => {
                index = (index + 1).min(count - 1);
                *self.selected_index.borrow_mut() = index;
                select_row(self, listbox, index);
                true
            }
            InputAction::NavigateUp | InputAction::NavigateLeft => {
                index = index.saturating_sub(1);
                *self.selected_index.borrow_mut() = index;
                select_row(self, listbox, index);
                true
            }
            InputAction::Activate => {
                if let Some(row) = listbox.row_at_index(index as i32) {
                    row.activate();
                }
                true
            }
            _ => false,
        }
    }
}

fn select_row(navigator: &PlaceholderNavigator, listbox: &gtk::ListBox, index: u32) {
    navigator.clear_row_focus();
    if let Some(row) = listbox.row_at_index(index as i32) {
        listbox.select_row(Some(&row));
        row.grab_focus();
        if let Ok(server_row) = row.clone().downcast::<ServerActionRow>() {
            server_row.set_tv_focused(true);
        } else {
            set_tv_focused(&row, true);
        }
        *navigator.last_row.borrow_mut() = Some(row);
    }
}

impl PlaceholderNavigator {
    fn clear_row_focus(&self) {
        if let Some(row) = self.last_row.borrow_mut().take() {
            if let Ok(server_row) = row.clone().downcast::<ServerActionRow>() {
                server_row.set_tv_focused(false);
            } else {
                set_tv_focused(&row, false);
            }
        }
    }
}
