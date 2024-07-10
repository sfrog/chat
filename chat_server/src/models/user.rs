use crate::{AppError, User};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::mem;

use super::{ChatUser, Workspace};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUser {
    pub fullname: String,
    pub email: String,
    pub workspace: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}

impl User {
    pub async fn find_by_email(email: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user = sqlx::query_as(
            "SELECT id, ws_id, fullname, email, created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;
        Ok(user)
    }

    pub async fn create(input: &CreateUser, pool: &PgPool) -> Result<Self, AppError> {
        let user = User::find_by_email(&input.email, pool).await?;
        if user.is_some() {
            return Err(AppError::EmailAlreadyExists(input.email.clone()));
        }

        // check is workspace exists, if not create one
        let ws = match Workspace::find_by_name(&input.workspace, pool).await? {
            Some(ws) => ws,
            None => Workspace::create(&input.workspace, 0, pool).await?,
        };

        let password_hash = hash_password(&input.password)?;
        let user: User = sqlx::query_as(
            r#"
            INSERT INTO users (fullname, email, ws_id, password_hash)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, fullname, email, created_at
            "#,
        )
        .bind(&input.fullname)
        .bind(&input.email)
        .bind(ws.id)
        .bind(password_hash)
        .fetch_one(pool)
        .await?;

        if ws.owner_id == 0 {
            ws.update_owner(user.id as _, pool).await?;
        }

        Ok(user)
    }

    // add user to workspace
    // pub async fn add_to_workspace(
    //     &self,
    //     workspace_id: i64,
    //     pool: &PgPool,
    // ) -> Result<User, AppError> {
    //     let user = sqlx::query_as(
    //         r#"
    //         UPDATE users
    //         SET ws_id = $1
    //         WHERE id = $2
    //         RETURNING id, ws_id, fullname, email, created_at
    //         "#,
    //     )
    //     .bind(workspace_id)
    //     .bind(self.id)
    //     .fetch_one(pool)
    //     .await?;

    //     Ok(user)
    // }

    pub async fn verify(input: &SigninUser, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user: Option<User> = sqlx::query_as(
            r#"
            SELECT id, ws_id, fullname, email, created_at, password_hash
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(&input.email)
        .fetch_optional(pool)
        .await?;

        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash);
                let matches = verify_password(&input.password, &password_hash.unwrap_or_default())?;
                if matches {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}

impl ChatUser {
    // pub async fn fetch_all() -> Result<Vec<Self>, AppError> {
    //     let users = sqlx::query_as("SELECT id, fullname, email, created_at FROM users")
    //         .fetch_all(&pool)
    //         .await?;
    //     Ok(users)
    // }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}

fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();

    let password_hash = PasswordHash::new(hash)?;

    let matches = argon2
        .verify_password(password.as_bytes(), &password_hash)
        .is_ok();

    Ok(matches)
}

#[cfg(test)]
impl User {
    pub fn new(id: i64, fullname: &str, email: &str) -> Self {
        Self {
            id,
            ws_id: 0,
            fullname: fullname.to_string(),
            email: email.to_string(),
            password_hash: None,
            created_at: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
impl CreateUser {
    pub fn new(ws: &str, fullname: &str, email: &str, password: &str) -> Self {
        Self {
            fullname: fullname.to_string(),
            email: email.to_string(),
            workspace: ws.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
impl SigninUser {
    pub fn new(email: &str, password: &str) -> Self {
        Self {
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use sqlx_db_tester::TestPg;
    use std::path::Path;

    #[tokio::test]
    async fn test_user() -> Result<()> {
        let db = TestPg::new(
            "postgres://postgres:postgres@localhost:5432".to_string(),
            Path::new("../migrations"),
        );

        let pool = db.get_pool().await;

        let email = "test@abc.com";
        let fullname = "wangcong";
        let password = "1234asdf";
        let input = CreateUser::new("default", fullname, email, password);
        let user = User::create(&input, &pool).await?;

        let ret = User::create(&input, &pool).await;
        assert!(ret.is_err());

        assert_eq!(user.email, email);
        assert_eq!(user.fullname, fullname);
        assert!(user.id > 0);

        let user = User::find_by_email(email, &pool).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, email);
        assert_eq!(user.fullname, fullname);

        let input = SigninUser::new(email, password);
        let user = User::verify(&input, &pool).await?;
        assert!(user.is_some());

        Ok(())
    }
}
