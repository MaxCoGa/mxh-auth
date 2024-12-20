use async_trait::async_trait;
use axum_login::{AuthUser, AuthnBackend, UserId};
use password_auth::verify_password;
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use tokio::task;

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    id: i64,
    pub username: String,
    password: String,
}

// Here we've implemented `Debug` manually to avoid accidentally logging the
// password hash.
impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("password", &"[redacted]")
            .finish()
    }
}

impl AuthUser for User {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password.as_bytes() // We use the password hash as the auth
                                 // hash--what this means
                                 // is when the user changes their password the
                                 // auth session becomes invalid.
    }
}

// This allows us to extract the authentication fields from forms. We use this
// to authenticate requests with the backend.
#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub next: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Backend {
    db: SqlitePool,
}

impl Backend {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    // pub async fn add_user(&self, username: &str, password: &str) -> Result<(), Error> 
    pub async fn add_user(&self, username: &str, password: &str) -> Result<i64, Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| {
                eprintln!("Error hashing password: {}", e);
                sqlx::Error::Protocol("Error hashing password".to_string())
            })?
            .to_string();

        // export DATABASE_URL="sqlite://./sessions.db"
        // TODO: switch to query_as
        let result = sqlx::query!(
                "INSERT INTO users (username, password) VALUES (?, ?)",
                username,
                password_hash
            )
            .execute(&self.db)
            .await?;
        // sqlx::query!(
        //     "INSERT INTO users (username, password) VALUES (?, ?)",
        //     username,
        //     password_hash
        // )
        // .execute(&self.db)
        // .await?;

        //Ok(())
        // The `last_insert_rowid()` function returns the ROWID of the last row insert from the database connection which invoked the function.
        let user_id = result.last_insert_rowid();
        Ok(user_id)
    }

    pub async fn remove_user(&self, username: &str) -> Result<Option<()>, Error> {    
        sqlx::query!("DELETE FROM users WHERE username = ?", username)
            .execute(&self.db)
            .await
            .map(|_| None)
            .map_err(Error::from)
        // let result = sqlx::query_as("DELETE FROM users WHERE username = ?")
        //     .bind(username)
        //     .fetch_optional(&self.db)
        //     .await?;
        // Ok(result.map(|_|()))
    }

}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    TaskJoin(#[from] task::JoinError),
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = Error;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user: Option<Self::User> = sqlx::query_as("select * from users where username = ? ")
            .bind(creds.username)
            .fetch_optional(&self.db)
            .await?;

        // Verifying the password is blocking and potentially slow, so we'll do so via
        // `spawn_blocking`.
        task::spawn_blocking(|| {
            // We're using password-based authentication--this works by comparing our form
            // input with an argon2 password hash.
            Ok(user.filter(|user| verify_password(creds.password, &user.password).is_ok()))
        })
        .await?
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as("select * from users where id = ?")
            .bind(user_id)
            .fetch_optional(&self.db)
            .await?;

        Ok(user)
    }


}

// We use a type alias for convenience.
//
// Note that we've supplied our concrete backend here.
pub type AuthSession = axum_login::AuthSession<Backend>;