//! Shared audio configuration for timers, effects, and alerts

use serde::{Deserialize, Serialize};

/// Audio configuration shared by timers, effects, and alerts
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Master toggle for audio on this item
    #[serde(default, skip_serializing_if = "crate::serde_defaults::is_false")]
    pub enabled: bool,

    /// Audio file to play. Folder-relative under `core/definitions/`
    /// (e.g. `"sounds/Alert.mp3"`, `"mechanic-sounds/Acid Deluge.mp3"`).
    /// A bare filename (no `/`) is resolved against the General `sounds/`
    /// folder for backward compatibility; an absolute path is used verbatim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,

    /// Seconds before expiration to play audio (0 = on expiration)
    #[serde(default, skip_serializing_if = "crate::serde_defaults::is_zero_u8")]
    pub offset: u8,

    /// Start countdown audio at N seconds remaining (0 = disabled)
    #[serde(default, skip_serializing_if = "crate::serde_defaults::is_zero_u8")]
    pub countdown_start: u8,

    /// Voice pack for countdown (None = default)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub countdown_voice: Option<String>,

    /// Alert text to display on alert overlay when effect triggers.
    /// If non-empty, sends this text to the alert overlay.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alert_text: Option<String>,

    /// Defer the audio cue until this many seconds remain on the GCD.
    /// Only consulted when this `AudioConfig` is used as a timer's
    /// `queue_next_audio` (the rising-edge "becomes next cast" cue) — other
    /// audio paths ignore it. `None` (or unset) plays immediately when the
    /// timer becomes the unique highest-priority next cast.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at_gcd_remaining: Option<f32>,
}

impl AudioConfig {
    /// Returns true if all fields are at their default values
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    /// Check if any audio is configured
    pub fn has_audio(&self) -> bool {
        self.enabled && (self.file.is_some() || self.countdown_start > 0)
    }

    /// Check if countdown audio is enabled
    pub fn has_countdown(&self) -> bool {
        self.enabled && self.countdown_start > 0
    }

    /// Check if alert text is configured
    pub fn has_alert_text(&self) -> bool {
        self.alert_text.as_ref().is_some_and(|t| !t.is_empty())
    }
}
