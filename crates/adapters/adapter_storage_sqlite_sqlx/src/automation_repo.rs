//! `SQLite` implementation of [`AutomationRepository`].

use std::str::FromStr;

use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqlitePool};

use minihub_app::ports::AutomationRepository;
use minihub_domain::automation::{Action, Automation, Condition, Trigger};
use minihub_domain::error::MiniHubError;
use minihub_domain::id::AutomationId;

use crate::error::StorageError;

struct Wrapper(Automation);

impl Wrapper {
    fn maybe(value: Option<Self>) -> Option<Automation> {
        value.map(|w| w.0)
    }
}

impl<'r> FromRow<'r, SqliteRow> for Wrapper {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let id: String = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        let enabled: bool = row.try_get("enabled")?;
        let trigger_json: String = row.try_get("trigger_data")?;
        let conditions_json: String = row.try_get("conditions")?;
        let actions_json: String = row.try_get("actions")?;
        let last_triggered_str: Option<String> = row.try_get("last_triggered")?;

        let id = AutomationId::from_str(&id).map_err(|err| sqlx::Error::Decode(Box::new(err)))?;
        let trigger: Trigger = serde_json::from_str(&trigger_json)
            .map_err(|err| sqlx::Error::Decode(Box::new(err)))?;
        let conditions: Vec<Condition> = serde_json::from_str(&conditions_json)
            .map_err(|err| sqlx::Error::Decode(Box::new(err)))?;
        let actions: Vec<Action> = serde_json::from_str(&actions_json)
            .map_err(|err| sqlx::Error::Decode(Box::new(err)))?;
        let last_triggered = last_triggered_str
            .map(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.to_utc())
                    .map_err(|err| sqlx::Error::Decode(Box::new(err)))
            })
            .transpose()?;

        Ok(Self(Automation {
            id,
            name,
            enabled,
            trigger,
            conditions,
            actions,
            last_triggered,
        }))
    }
}

/// `SQLite`-backed automation repository.
pub struct SqliteAutomationRepository {
    pool: SqlitePool,
}

impl SqliteAutomationRepository {
    /// Create a new repository backed by the given connection pool.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl AutomationRepository for SqliteAutomationRepository {
    async fn create(&self, automation: Automation) -> Result<Automation, MiniHubError> {
        let id = automation.id.as_uuid();
        let trigger_json =
            serde_json::to_string(&automation.trigger).map_err(StorageError::from)?;
        let conditions_json =
            serde_json::to_string(&automation.conditions).map_err(StorageError::from)?;
        let actions_json =
            serde_json::to_string(&automation.actions).map_err(StorageError::from)?;
        let last_triggered = automation.last_triggered.map(|ts| ts.to_rfc3339());

        sqlx::query(
                "INSERT INTO automations (id, name, enabled, trigger_data, conditions, actions, last_triggered) VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&id)
            .bind(&automation.name)
            .bind(automation.enabled)
            .bind(&trigger_json)
            .bind(&conditions_json)
            .bind(&actions_json)
            .bind(&last_triggered)
            .execute(&self.pool)
            .await
            .map_err(StorageError::from)?;

        Ok(automation)
    }

    async fn get_by_id(&self, id: AutomationId) -> Result<Option<Automation>, MiniHubError> {
        let row: Option<Wrapper> = sqlx::query_as("SELECT * FROM automations WHERE id = ?")
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(StorageError::from)?;
        Ok(Wrapper::maybe(row))
    }

    async fn get_all(&self) -> Result<Vec<Automation>, MiniHubError> {
        let rows: Vec<Wrapper> = sqlx::query_as("SELECT * FROM automations ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::from)?;
        Ok(rows.into_iter().map(|w| w.0).collect())
    }

    async fn get_enabled(&self) -> Result<Vec<Automation>, MiniHubError> {
        let rows: Vec<Wrapper> =
            sqlx::query_as("SELECT * FROM automations WHERE enabled = 1 ORDER BY name")
                .fetch_all(&self.pool)
                .await
                .map_err(StorageError::from)?;
        Ok(rows.into_iter().map(|w| w.0).collect())
    }

    async fn update(&self, automation: Automation) -> Result<Automation, MiniHubError> {
        let id = automation.id.as_uuid();
        let trigger_json =
            serde_json::to_string(&automation.trigger).map_err(StorageError::from)?;
        let conditions_json =
            serde_json::to_string(&automation.conditions).map_err(StorageError::from)?;
        let actions_json =
            serde_json::to_string(&automation.actions).map_err(StorageError::from)?;
        let last_triggered = automation.last_triggered.map(|ts| ts.to_rfc3339());

        sqlx::query(
                "UPDATE automations SET name = ?, enabled = ?, trigger_data = ?, conditions = ?, actions = ?, last_triggered = ? WHERE id = ?",
            )
            .bind(&automation.name)
            .bind(automation.enabled)
            .bind(&trigger_json)
            .bind(&conditions_json)
            .bind(&actions_json)
            .bind(&last_triggered)
            .bind(&id)
            .execute(&self.pool)
            .await
            .map_err(StorageError::from)?;

        Ok(automation)
    }

    async fn delete(&self, id: AutomationId) -> Result<(), MiniHubError> {
        sqlx::query("DELETE FROM automations WHERE id = ?")
            .bind(id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(StorageError::from)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::Config;
    use minihub_domain::automation::Trigger;
    use minihub_domain::id::EntityId;

    async fn setup() -> SqliteAutomationRepository {
        let db = Config {
            database_url: "sqlite::memory:".to_string(),
        }
        .build()
        .await
        .unwrap();
        SqliteAutomationRepository::new(db.pool().clone())
    }

    fn valid_automation() -> Automation {
        Automation::builder()
            .name("Test rule")
            .trigger(Trigger::Manual)
            .action(Action::CallService {
                entity_id: EntityId::new(),
                service: "turn_on".to_string(),
                data: serde_json::json!({}),
            })
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn should_create_and_retrieve_automation() {
        let repo = setup().await;
        let auto = valid_automation();
        let id = auto.id;

        repo.create(auto).await.unwrap();
        let fetched = repo.get_by_id(id).await.unwrap().unwrap();
        assert_eq!(fetched.id, id);
        assert_eq!(fetched.name, "Test rule");
        assert!(fetched.enabled);
    }

    #[tokio::test]
    async fn should_return_none_when_automation_not_found() {
        let repo = setup().await;
        let result = repo.get_by_id(AutomationId::new()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn should_list_all_automations() {
        let repo = setup().await;
        repo.create(valid_automation()).await.unwrap();
        let mut auto2 = valid_automation();
        auto2.name = "Second rule".to_string();
        repo.create(auto2).await.unwrap();

        let all = repo.get_all().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn should_list_only_enabled_automations() {
        let repo = setup().await;
        repo.create(valid_automation()).await.unwrap();

        let mut disabled = valid_automation();
        disabled.name = "Disabled rule".to_string();
        disabled.enabled = false;
        repo.create(disabled).await.unwrap();

        let enabled = repo.get_enabled().await.unwrap();
        assert_eq!(enabled.len(), 1);
        assert!(enabled[0].enabled);
    }

    #[tokio::test]
    async fn should_update_automation() {
        let repo = setup().await;
        let auto = valid_automation();
        let id = auto.id;
        repo.create(auto).await.unwrap();

        let mut fetched = repo.get_by_id(id).await.unwrap().unwrap();
        fetched.name = "Updated name".to_string();
        fetched.enabled = false;
        repo.update(fetched).await.unwrap();

        let updated = repo.get_by_id(id).await.unwrap().unwrap();
        assert_eq!(updated.name, "Updated name");
        assert!(!updated.enabled);
    }

    #[tokio::test]
    async fn should_delete_automation() {
        let repo = setup().await;
        let auto = valid_automation();
        let id = auto.id;
        repo.create(auto).await.unwrap();

        repo.delete(id).await.unwrap();
        let result = repo.get_by_id(id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn should_preserve_trigger_and_actions_through_roundtrip() {
        let repo = setup().await;
        let eid = EntityId::new();
        let auto = Automation::builder()
            .name("Complex rule")
            .trigger(Trigger::StateChanged {
                entity_id: eid,
                from: Some(minihub_domain::entity::EntityState::Off),
                to: Some(minihub_domain::entity::EntityState::On),
            })
            .condition(minihub_domain::automation::Condition::StateIs {
                entity_id: eid,
                state: "on".to_string(),
            })
            .action(Action::CallService {
                entity_id: eid,
                service: "turn_off".to_string(),
                data: serde_json::json!({"brightness": 128}),
            })
            .action(Action::Delay { seconds: 5 })
            .build()
            .unwrap();
        let id = auto.id;

        repo.create(auto).await.unwrap();
        let fetched = repo.get_by_id(id).await.unwrap().unwrap();

        assert!(matches!(
            fetched.trigger,
            Trigger::StateChanged { entity_id, .. } if entity_id == eid
        ));
        assert_eq!(fetched.conditions.len(), 1);
        assert_eq!(fetched.actions.len(), 2);
    }
}
