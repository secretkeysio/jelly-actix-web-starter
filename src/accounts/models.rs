// Implements a basic Account model, with support for creating/updating/deleting
// users, along with welcome email and verification.

use jelly::accounts::{User, OneTimeUseTokenGenerator};
use jelly::djangohashers::{make_password, check_password};
use jelly::serde::{Deserialize, Serialize};
use jelly::chrono::{DateTime, Utc};
use jelly::error::Error;
use jelly::sqlx;
use jelly::sqlx::{postgres::PgPool, FromRow, types::Json};

use super::forms::{LoginForm, NewAccountForm};

/// Personalized profile data that is a pain to make a needless JOIN
/// for; just shove it in a jsonb field.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Profile {}

/// A user Account.
#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: String,
    pub profile: Json<Profile>,
    pub plan: i32,
    pub is_active: bool,
    pub is_admin: bool,
    pub has_verified_email: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>
}

impl Account {
    pub async fn get(uid: i32, pool: &PgPool) -> Result<Self, Error> {
        Ok(sqlx::query_as_unchecked!(Account, "
            SELECT
                id, name, email, password, profile, plan,
                is_active, is_admin, has_verified_email,
                last_login, created, updated
            FROM accounts WHERE id = $1
        ", uid).fetch_one(pool).await?)
    }

    pub async fn get_by_email(email: &str, pool: &PgPool) -> Result<Self, Error> {
        Ok(sqlx::query_as_unchecked!(Account, "
            SELECT 
                id, name, email, password, profile, plan,
                is_active, is_admin, has_verified_email,
                last_login, created, updated
            FROM accounts WHERE email = $1
        ", email).fetch_one(pool).await?)
    }

    pub async fn authenticate(form: &LoginForm, pool: &PgPool) -> Result<User, Error> {
        let user = sqlx::query!("
            SELECT
                id, name, password, is_admin
            FROM accounts WHERE email = $1
        ", form.email.value).fetch_one(pool).await?;

        if !check_password(&form.password, &user.password)? {
            return Err(Error::InvalidPassword);
        }

        Ok(User {
            id: user.id,
            name: user.name,
            is_admin: user.is_admin,
            is_anonymous: false
        })
    }
    
    pub async fn fetch_email(id: i32, pool: &PgPool) -> Result<(String, String), Error> {
        let data = sqlx::query!("
            SELECT
                name, email
            FROM accounts WHERE id = $1
        ", id).fetch_one(pool).await?;

        Ok((data.name, data.email))
    }

    pub async fn fetch_name_from_email(email: &str, pool: &PgPool) -> Result<String, Error> {
        let data = sqlx::query!("
            SELECT name FROM accounts WHERE email = $1
        ", email).fetch_one(pool).await?;

        Ok(data.name)
    }

    pub async fn register(form: &NewAccountForm, pool: &PgPool) -> Result<i32, Error> {
        let password = make_password(&form.password);

        Ok(sqlx::query!("
            INSERT INTO accounts (name, email, password) 
            VALUES ($1, $2, $3)
            RETURNING id
        ", form.name.value, form.email.value, password).fetch_one(pool).await?.id)
    }

    pub async fn mark_verified(id: i32, pool: &PgPool) -> Result<(), Error> {
        sqlx::query!("
            UPDATE accounts
            SET has_verified_email = true, last_login = now()
            WHERE id = $1
        ", id).execute(pool).await?;

        Ok(())
    }

    pub async fn update_last_login(id: i32, pool: &PgPool) -> Result<(), Error> {
        sqlx::query!("
            UPDATE accounts
            SET last_login = now()
            WHERE id = $1
        ", id).execute(pool).await?;

        Ok(())
    }

    pub async fn update_password_and_last_login(
        id: i32,
        password: &str,
        pool: &PgPool
    ) -> Result<(), Error> {
        let password = make_password(&password);
        
        sqlx::query!("
            UPDATE accounts
            SET password = $2, last_login = now()
            WHERE id = $1
        ", id, password).execute(pool).await?;

        Ok(())
    }
}

impl OneTimeUseTokenGenerator for Account {
    fn hash_value(&self) -> String {
        format!(
            "{}{}{}{}",
            self.id,
            self.password,
            match self.last_login {
                Some(ts) => format!("{}", ts.timestamp()),
                None => "Unverified".to_string()
            },
            self.email
        )
    }
}
