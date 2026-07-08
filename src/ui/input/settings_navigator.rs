use adw::prelude::*;

use super::actions::InputAction;
use crate::{
    tv::set_tv_focused,
    ui::widgets::account_add::AccountWindow,
};

pub struct SettingsNavigator {
    row_index: std::cell::Cell<u32>,
}

impl Default for SettingsNavigator {
    fn default() -> Self {
        Self {
            row_index: std::cell::Cell::new(0),
        }
    }
}

impl SettingsNavigator {
    pub fn handle_window(
        &self,
        window: &crate::ui::widgets::account_settings::AccountSettings,
        action: InputAction,
    ) -> bool {
        self.handle_window_inner(window.upcast_ref(), action)
    }

    fn handle_window_inner(&self, window: &adw::Window, action: InputAction) -> bool {
        let rows = collect_focusable_rows(window);
        if rows.is_empty() {
            return false;
        }
        let count = rows.len() as u32;
        let mut index = self.row_index.get().min(count - 1);

        match action {
            InputAction::NavigateDown | InputAction::NavigateRight => {
                index = (index + 1).min(count - 1);
                self.row_index.set(index);
                self.apply_focus(&rows, index);
                true
            }
            InputAction::NavigateUp | InputAction::NavigateLeft => {
                index = index.saturating_sub(1);
                self.row_index.set(index);
                self.apply_focus(&rows, index);
                true
            }
            InputAction::Activate => {
                if let Some(row) = rows.get(index as usize) {
                    if let Ok(switch) = row.clone().downcast::<adw::SwitchRow>() {
                        switch.set_active(!switch.is_active());
                    } else {
                        row.activate();
                    }
                }
                true
            }
            InputAction::Back => {
                window.close();
                true
            }
            _ => false,
        }
    }

    pub fn handle_account_window(&self, account: &AccountWindow, action: InputAction) -> bool {
        let widgets = account.focus_widgets();
        let count = widgets.len() as u32;
        if matches!(action, InputAction::Activate) {
            let index = self.row_index.get().min(count.saturating_sub(1));
            if index == count.saturating_sub(1) {
                let account = account.clone();
                crate::utils::spawn(async move {
                    account.add().await;
                });
                return true;
            }
        }
        self.handle_widgets(&widgets, action, || {
            adw::prelude::AdwDialogExt::close(account);
        })
    }

    pub fn handle_widgets(
        &self,
        widgets: &[gtk::Widget],
        action: InputAction,
        on_back: impl FnOnce(),
    ) -> bool {
        if widgets.is_empty() {
            return false;
        }
        let count = widgets.len() as u32;
        let mut index = self.row_index.get().min(count - 1);

        match action {
            InputAction::NavigateDown | InputAction::NavigateRight => {
                index = (index + 1).min(count - 1);
                self.row_index.set(index);
                self.apply_widget_focus(widgets, index);
                true
            }
            InputAction::NavigateUp | InputAction::NavigateLeft => {
                index = index.saturating_sub(1);
                self.row_index.set(index);
                self.apply_widget_focus(widgets, index);
                true
            }
            InputAction::Activate => {
                if let Some(widget) = widgets.get(index as usize) {
                    if let Ok(switch) = widget.clone().downcast::<adw::SwitchRow>() {
                        switch.set_active(!switch.is_active());
                    } else if let Ok(switch) = widget.clone().downcast::<gtk::Switch>() {
                        switch.set_active(!switch.is_active());
                    } else if let Ok(spin) = widget.clone().downcast::<adw::SpinRow>() {
                        spin.grab_focus();
                    } else if let Ok(row) = widget.clone().downcast::<adw::ComboRow>() {
                        adw::prelude::ActionRowExt::activate(&row);
                    } else if let Ok(row) = widget.clone().downcast::<adw::ActionRow>() {
                        adw::prelude::ActionRowExt::activate(&row);
                    } else if let Ok(button) = widget.clone().downcast::<gtk::Button>() {
                        button.emit_clicked();
                    } else {
                        widget.grab_focus();
                    }
                }
                true
            }
            InputAction::Back => {
                on_back();
                true
            }
            _ => false,
        }
    }

    fn apply_focus(&self, rows: &[gtk::Widget], index: u32) {
        for row in rows {
            set_tv_focused(row, false);
        }
        if let Some(row) = rows.get(index as usize) {
            set_tv_focused(row, true);
            row.grab_focus();
        }
    }

    fn apply_widget_focus(&self, widgets: &[gtk::Widget], index: u32) {
        for widget in widgets {
            set_tv_focused(widget, false);
        }
        if let Some(widget) = widgets.get(index as usize) {
            set_tv_focused(widget, true);
            widget.grab_focus();
        }
    }
}

fn collect_focusable_rows(window: &adw::Window) -> Vec<gtk::Widget> {
    if let Some(prefs) = window.downcast_ref::<adw::PreferencesWindow>() {
        if let Some(page) = prefs.visible_page() {
            return collect_focusable_rows_from_root(&page.upcast::<gtk::Widget>());
        }
    }
    collect_focusable_rows_from_root(&window.clone().upcast::<gtk::Widget>())
}

fn collect_focusable_rows_from_root(root: &gtk::Widget) -> Vec<gtk::Widget> {
    let mut rows = Vec::new();
    let mut stack = vec![root.clone()];
    while let Some(widget) = stack.pop() {
        if widget.downcast_ref::<adw::PreferencesRow>().is_some()
            || widget.downcast_ref::<adw::ActionRow>().is_some()
            || widget.downcast_ref::<adw::ButtonRow>().is_some()
            || widget.downcast_ref::<adw::ComboRow>().is_some()
            || widget.downcast_ref::<adw::EntryRow>().is_some()
            || widget.downcast_ref::<adw::PasswordEntryRow>().is_some()
            || widget.downcast_ref::<adw::SpinRow>().is_some()
            || widget.downcast_ref::<adw::SwitchRow>().is_some()
        {
            if widget.is_visible() && widget.is_sensitive() {
                rows.push(widget);
            }
            continue;
        }
        let mut child = widget.first_child();
        while let Some(c) = child {
            stack.push(c.clone());
            child = c.next_sibling();
        }
    }
    rows
}
