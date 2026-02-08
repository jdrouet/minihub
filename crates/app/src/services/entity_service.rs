//! Entity service — use-cases for managing entities.

use minihub_domain::entity::{Entity, EntityState};
use minihub_domain::error::{MiniHubError, NotFoundError};
use minihub_domain::id::EntityId;
use minihub_domain::time::now;

use crate::ports::EntityRepository;

/// Application service for entity CRUD and state management.
pub struct EntityService<R> {
    repo: R,
}

impl<R: EntityRepository> EntityService<R> {
    /// Create a new service backed by the given repository.
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Create a new entity after validating domain invariants.
    ///
    /// # Errors
    ///
    /// Returns [`MiniHubError::Validation`] if invariants fail, or a
    /// storage error propagated from the repository.
    pub async fn create_entity(&self, mut entity: Entity) -> Result<Entity, MiniHubError> {
        entity.validate()?;
        let ts = now();
        entity.last_updated = ts;
        entity.last_changed = ts;
        self.repo.create(entity).await
    }

    /// Look up an entity by id, returning an error if not found.
    ///
    /// # Errors
    ///
    /// Returns [`MiniHubError::NotFound`] when no entity with `id` exists,
    /// or a storage error from the repository.
    pub async fn get_entity(&self, id: EntityId) -> Result<Entity, MiniHubError> {
        self.repo.get_by_id(id).await?.ok_or_else(|| {
            NotFoundError {
                entity: "Entity",
                id: id.to_string(),
            }
            .into()
        })
    }

    /// List all entities.
    ///
    /// # Errors
    ///
    /// Returns a storage error propagated from the repository.
    pub async fn list_entities(&self) -> Result<Vec<Entity>, MiniHubError> {
        self.repo.get_all().await
    }

    /// Update the state of an existing entity.
    ///
    /// # Errors
    ///
    /// Returns [`MiniHubError::NotFound`] if the entity does not exist,
    /// or a storage error from the repository.
    pub async fn update_entity_state(
        &self,
        id: EntityId,
        new_state: EntityState,
    ) -> Result<Entity, MiniHubError> {
        let mut entity = self.get_entity(id).await?;
        entity.update_state(new_state, now());
        self.repo.update(entity).await
    }

    /// Delete an entity by id.
    ///
    /// # Errors
    ///
    /// Returns a storage error propagated from the repository.
    pub async fn delete_entity(&self, id: EntityId) -> Result<(), MiniHubError> {
        self.repo.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use minihub_domain::entity::EntityState;
    use minihub_domain::error::ValidationError;
    use minihub_domain::id::DeviceId;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct InMemoryEntityRepo {
        store: Mutex<HashMap<EntityId, Entity>>,
    }

    impl Default for InMemoryEntityRepo {
        fn default() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }
    }

    impl EntityRepository for InMemoryEntityRepo {
        fn create(
            &self,
            entity: Entity,
        ) -> impl Future<Output = Result<Entity, MiniHubError>> + Send {
            let mut store = self.store.lock().unwrap();
            store.insert(entity.id, entity.clone());
            async { Ok(entity) }
        }

        fn get_by_id(
            &self,
            id: EntityId,
        ) -> impl Future<Output = Result<Option<Entity>, MiniHubError>> + Send {
            let store = self.store.lock().unwrap();
            let result = store.get(&id).cloned();
            async { Ok(result) }
        }

        fn get_all(&self) -> impl Future<Output = Result<Vec<Entity>, MiniHubError>> + Send {
            let store = self.store.lock().unwrap();
            let result: Vec<Entity> = store.values().cloned().collect();
            async { Ok(result) }
        }

        fn find_by_device_id(
            &self,
            device_id: DeviceId,
        ) -> impl Future<Output = Result<Vec<Entity>, MiniHubError>> + Send {
            let store = self.store.lock().unwrap();
            let result: Vec<Entity> = store
                .values()
                .filter(|ent| ent.device_id == device_id)
                .cloned()
                .collect();
            async { Ok(result) }
        }

        fn update(
            &self,
            entity: Entity,
        ) -> impl Future<Output = Result<Entity, MiniHubError>> + Send {
            let mut store = self.store.lock().unwrap();
            store.insert(entity.id, entity.clone());
            async { Ok(entity) }
        }

        fn delete(&self, id: EntityId) -> impl Future<Output = Result<(), MiniHubError>> + Send {
            let mut store = self.store.lock().unwrap();
            store.remove(&id);
            async { Ok(()) }
        }
    }

    use std::future::Future;

    fn make_service() -> EntityService<InMemoryEntityRepo> {
        EntityService::new(InMemoryEntityRepo::default())
    }

    fn valid_entity() -> Entity {
        Entity::builder()
            .entity_id("light.living_room")
            .friendly_name("Living Room Light")
            .state(EntityState::Off)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn should_create_entity_when_valid() {
        let svc = make_service();
        let entity = valid_entity();
        let id = entity.id;

        let created = svc.create_entity(entity).await.unwrap();
        assert_eq!(created.id, id);

        let fetched = svc.get_entity(id).await.unwrap();
        assert_eq!(fetched.entity_id, "light.living_room");
    }

    #[tokio::test]
    async fn should_reject_create_when_entity_id_is_empty() {
        let svc = make_service();
        let mut entity = valid_entity();
        entity.entity_id = String::new();

        let result = svc.create_entity(entity).await;
        assert!(matches!(
            result,
            Err(MiniHubError::Validation(ValidationError::EmptyEntityId))
        ));
    }

    #[tokio::test]
    async fn should_return_not_found_when_entity_missing() {
        let svc = make_service();
        let result = svc.get_entity(EntityId::new()).await;

        assert!(matches!(result, Err(MiniHubError::NotFound(_))));
    }

    #[tokio::test]
    async fn should_list_all_entities() {
        let svc = make_service();
        svc.create_entity(valid_entity()).await.unwrap();

        let mut entity2 = valid_entity();
        entity2.entity_id = "sensor.temperature".to_string();
        svc.create_entity(entity2).await.unwrap();

        let all = svc.list_entities().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn should_update_entity_state() {
        let svc = make_service();
        let entity = valid_entity();
        let id = entity.id;
        svc.create_entity(entity).await.unwrap();

        let updated = svc.update_entity_state(id, EntityState::On).await.unwrap();
        assert_eq!(updated.state, EntityState::On);

        let fetched = svc.get_entity(id).await.unwrap();
        assert_eq!(fetched.state, EntityState::On);
    }

    #[tokio::test]
    async fn should_return_not_found_when_updating_missing_entity() {
        let svc = make_service();
        let result = svc
            .update_entity_state(EntityId::new(), EntityState::On)
            .await;

        assert!(matches!(result, Err(MiniHubError::NotFound(_))));
    }

    #[tokio::test]
    async fn should_delete_entity() {
        let svc = make_service();
        let entity = valid_entity();
        let id = entity.id;
        svc.create_entity(entity).await.unwrap();

        svc.delete_entity(id).await.unwrap();

        let result = svc.get_entity(id).await;
        assert!(matches!(result, Err(MiniHubError::NotFound(_))));
    }
}
