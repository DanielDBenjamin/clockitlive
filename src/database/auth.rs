#[cfg(feature = "ssr")]
use crate::database::models::{CreateUserRequest, User};
#[cfg(feature = "ssr")]
use crate::types::UserProfile;
#[cfg(feature = "ssr")]
use chrono::Utc;
#[cfg(feature = "ssr")]
use sqlx::SqlitePool;

/// Simple password hashing (for development only - replace with bcrypt in production)
#[cfg(feature = "ssr")]
fn hash_password(password: &str) -> String {
    // Simple hash for development - in production use bcrypt
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    password.hash(&mut hasher);
    format!("simple_hash_{}", hasher.finish())
}

/// Simple password verification (for development only)
#[cfg(feature = "ssr")]
fn verify_password(password: &str, hash: &str) -> bool {
    hash_password(password) == hash
}

/// Create a new user account
#[cfg(feature = "ssr")]
pub async fn create_user(
    pool: &SqlitePool,
    request: CreateUserRequest,
) -> Result<UserProfile, String> {
    // Check if user already exists
    let existing_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE emailAddress = ?")
        .bind(&request.email)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    if existing_user.is_some() {
        return Err("User with this email already exists".to_string());
    }

    // Hash the password
    let password_hash = hash_password(&request.password);

    // Create timestamp
    let now = Utc::now().to_rfc3339();

    // Insert new user
    let result = sqlx::query(
        r#"
        INSERT INTO users (name, surname, emailAddress, password, role, university, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&request.name)
    .bind(&request.surname)
    .bind(&request.email)
    .bind(&password_hash)
    .bind(&request.role)
    .bind("Stellenbosch University")
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create user: {}", e))?;

    // Get the created user
    let user_id = result.last_insert_rowid();

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE userID = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to fetch created user: {}", e))?;

    Ok(user.into())
}

/// Authenticate user login
#[cfg(feature = "ssr")]
pub async fn authenticate_user(
    pool: &SqlitePool,
    email: &str,
    password: &str,
) -> Result<UserProfile, String> {
    // Find user by email
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE emailAddress = ?")
        .bind(email)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    let user = match user {
        Some(user) => user,
        None => return Err("Invalid email or password".to_string()),
    };

    // Verify password
    let password_valid = verify_password(password, &user.password);

    if !password_valid {
        return Err("Invalid email or password".to_string());
    }

    Ok(user.into())
}

/// Get user by ID
#[cfg(feature = "ssr")]
pub async fn get_user_by_id(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<Option<UserProfile>, String> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE userID = ?")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(user.map(|u| u.into()))
}

/// Get user by email
#[cfg(feature = "ssr")]
pub async fn get_user_by_email(
    pool: &SqlitePool,
    email: &str,
) -> Result<Option<UserProfile>, String> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE emailAddress = ?")
        .bind(email)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(user.map(|u| u.into()))
}

#[cfg(feature = "ssr")]
pub fn print_test_hash() {
    let password = "password123";
    let hash = hash_password(password);
    println!("\n=================================");
    println!("Password: {}", password);
    println!("Hash: {}", hash);
    println!("=================================\n");
}
#[cfg(feature = "ssr")]
pub async fn update_user_password_by_email(
    pool: &SqlitePool,
    email: &str,
    new_password: &str,
) -> Result<(), String> {
    let hashed = hash_password(new_password);
    let now = Utc::now().to_rfc3339();
    let result =
        sqlx::query("UPDATE users SET password = ?, updated_at = ? WHERE emailAddress = ?")
            .bind(&hashed)
            .bind(&now)
            .bind(&email.to_lowercase())
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to update password: {}", e))?;

    if result.rows_affected() == 0 {
        return Err("No user found with that email".to_string());
    }

    Ok(())
}
