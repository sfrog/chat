use crate::{AppError, User};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sqlx::PgPool;
use std::mem;

impl User {
    pub async fn find_by_email(email: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user =
            sqlx::query_as("SELECT id, fullname, email, created_at FROM users WHERE email = $1")
                .bind(email)
                .fetch_optional(pool)
                .await?;
        Ok(user)
    }

    pub async fn create(
        email: &str,
        fullname: &str,
        password: &str,
        pool: &PgPool,
    ) -> Result<Self, AppError> {
        let password_hash = hash_password(password)?;
        let user = sqlx::query_as(
            r#"
            INSERT INTO users (fullname, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, fullname, email, created_at
            "#,
        )
        .bind(fullname)
        .bind(email)
        .bind(password_hash)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn verify(
        email: &str,
        password: &str,
        pool: &PgPool,
    ) -> Result<Option<Self>, AppError> {
        let user: Option<User> = sqlx::query_as(
            r#"
            SELECT id, fullname, email, created_at, password_hash
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash);
                let matches = verify_password(password, &password_hash.unwrap_or_default())?;
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
        let user = User::create(email, fullname, password, &pool).await?;

        assert_eq!(user.email, email);
        assert_eq!(user.fullname, fullname);
        assert!(user.id > 0);

        let user = User::find_by_email(email, &pool).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, email);
        assert_eq!(user.fullname, fullname);

        let user = User::verify(email, password, &pool).await?;
        assert!(user.is_some());

        Ok(())
    }
}
