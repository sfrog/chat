use sqlx::PgPool;

use crate::AppError;

use super::{ChatUser, Workspace};

impl Workspace {
    pub async fn create(name: &str, user_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            r#"
            INSERT INTO workspaces (name, owner_id)
            VALUES ($1, $2)
            RETURNING id, name, owner_id, created_at
            "#,
        )
        .bind(name)
        .bind(user_id as i64)
        .fetch_one(pool)
        .await?;

        Ok(ws)
    }

    pub async fn update_owner(&self, owner_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            r#"
            UPDATE workspaces
            SET owner_id = $1
            WHERE id = $2 and (SELECT ws_id FROM users WHERE id = $1) = $2
            RETURNING id, name, owner_id, created_at
            "#,
        )
        .bind(owner_id as i64)
        .bind(self.id)
        .fetch_one(pool)
        .await?;

        Ok(ws)
    }

    pub async fn find_by_name(name: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id, name, owner_id, created_at
            FROM workspaces
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;

        Ok(ws)
    }

    #[allow(dead_code)]
    pub async fn find_by_id(id: u64, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id, name, owner_id, created_at
            FROM workspaces
            WHERE id = $1
            "#,
        )
        .bind(id as i64)
        .fetch_optional(pool)
        .await?;

        Ok(ws)
    }

    #[allow(dead_code)]
    pub async fn fetch_all_chat_users(id: u64, pool: &PgPool) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
            SELECT id, fullname, email
            FROM users
            WHERE ws_id = $1 order by id
            "#,
        )
        .bind(id as i64)
        .fetch_all(pool)
        .await?;

        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{models::CreateUser, User};

    use super::*;
    use anyhow::Result;
    use sqlx_db_tester::TestPg;

    #[tokio::test]
    async fn test_create_workspace_and_set_owner() -> Result<()> {
        let tdb = TestPg::new(
            "postgres://postgres:postgres@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;

        let ws = Workspace::create("test", 0, &pool).await.unwrap();
        assert_eq!(ws.name, "test");

        let input = CreateUser::new(&ws.name, "test_workspace", "test_workspace", "password");
        let user = User::create(&input, &pool).await?;
        assert_eq!(user.ws_id, ws.id);

        let ws = ws.update_owner(user.id as _, &pool).await?;
        assert_eq!(ws.owner_id, user.id);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_name() -> Result<()> {
        let tdb = TestPg::new(
            "postgres://postgres:postgres@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;

        Workspace::create("test", 0, &pool).await?;
        let ws = Workspace::find_by_name("test", &pool).await?;

        assert!(ws.is_some());
        assert_eq!(ws.unwrap().name, "test");

        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_all_chat_users() -> Result<()> {
        let tdb = TestPg::new(
            "postgres://postgres:postgres@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;

        let ws = Workspace::create("test", 0, &pool).await?;
        let input = CreateUser::new(&ws.name, "name", "email", "password");
        let user1 = User::create(&input, &pool).await?;
        let input = CreateUser::new(&ws.name, "name2", "email2", "password");
        let user2 = User::create(&input, &pool).await?;

        let users = Workspace::fetch_all_chat_users(ws.id as _, &pool).await?;
        assert_eq!(users.len(), 2);
        assert_eq!(user1.id, users[0].id);
        assert_eq!(user2.id, users[1].id);

        Ok(())
    }
}
