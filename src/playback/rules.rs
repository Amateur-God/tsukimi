use serde::{
    Deserialize,
    Serialize,
};

use crate::ui::SETTINGS;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaybackRulesConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub rules: Vec<PlaybackRule>,
    #[serde(default)]
    pub default: PlaybackOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackRule {
    pub priority: u32,
    pub when: RuleCondition,
    pub then: PlaybackOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    #[serde(default)]
    pub audio_language: LanguageCondition,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "op", content = "value")]
pub enum LanguageCondition {
    #[default]
    #[serde(rename = "any")]
    Any,
    #[serde(rename = "equals")]
    Equals(String),
    #[serde(rename = "not_equals")]
    NotEquals(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlaybackOutcome {
    #[serde(default)]
    pub audio: AudioOutcome,
    #[serde(default)]
    pub subtitles: SubtitleOutcome,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "mode")]
pub enum AudioOutcome {
    #[default]
    #[serde(rename = "no_override")]
    NoOverride,
    #[serde(rename = "prefer_language")]
    PreferLanguage {
        language: String,
    },
    #[serde(rename = "original")]
    Original,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "mode")]
pub enum SubtitleOutcome {
    #[default]
    #[serde(rename = "off")]
    Off,
    #[serde(rename = "forced")]
    Forced {
        #[serde(default)]
        language: String,
    },
    #[serde(rename = "full")]
    Full {
        #[serde(default)]
        language: String,
    },
    #[serde(rename = "prefer_language")]
    PreferLanguage {
        language: String,
    },
}

pub struct PlaybackRules;

impl PlaybackRules {
    pub fn load() -> PlaybackRulesConfig {
        SETTINGS.playback_conditional_rules()
    }

    pub fn evaluate(audio_language: Option<&str>, config: &PlaybackRulesConfig) -> PlaybackOutcome {
        if !config.enabled {
            return PlaybackOutcome::default();
        }

        let lang = audio_language.unwrap_or("");

        let mut rules = config.rules.clone();
        rules.sort_by_key(|r| r.priority);

        for rule in rules {
            if condition_matches(&rule.when, lang) {
                return rule.then.clone();
            }
        }

        config.default.clone()
    }
}

fn condition_matches(condition: &RuleCondition, audio_language: &str) -> bool {
    match &condition.audio_language {
        LanguageCondition::Any => true,
        LanguageCondition::Equals(expected) => {
            language_matches(audio_language, expected)
        }
        LanguageCondition::NotEquals(expected) => {
            !language_matches(audio_language, expected)
        }
    }
}

fn language_matches(actual: &str, expected: &str) -> bool {
    let actual = actual.to_lowercase();
    let expected = expected.to_lowercase();
    actual == expected || actual.starts_with(&format!("{expected}-"))
}
