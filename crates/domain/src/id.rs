//! Typed identifier newtypes backed by UUIDs.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

macro_rules! define_id {
    ($(#[doc = $doc:expr])* $name:ident) => {
        $(#[doc = $doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(uuid::Uuid);

        impl Default for $name {
            fn default() -> Self {
                Self(uuid::Uuid::new_v4())
            }
        }

        impl $name {
            /// Generate a new random identifier.
            #[must_use]
            pub fn new() -> Self {
                Self::default()
            }

            /// Wrap an existing UUID.
            #[must_use]
            pub fn from_uuid(uuid: uuid::Uuid) -> Self {
                Self(uuid)
            }

            /// Access the inner UUID.
            #[must_use]
            pub fn as_uuid(self) -> uuid::Uuid {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                uuid::Uuid::parse_str(s).map(Self)
            }
        }
    };
}

define_id!(
    /// Unique identifier for an [`Entity`](crate::entity::Entity).
    EntityId
);

define_id!(
    /// Unique identifier for a [`Device`](crate::device::Device).
    DeviceId
);

define_id!(
    /// Unique identifier for an [`Area`](crate::area::Area).
    AreaId
);

define_id!(
    /// Unique identifier for an [`Automation`](crate::automation).
    AutomationId
);

define_id!(
    /// Unique identifier for an [`Event`](crate::event).
    EventId
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_unique_ids_when_called_twice() {
        let a = EntityId::new();
        let b = EntityId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn should_roundtrip_through_display_and_from_str() {
        let id = DeviceId::new();
        let text = id.to_string();
        let parsed: DeviceId = text.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn should_roundtrip_through_serde_json() {
        let id = AreaId::new();
        let json = serde_json::to_string(&id).unwrap();
        let parsed: AreaId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn should_return_error_when_parsing_invalid_uuid() {
        let result = EntityId::from_str("not-a-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn should_wrap_existing_uuid_when_using_from_uuid() {
        let uuid = uuid::Uuid::new_v4();
        let id = AutomationId::from_uuid(uuid);
        assert_eq!(id.as_uuid(), uuid);
    }
}
