//! Entity state — the current operational state of an entity.

use serde::{Deserialize, Serialize};

/// Discrete operational state of an entity.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntityState {
    On,
    Off,
    #[default]
    Unknown,
    Unavailable,
}

impl EntityState {
    /// Whether the entity is reachable (anything but [`Unavailable`](Self::Unavailable)).
    #[must_use]
    pub fn is_available(&self) -> bool {
        !matches!(self, Self::Unavailable)
    }
}

impl std::fmt::Display for EntityState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::On => f.write_str("on"),
            Self::Off => f.write_str("off"),
            Self::Unknown => f.write_str("unknown"),
            Self::Unavailable => f.write_str("unavailable"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_report_available_when_state_is_on() {
        assert!(EntityState::On.is_available());
    }

    #[test]
    fn should_report_available_when_state_is_off() {
        assert!(EntityState::Off.is_available());
    }

    #[test]
    fn should_report_available_when_state_is_unknown() {
        assert!(EntityState::Unknown.is_available());
    }

    #[test]
    fn should_report_unavailable_when_state_is_unavailable() {
        assert!(!EntityState::Unavailable.is_available());
    }

    #[test]
    fn should_default_to_unknown() {
        assert_eq!(EntityState::default(), EntityState::Unknown);
    }

    #[test]
    fn should_display_lowercase_variant_name() {
        assert_eq!(EntityState::On.to_string(), "on");
        assert_eq!(EntityState::Off.to_string(), "off");
    }

    #[test]
    fn should_roundtrip_through_serde_json() {
        let state = EntityState::On;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"on\"");
        let parsed: EntityState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, state);
    }
}
