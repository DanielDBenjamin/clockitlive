use serde::{Deserialize, Serialize};

// Safe user profile (no password) - available for both client and server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: i64,
    pub name: String,
    pub surname: String,
    pub email_address: String,
    pub role: String,
    pub university: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegisterData {
    pub name: String,
    pub surname: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
    pub role: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<UserProfile>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BasicResponse {
    pub success: bool,
    pub message: String,
}
