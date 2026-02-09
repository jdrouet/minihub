//! `SQLite` implementation of [`AreaRepository`].

use std::future::Future;
use std::str::FromStr;

use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqlitePool};

use minihub_app::ports::AreaRepository;
use minihub_domain::area::Area;
use minihub_domain::error::MiniHubError;
use minihub_domain::id::AreaId;

use crate::error::StorageError;

/// Wrapper for converting database rows into domain [`Area`].
struct Wrapper(Area);

impl Wrapper {
    fn maybe(value: Option<Self>) -> Option<Area> {
        value.map(|w| w.0)
    }
}

impl<'r> FromRow<'r, SqliteRow> for Wrapper {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let id: String = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        let parent_id: Option<String> = row.try_get("parent_id")?;

        let id = AreaId::from_str(&id).map_err(|err| sqlx::Error::Decode(Box::new(err)))?;
        let parent_id = parent_id
            .map(|s| AreaId::from_str(&s))
            .transpose()
            .map_err(|err| sqlx::Error::Decode(Box::new(err)))?;

        Ok(Self(Area {
            id,
            name,
            parent_id,
        }))
    }
}

const INSERT: &str = "INSERT INTO areas (id, name, parent_id) VALUES (?, ?, ?)";
const SELECT_BY_ID: &str = "SELECT * FROM areas WHERE id = ?";
const SELECT_ALL: &str = "SELECT * FROM areas";
const UPDATE: &str = "UPDATE areas SET name = ?, parent_id = ? WHERE id = ?";
const DELETE_BY_ID: &str = "DELETE FROM areas WHERE id = ?";

/// `SQLite`-backed area repository.
pub struct SqliteAreaRepository {
    pool: SqlitePool,
}

impl SqliteAreaRepository {
    /// Create a new repository using the given connection pool.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl AreaRepository for SqliteAreaRepository {
    fn create(&self, area: Area) -> impl Future<Output = Result<Area, MiniHubError>> + Send {
        let pool = self.pool.clone();
        async move {
            sqlx::query(INSERT)
                .bind(area.id.to_string())
                .bind(&area.name)
                .bind(area.parent_id.map(|id| id.to_string()))
                .execute(&pool)
                .await
                .map_err(StorageError::from)?;

            Ok(area)
        }
    }

    fn get_by_id(
        &self,
        id: AreaId,
    ) -> impl Future<Output = Result<Option<Area>, MiniHubError>> + Send {
        let pool = self.pool.clone();
        async move {
            let row: Option<Wrapper> = sqlx::query_as(SELECT_BY_ID)
                .bind(id.to_string())
                .fetch_optional(&pool)
                .await
                .map_err(StorageError::from)?;

            Ok(Wrapper::maybe(row))
        }
    }

    fn get_all(&self) -> impl Future<Output = Result<Vec<Area>, MiniHubError>> + Send {
        let pool = self.pool.clone();
        async move {
            let rows: Vec<Wrapper> = sqlx::query_as(SELECT_ALL)
                .fetch_all(&pool)
                .await
                .map_err(StorageError::from)?;

            Ok(rows.into_iter().map(|w| w.0).collect())
        }
    }

    fn update(&self, area: Area) -> impl Future<Output = Result<Area, MiniHubError>> + Send {
        let pool = self.pool.clone();
        async move {
            sqlx::query(UPDATE)
                .bind(&area.name)
                .bind(area.parent_id.map(|id| id.to_string()))
                .bind(area.id.to_string())
                .execute(&pool)
                .await
                .map_err(StorageError::from)?;

            Ok(area)
        }
    }

    fn delete(&self, id: AreaId) -> impl Future<Output = Result<(), MiniHubError>> + Send {
        let pool = self.pool.clone();
        async move {
            sqlx::query(DELETE_BY_ID)
                .bind(id.to_string())
                .execute(&pool)
                .await
                .map_err(StorageError::from)?;

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::Config;

    async fn setup() -> SqliteAreaRepository {
        let db = Config {
            database_url: "sqlite::memory:".to_string(),
        }
        .build()
        .await
        .unwrap();
        SqliteAreaRepository::new(db.pool().clone())
    }

    fn test_area() -> Area {
        Area::builder().name("Living Room").build().unwrap()
    }

    #[tokio::test]
    async fn should_create_and_retrieve_area_when_valid() {
        let repo = setup().await;
        let area = test_area();
        let id = area.id;

        repo.create(area).await.unwrap();

        let fetched = repo.get_by_id(id).await.unwrap().unwrap();
        assert_eq!(fetched.id, id);
        assert_eq!(fetched.name, "Living Room");
    }

    #[tokio::test]
    async fn should_return_none_when_area_not_found() {
        let repo = setup().await;
        let result = repo.get_by_id(AreaId::new()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn should_list_all_areas() {
        let repo = setup().await;
        repo.create(test_area()).await.unwrap();
        repo.create(Area::builder().name("Kitchen").build().unwrap())
            .await
            .unwrap();

        let all = repo.get_all().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn should_update_area_when_exists() {
        let repo = setup().await;
        let mut area = test_area();
        let id = area.id;
        repo.create(area.clone()).await.unwrap();

        area.name = "Updated Room".to_string();
        repo.update(area).await.unwrap();

        let fetched = repo.get_by_id(id).await.unwrap().unwrap();
        assert_eq!(fetched.name, "Updated Room");
    }

    #[tokio::test]
    async fn should_delete_area_when_exists() {
        let repo = setup().await;
        let area = test_area();
        let id = area.id;
        repo.create(area).await.unwrap();

        repo.delete(id).await.unwrap();

        let result = repo.get_by_id(id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn should_store_parent_id_through_roundtrip() {
        let repo = setup().await;
        let parent = test_area();
        let parent_id = parent.id;
        repo.create(parent).await.unwrap();

        let child = Area::builder()
            .name("Bedroom")
            .parent_id(parent_id)
            .build()
            .unwrap();
        let child_id = child.id;
        repo.create(child).await.unwrap();

        let fetched = repo.get_by_id(child_id).await.unwrap().unwrap();
        assert_eq!(fetched.parent_id, Some(parent_id));
    }
}
