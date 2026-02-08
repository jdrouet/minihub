//! Area service — use-cases for managing areas.

use minihub_domain::area::Area;
use minihub_domain::error::{MiniHubError, NotFoundError};
use minihub_domain::id::AreaId;

use crate::ports::AreaRepository;

/// Application service for area CRUD operations.
pub struct AreaService<R> {
    repo: R,
}

impl<R: AreaRepository> AreaService<R> {
    /// Create a new service backed by the given repository.
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Create a new area after validating domain invariants.
    ///
    /// # Errors
    ///
    /// Returns [`MiniHubError::Validation`] if invariants fail, or a
    /// storage error propagated from the repository.
    pub async fn create_area(&self, area: Area) -> Result<Area, MiniHubError> {
        area.validate()?;
        self.repo.create(area).await
    }

    /// Look up an area by id, returning an error if not found.
    ///
    /// # Errors
    ///
    /// Returns [`MiniHubError::NotFound`] when no area with `id` exists,
    /// or a storage error from the repository.
    pub async fn get_area(&self, id: AreaId) -> Result<Area, MiniHubError> {
        self.repo.get_by_id(id).await?.ok_or_else(|| {
            NotFoundError {
                entity: "Area",
                id: id.to_string(),
            }
            .into()
        })
    }

    /// List all areas.
    ///
    /// # Errors
    ///
    /// Returns a storage error propagated from the repository.
    pub async fn list_areas(&self) -> Result<Vec<Area>, MiniHubError> {
        self.repo.get_all().await
    }

    /// Update an existing area.
    ///
    /// # Errors
    ///
    /// Returns [`MiniHubError::Validation`] if invariants fail, or a
    /// storage error from the repository.
    pub async fn update_area(&self, area: Area) -> Result<Area, MiniHubError> {
        area.validate()?;
        self.repo.update(area).await
    }

    /// Delete an area by id.
    ///
    /// # Errors
    ///
    /// Returns a storage error propagated from the repository.
    pub async fn delete_area(&self, id: AreaId) -> Result<(), MiniHubError> {
        self.repo.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use minihub_domain::error::ValidationError;
    use std::collections::HashMap;
    use std::future::Future;
    use std::sync::Mutex;

    struct InMemoryAreaRepo {
        store: Mutex<HashMap<AreaId, Area>>,
    }

    impl Default for InMemoryAreaRepo {
        fn default() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }
    }

    impl AreaRepository for InMemoryAreaRepo {
        fn create(&self, area: Area) -> impl Future<Output = Result<Area, MiniHubError>> + Send {
            let mut store = self.store.lock().unwrap();
            store.insert(area.id, area.clone());
            async { Ok(area) }
        }

        fn get_by_id(
            &self,
            id: AreaId,
        ) -> impl Future<Output = Result<Option<Area>, MiniHubError>> + Send {
            let store = self.store.lock().unwrap();
            let result = store.get(&id).cloned();
            async { Ok(result) }
        }

        fn get_all(&self) -> impl Future<Output = Result<Vec<Area>, MiniHubError>> + Send {
            let store = self.store.lock().unwrap();
            let result: Vec<Area> = store.values().cloned().collect();
            async { Ok(result) }
        }

        fn update(&self, area: Area) -> impl Future<Output = Result<Area, MiniHubError>> + Send {
            let mut store = self.store.lock().unwrap();
            store.insert(area.id, area.clone());
            async { Ok(area) }
        }

        fn delete(&self, id: AreaId) -> impl Future<Output = Result<(), MiniHubError>> + Send {
            let mut store = self.store.lock().unwrap();
            store.remove(&id);
            async { Ok(()) }
        }
    }

    fn make_service() -> AreaService<InMemoryAreaRepo> {
        AreaService::new(InMemoryAreaRepo::default())
    }

    fn valid_area() -> Area {
        Area::builder().name("Living Room").build().unwrap()
    }

    #[tokio::test]
    async fn should_create_area_when_valid() {
        let svc = make_service();
        let area = valid_area();
        let id = area.id;

        let created = svc.create_area(area).await.unwrap();
        assert_eq!(created.id, id);

        let fetched = svc.get_area(id).await.unwrap();
        assert_eq!(fetched.name, "Living Room");
    }

    #[tokio::test]
    async fn should_reject_create_when_name_is_empty() {
        let svc = make_service();
        let mut area = valid_area();
        area.name = String::new();

        let result = svc.create_area(area).await;
        assert!(matches!(
            result,
            Err(MiniHubError::Validation(ValidationError::EmptyName))
        ));
    }

    #[tokio::test]
    async fn should_return_not_found_when_area_missing() {
        let svc = make_service();
        let result = svc.get_area(AreaId::new()).await;
        assert!(matches!(result, Err(MiniHubError::NotFound(_))));
    }

    #[tokio::test]
    async fn should_list_all_areas() {
        let svc = make_service();
        svc.create_area(valid_area()).await.unwrap();
        svc.create_area(Area::builder().name("Kitchen").build().unwrap())
            .await
            .unwrap();

        let all = svc.list_areas().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn should_update_area() {
        let svc = make_service();
        let area = valid_area();
        let id = area.id;
        svc.create_area(area).await.unwrap();

        let mut updated = svc.get_area(id).await.unwrap();
        updated.name = "Bedroom".to_string();
        let saved = svc.update_area(updated).await.unwrap();
        assert_eq!(saved.name, "Bedroom");
    }

    #[tokio::test]
    async fn should_delete_area() {
        let svc = make_service();
        let area = valid_area();
        let id = area.id;
        svc.create_area(area).await.unwrap();

        svc.delete_area(id).await.unwrap();

        let result = svc.get_area(id).await;
        assert!(matches!(result, Err(MiniHubError::NotFound(_))));
    }

    #[tokio::test]
    async fn should_create_area_with_parent() {
        let svc = make_service();
        let parent = valid_area();
        let parent_id = parent.id;
        svc.create_area(parent).await.unwrap();

        let child = Area::builder()
            .name("Master Bedroom")
            .parent_id(parent_id)
            .build()
            .unwrap();
        let created = svc.create_area(child).await.unwrap();
        assert_eq!(created.parent_id, Some(parent_id));
    }
}
