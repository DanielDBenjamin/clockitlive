use crate::types::UserProfile;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use sqlx::FromRow;

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    #[sqlx(rename = "userID")]
    #[serde(rename = "userID")]
    pub user_id: i64,
    pub name: String,
    pub surname: String,
    #[sqlx(rename = "emailAddress")]
    #[serde(rename = "emailAddress")]
    pub email_address: String,
    pub password: String,
    pub role: String,
    pub university: String,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub surname: String,
    pub email: String,
    pub password: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[cfg(feature = "ssr")]
impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        UserProfile {
            user_id: user.user_id,
            name: user.name,
            surname: user.surname,
            email_address: user.email_address,
            role: user.role,
            university: user.university,
        }
    }
}
