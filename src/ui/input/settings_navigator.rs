use adw::prelude::*;
use gtk::prelude::*;

use super::actions::InputAction;
use crate::{
    tv::set_tv_focused,
    ui::widgets::account_add::AccountWindow,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum SettingsZone {
    Tabs,
    Rows,
}

pub struct SettingsNavigator {
    row_index: std::cell::Cell<u32>,
    tab_index: std::cell::Cell<u32>,
    zone: std::cell::Cell<SettingsZone>,
}

impl Default for SettingsNavigator {
    fn default() -> Self {
        Self {
            row_index: std::cell::Cell::new(0),
            tab_index: std::cell::Cell::new(0),
            zone: std::cell::Cell::new(SettingsZone::Rows),
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
        if let Some(prefs) = window.downcast_ref::<adw::PreferencesWindow>() {
            return self.handle_preferences(prefs, action);
        }
        let rows = collect_focusable_rows_from_root(&window.clone().upcast::<gtk::Widget>());
        self.handle_rows_only(&rows, action, || window.close())
    }

    fn handle_preferences(&self, prefs: &adw::PreferencesWindow, action: InputAction) -> bool {
        let pages = collect_preference_pages(prefs);
        if pages.is_empty() {
            return false;
        }

        let tab_count = pages.len() as u32;
        let mut tab_index = self.tab_index.get().min(tab_count - 1);
        if let Some(page) = pages.get(tab_index as usize) {
            prefs.set_visible_page(page);
        }

        let rows = collect_focusable_rows_from_root(
            &prefs
                .visible_page()
                .unwrap_or_else(|| pages[tab_index as usize].clone())
                .upcast::<gtk::Widget>(),
        );

        match self.zone.get() {
            SettingsZone::Tabs => match action {
                InputAction::NavigateLeft => {
                    tab_index = tab_index.saturating_sub(1);
                    self.switch_tab(prefs, &pages, tab_index);
                    true
                }
                InputAction::NavigateRight => {
                    tab_index = (tab_index + 1).min(tab_count - 1);
                    self.switch_tab(prefs, &pages, tab_index);
                    true
                }
                InputAction::NavigateDown => {
                    if rows.is_empty() {
                        return true;
                    }
                    self.zone.set(SettingsZone::Rows);
                    self.row_index.set(0);
                    self.apply_focus(&rows, 0);
                    true
                }
                InputAction::NavigateUp => true,
                InputAction::Activate => true,
                InputAction::Back => {
                    prefs.close();
                    true
                }
                _ => false,
            },
            SettingsZone::Rows => {
                if rows.is_empty() {
                    return match action {
                        InputAction::NavigateUp => {
                            self.zone.set(SettingsZone::Tabs);
                            true
                        }
                        InputAction::Back => {
                            prefs.close();
                            true
                        }
                        _ => false,
                    };
                }

                let count = rows.len() as u32;
                let mut index = self.row_index.get().min(count - 1);

                match action {
                    InputAction::NavigateDown => {
                        index = (index + 1).min(count - 1);
                        self.row_index.set(index);
                        self.apply_focus(&rows, index);
                        true
                    }
                    InputAction::NavigateUp => {
                        if index == 0 {
                            self.zone.set(SettingsZone::Tabs);
                            self.clear_row_focus(&rows);
                            true
                        } else {
                            index = index.saturating_sub(1);
                            self.row_index.set(index);
                            self.apply_focus(&rows, index);
                            true
                        }
                    }
                    InputAction::NavigateLeft | InputAction::NavigateRight => {
                        if let Some(row) = rows.get(index as usize) {
                            self.activate_row_widget(row);
                        }
                        true
                    }
                    InputAction::Activate => {
                        if let Some(row) = rows.get(index as usize) {
                            self.activate_row_widget(row);
                        }
                        true
                    }
                    InputAction::Back => {
                        prefs.close();
                        true
                    }
                    _ => false,
                }
            }
        }
    }

    fn switch_tab(
        &self,
        prefs: &adw::PreferencesWindow,
        pages: &[adw::PreferencesPage],
        tab_index: u32,
    ) {
        self.tab_index.set(tab_index);
        self.row_index.set(0);
        if let Some(page) = pages.get(tab_index as usize) {
            prefs.set_visible_page(page);
        }
        if self.zone.get() == SettingsZone::Rows {
            let rows = collect_focusable_rows_from_root(
                &prefs
                    .visible_page()
                    .unwrap_or_else(|| pages[tab_index as usize].clone())
                    .upcast::<gtk::Widget>(),
            );
            if rows.is_empty() {
                self.zone.set(SettingsZone::Tabs);
            } else {
                self.apply_focus(&rows, 0);
            }
        }
    }

    fn handle_rows_only(
        &self,
        rows: &[gtk::Widget],
        action: InputAction,
        on_back: impl FnOnce(),
    ) -> bool {
        if rows.is_empty() {
            return false;
        }
        let count = rows.len() as u32;
        let mut index = self.row_index.get().min(count - 1);

        match action {
            InputAction::NavigateDown => {
                index = (index + 1).min(count - 1);
                self.row_index.set(index);
                self.apply_focus(rows, index);
                true
            }
            InputAction::NavigateUp => {
                index = index.saturating_sub(1);
                self.row_index.set(index);
                self.apply_focus(rows, index);
                true
            }
            InputAction::NavigateLeft | InputAction::NavigateRight => {
                if let Some(row) = rows.get(index as usize) {
                    self.activate_row_widget(row);
                }
                true
            }
            InputAction::Activate => {
                if let Some(row) = rows.get(index as usize) {
                    self.activate_row_widget(row);
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
        self.handle_rows_only(widgets, action, on_back)
    }

    fn activate_row_widget(&self, row: &gtk::Widget) {
        if let Ok(switch) = row.clone().downcast::<adw::SwitchRow>() {
            switch.set_active(!switch.is_active());
        } else if let Ok(switch) = row.clone().downcast::<gtk::Switch>() {
            switch.set_active(!switch.is_active());
        } else if let Ok(spin) = row.clone().downcast::<adw::SpinRow>() {
            spin.grab_focus();
        } else if let Ok(row) = row.clone().downcast::<adw::ComboRow>() {
            adw::prelude::ActionRowExt::activate(&row);
        } else if let Ok(row) = row.clone().downcast::<adw::ActionRow>() {
            adw::prelude::ActionRowExt::activate(&row);
        } else if let Ok(button) = row.clone().downcast::<gtk::Button>() {
            button.emit_clicked();
        } else {
            row.activate();
        }
    }

    fn clear_row_focus(&self, rows: &[gtk::Widget]) {
        for row in rows {
            set_tv_focused(row, false);
        }
    }

    fn apply_focus(&self, rows: &[gtk::Widget], index: u32) {
        self.clear_row_focus(rows);
        if let Some(row) = rows.get(index as usize) {
            set_tv_focused(row, true);
            row.grab_focus();
        }
    }
}

fn collect_preference_pages(prefs: &adw::PreferencesWindow) -> Vec<adw::PreferencesPage> {
    let mut pages = Vec::new();
    let mut stack = vec![prefs.clone().upcast::<gtk::Widget>()];
    while let Some(widget) = stack.pop() {
        if let Ok(page) = widget.clone().downcast::<adw::PreferencesPage>() {
            pages.push(page);
            continue;
        }
        let mut child = widget.first_child();
        while let Some(next) = child {
            stack.push(next.clone());
            child = next.next_sibling();
        }
    }
    pages
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
