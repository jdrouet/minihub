//! Area — a logical grouping (room, floor, zone) for devices and entities.

use serde::{Deserialize, Serialize};

use crate::error::{MiniHubError, ValidationError};
use crate::id::AreaId;

/// A logical grouping such as a room, floor, or zone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Area {
    pub id: AreaId,
    pub name: String,
    pub parent_id: Option<AreaId>,
}

impl Area {
    /// Create a builder for constructing an [`Area`].
    #[must_use]
    pub fn builder() -> AreaBuilder {
        AreaBuilder::default()
    }

    /// Check domain invariants.
    ///
    /// # Errors
    ///
    /// Returns [`MiniHubError::Validation`] when `name` is empty.
    pub fn validate(&self) -> Result<(), MiniHubError> {
        if self.name.is_empty() {
            return Err(ValidationError::EmptyName.into());
        }
        Ok(())
    }
}

/// Step-by-step builder for [`Area`].
#[derive(Debug, Default)]
pub struct AreaBuilder {
    id: Option<AreaId>,
    name: Option<String>,
    parent_id: Option<AreaId>,
}

impl AreaBuilder {
    #[must_use]
    pub fn id(mut self, id: AreaId) -> Self {
        self.id = Some(id);
        self
    }

    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    #[must_use]
    pub fn parent_id(mut self, parent_id: AreaId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Consume the builder, validate, and return an [`Area`].
    ///
    /// # Errors
    ///
    /// Returns [`MiniHubError::Validation`] if `name` is missing or empty.
    pub fn build(self) -> Result<Area, MiniHubError> {
        let area = Area {
            id: self.id.unwrap_or_default(),
            name: self.name.unwrap_or_default(),
            parent_id: self.parent_id,
        };
        area.validate()?;
        Ok(area)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_build_valid_area_when_name_provided() {
        let area = Area::builder().name("Living Room").build().unwrap();
        assert_eq!(area.name, "Living Room");
        assert!(area.parent_id.is_none());
    }

    #[test]
    fn should_return_validation_error_when_name_is_empty() {
        let result = Area::builder().build();
        assert!(matches!(
            result,
            Err(MiniHubError::Validation(ValidationError::EmptyName))
        ));
    }

    #[test]
    fn should_build_area_with_parent() {
        let parent = AreaId::new();
        let area = Area::builder()
            .name("Bedroom")
            .parent_id(parent)
            .build()
            .unwrap();

        assert_eq!(area.parent_id, Some(parent));
    }

    #[test]
    fn should_roundtrip_through_serde_json() {
        let area = Area::builder().name("Kitchen").build().unwrap();
        let json = serde_json::to_string(&area).unwrap();
        let parsed: Area = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, area.id);
        assert_eq!(parsed.name, area.name);
    }
}
