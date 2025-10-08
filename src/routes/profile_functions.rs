use crate::types::UserProfile;
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::database::init_db_pool;
#[cfg(feature = "ssr")]
use chrono::Utc;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UpdateProfileRequest {
    pub user_id: i64,
    pub name: String,
    pub surname: String,
    pub email_address: String,
    pub university: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProfileResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<UserProfile>,
}

/// Update user profile
#[server(UpdateProfile, "/api")]
pub async fn update_profile(
    request: UpdateProfileRequest,
) -> Result<ProfileResponse, ServerFnError> {
    // Validate input
    if request.name.trim().is_empty() {
        return Ok(ProfileResponse {
            success: false,
            message: "Name is required".to_string(),
            user: None,
        });
    }

    if request.surname.trim().is_empty() {
        return Ok(ProfileResponse {
            success: false,
            message: "Surname is required".to_string(),
            user: None,
        });
    }

    if request.email_address.trim().is_empty() {
        return Ok(ProfileResponse {
            success: false,
            message: "Email is required".to_string(),
            user: None,
        });
    }

    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let now = Utc::now().to_rfc3339();

    // Update user
    sqlx::query(
        r#"
        UPDATE users
        SET name = ?, surname = ?, emailAddress = ?, university = ?, updated_at = ?
        WHERE userID = ?
        "#,
    )
    .bind(&request.name.trim())
    .bind(&request.surname.trim())
    .bind(&request.email_address.trim())
    .bind(&request.university.trim())
    .bind(&now)
    .bind(request.user_id)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to update profile: {}", e)))?;

    // Fetch updated user
    let user = sqlx::query_as::<_, (i64, String, String, String, String, String)>(
        "SELECT userID, name, surname, emailAddress, role, university FROM users WHERE userID = ?",
    )
    .bind(request.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to fetch updated user: {}", e)))?;

    Ok(ProfileResponse {
        success: true,
        message: "Profile updated successfully!".to_string(),
        user: Some(UserProfile {
            user_id: user.0,
            name: user.1,
            surname: user.2,
            email_address: user.3,
            role: user.4,
            university: user.5,
        }),
    })
}
