use adw::prelude::*;
use gettextrs::gettext;

use super::language_codes::{
    code_at_index,
    index_for_code,
    language_combo_labels,
};
use super::rules::{
    AudioOutcome,
    LanguageCondition,
    PlaybackOutcome,
    PlaybackRule,
    PlaybackRulesConfig,
    RuleCondition,
    SubtitleOutcome,
};
use crate::ui::input::{
    InputAction,
    SettingsNavigator,
};

thread_local! {
    static ACTIVE_EDITOR: std::cell::RefCell<Option<PlaybackRuleEditor>> = const { std::cell::RefCell::new(None) };
}

static SETTINGS_NAVIGATOR: std::sync::LazyLock<std::sync::Mutex<SettingsNavigator>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(SettingsNavigator::default()));

pub fn handle_active_input(action: InputAction) -> bool {
    ACTIVE_EDITOR.with(|slot| {
        let Some(editor) = slot.borrow().clone() else {
            return false;
        };
        if !editor.dialog.is_visible() {
            *slot.borrow_mut() = None;
            return false;
        }
        SETTINGS_NAVIGATOR.lock().unwrap().handle_widgets(
            &editor.focus_widgets(),
            action,
            || {
                editor.dialog.close();
            },
        )
    })
}

#[derive(Clone)]
pub struct PlaybackRuleEditor {
    dialog: adw::Dialog,
    priority_entry: adw::SpinRow,
    when_combo: adw::ComboRow,
    when_lang_combo: adw::ComboRow,
    audio_combo: adw::ComboRow,
    audio_lang_combo: adw::ComboRow,
    subtitle_combo: adw::ComboRow,
    subtitle_lang_combo: adw::ComboRow,
    save_button: gtk::Button,
    cancel_button: gtk::Button,
    editing_default: bool,
}

impl PlaybackRuleEditor {
    pub fn new_rule_dialog(next_priority: u32) -> Self {
        let editor = Self::build_dialog(&gettext("Add Playback Rule"), false);
        editor.priority_entry.set_value(next_priority as f64);
        editor.when_combo.set_selected(1);
        editor.when_lang_combo.set_selected(index_for_code("jpn"));
        editor.subtitle_combo.set_selected(3);
        editor.subtitle_lang_combo.set_selected(index_for_code("eng"));
        editor
    }

    pub fn edit_rule_dialog(rule: &PlaybackRule) -> Self {
        let mut editor = Self::build_dialog(&gettext("Edit Playback Rule"), false);
        editor.load_rule(rule);
        editor
    }

    pub fn default_outcome_dialog(config: &PlaybackRulesConfig) -> Self {
        let mut editor = Self::build_dialog(&gettext("Default Playback Outcome"), true);
        editor.load_outcome(&config.default);
        editor
    }

    fn language_combo_row(title: &str) -> adw::ComboRow {
        let labels = language_combo_labels();
        let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
        let combo = adw::ComboRow::new();
        combo.set_title(title);
        combo.set_model(Some(&gtk::StringList::new(&refs)));
        combo
    }

    fn build_dialog(title: &str, editing_default: bool) -> Self {
        let dialog = adw::Dialog::builder()
            .title(title)
            .content_width(480)
            .content_height(420)
            .build();

        let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);

        let priority_entry = adw::SpinRow::with_range(1.0, 999.0, 1.0);
        priority_entry.set_title(&gettext("Priority"));

        let when_combo = adw::ComboRow::new();
        when_combo.set_title(&gettext("When audio language"));
        when_combo.set_model(Some(&gtk::StringList::new(&[
            &gettext("Any"),
            &gettext("Equals"),
            &gettext("Not equals"),
        ])));

        let when_lang_combo = Self::language_combo_row(&gettext("Language"));

        let audio_combo = adw::ComboRow::new();
        audio_combo.set_title(&gettext("Audio track"));
        audio_combo.set_model(Some(&gtk::StringList::new(&[
            &gettext("No override"),
            &gettext("Prefer language"),
            &gettext("Original"),
        ])));

        let audio_lang_combo = Self::language_combo_row(&gettext("Audio language"));

        let subtitle_combo = adw::ComboRow::new();
        subtitle_combo.set_title(&gettext("Subtitles"));
        subtitle_combo.set_model(Some(&gtk::StringList::new(&[
            &gettext("Off"),
            &gettext("Forced"),
            &gettext("Full"),
            &gettext("Prefer language"),
        ])));

        let subtitle_lang_combo = Self::language_combo_row(&gettext("Subtitle language"));

        let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        button_box.set_halign(gtk::Align::End);
        let save_button = gtk::Button::with_label(&gettext("Save"));
        save_button.add_css_class("suggested-action");
        let cancel_button = gtk::Button::with_label(&gettext("Cancel"));
        button_box.append(&cancel_button);
        button_box.append(&save_button);

        content.append(&priority_entry);
        content.append(&when_combo);
        content.append(&when_lang_combo);
        content.append(&audio_combo);
        content.append(&audio_lang_combo);
        content.append(&subtitle_combo);
        content.append(&subtitle_lang_combo);
        content.append(&button_box);
        dialog.set_child(Some(&content));

        if editing_default {
            priority_entry.set_visible(false);
            when_combo.set_visible(false);
            when_lang_combo.set_visible(false);
        }

        cancel_button.connect_clicked({
            let dialog = dialog.clone();
            move |_| {
                ACTIVE_EDITOR.with(|slot| *slot.borrow_mut() = None);
                dialog.close();
            }
        });

        dialog.connect_closed(|_| {
            ACTIVE_EDITOR.with(|slot| *slot.borrow_mut() = None);
        });

        Self {
            dialog,
            priority_entry,
            when_combo,
            when_lang_combo,
            audio_combo,
            audio_lang_combo,
            subtitle_combo,
            subtitle_lang_combo,
            save_button,
            cancel_button,
            editing_default,
        }
    }

    pub fn present(&self, parent: Option<&impl IsA<gtk::Widget>>) {
        ACTIVE_EDITOR.with(|slot| {
            *slot.borrow_mut() = Some(self.clone());
        });
        self.dialog.present(parent);
    }

    pub fn focus_widgets(&self) -> Vec<gtk::Widget> {
        let mut widgets = Vec::new();
        if self.priority_entry.is_visible() {
            widgets.push(self.priority_entry.clone().upcast());
        }
        if self.when_combo.is_visible() {
            widgets.push(self.when_combo.clone().upcast());
        }
        if self.when_lang_combo.is_visible() {
            widgets.push(self.when_lang_combo.clone().upcast());
        }
        widgets.push(self.audio_combo.clone().upcast());
        widgets.push(self.audio_lang_combo.clone().upcast());
        widgets.push(self.subtitle_combo.clone().upcast());
        widgets.push(self.subtitle_lang_combo.clone().upcast());
        widgets.push(self.save_button.clone().upcast());
        widgets.push(self.cancel_button.clone().upcast());
        widgets
    }

    pub fn connect_save<F>(&self, callback: F)
    where
        F: Fn(PlaybackRule) + 'static,
    {
        let dialog = self.dialog.clone();
        let editor = PlaybackRuleEditorRef {
            priority_entry: self.priority_entry.clone(),
            when_combo: self.when_combo.clone(),
            when_lang_combo: self.when_lang_combo.clone(),
            audio_combo: self.audio_combo.clone(),
            audio_lang_combo: self.audio_lang_combo.clone(),
            subtitle_combo: self.subtitle_combo.clone(),
            subtitle_lang_combo: self.subtitle_lang_combo.clone(),
            editing_default: self.editing_default,
        };
        self.save_button.connect_clicked(move |_| {
            let rule = if editor.editing_default {
                PlaybackRule {
                    priority: 0,
                    when: RuleCondition {
                        audio_language: LanguageCondition::Any,
                    },
                    then: editor.build_outcome(),
                }
            } else {
                editor.build_rule()
            };
            callback(rule);
            ACTIVE_EDITOR.with(|slot| *slot.borrow_mut() = None);
            dialog.close();
        });
    }

    fn load_rule(&mut self, rule: &PlaybackRule) {
        self.priority_entry.set_value(rule.priority as f64);
        match &rule.when.audio_language {
            LanguageCondition::Any => self.when_combo.set_selected(0),
            LanguageCondition::Equals(lang) => {
                self.when_combo.set_selected(1);
                self.when_lang_combo.set_selected(index_for_code(lang));
            }
            LanguageCondition::NotEquals(lang) => {
                self.when_combo.set_selected(2);
                self.when_lang_combo.set_selected(index_for_code(lang));
            }
        }
        self.load_outcome(&rule.then);
    }

    fn load_outcome(&mut self, outcome: &PlaybackOutcome) {
        match &outcome.audio {
            AudioOutcome::NoOverride => self.audio_combo.set_selected(0),
            AudioOutcome::PreferLanguage { language } => {
                self.audio_combo.set_selected(1);
                self.audio_lang_combo.set_selected(index_for_code(language));
            }
            AudioOutcome::Original => self.audio_combo.set_selected(2),
        }
        match &outcome.subtitles {
            SubtitleOutcome::Off => self.subtitle_combo.set_selected(0),
            SubtitleOutcome::Forced { language } => {
                self.subtitle_combo.set_selected(1);
                self.subtitle_lang_combo.set_selected(index_for_code(language));
            }
            SubtitleOutcome::Full { language } => {
                self.subtitle_combo.set_selected(2);
                self.subtitle_lang_combo.set_selected(index_for_code(language));
            }
            SubtitleOutcome::PreferLanguage { language } => {
                self.subtitle_combo.set_selected(3);
                self.subtitle_lang_combo.set_selected(index_for_code(language));
            }
        }
    }
}

#[derive(Clone)]
struct PlaybackRuleEditorRef {
    priority_entry: adw::SpinRow,
    when_combo: adw::ComboRow,
    when_lang_combo: adw::ComboRow,
    audio_combo: adw::ComboRow,
    audio_lang_combo: adw::ComboRow,
    subtitle_combo: adw::ComboRow,
    subtitle_lang_combo: adw::ComboRow,
    editing_default: bool,
}

impl PlaybackRuleEditorRef {
    fn build_rule(&self) -> PlaybackRule {
        PlaybackRule {
            priority: self.priority_entry.value() as u32,
            when: self.build_condition(),
            then: self.build_outcome(),
        }
    }

    fn build_condition(&self) -> RuleCondition {
        let lang = code_at_index(self.when_lang_combo.selected());
        let audio_language = match self.when_combo.selected() {
            1 => LanguageCondition::Equals(lang),
            2 => LanguageCondition::NotEquals(lang),
            _ => LanguageCondition::Any,
        };
        RuleCondition { audio_language }
    }

    fn build_outcome(&self) -> PlaybackOutcome {
        let audio_lang = code_at_index(self.audio_lang_combo.selected());
        let audio = match self.audio_combo.selected() {
            1 => AudioOutcome::PreferLanguage { language: audio_lang },
            2 => AudioOutcome::Original,
            _ => AudioOutcome::NoOverride,
        };

        let sub_lang = code_at_index(self.subtitle_lang_combo.selected());
        let subtitles = match self.subtitle_combo.selected() {
            1 => SubtitleOutcome::Forced {
                language: sub_lang,
            },
            2 => SubtitleOutcome::Full {
                language: sub_lang,
            },
            3 => SubtitleOutcome::PreferLanguage {
                language: sub_lang,
            },
            _ => SubtitleOutcome::Off,
        };

        PlaybackOutcome { audio, subtitles }
    }
}
