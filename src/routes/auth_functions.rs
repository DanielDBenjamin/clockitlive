#[cfg(feature = "ssr")]
use crate::database::models::User;
#[cfg(feature = "ssr")]
use crate::database::{
    authenticate_user, create_user, init_db_pool, update_user_password_by_email, CreateUserRequest,
};
use crate::types::{AuthResponse, BasicResponse, RegisterData};
use leptos::prelude::*;

#[server(RegisterUser, "/api")]
pub async fn register_user(data: RegisterData) -> Result<AuthResponse, ServerFnError> {
    // Validate input
    if data.name.trim().is_empty() {
        return Ok(AuthResponse {
            success: false,
            message: "Name is required".to_string(),
            user: None,
        });
    }

    if data.surname.trim().is_empty() {
        return Ok(AuthResponse {
            success: false,
            message: "Surname is required".to_string(),
            user: None,
        });
    }

    if data.email.trim().is_empty() {
        return Ok(AuthResponse {
            success: false,
            message: "Email is required".to_string(),
            user: None,
        });
    }

    if data.password.len() < 6 {
        return Ok(AuthResponse {
            success: false,
            message: "Password must be at least 6 characters".to_string(),
            user: None,
        });
    }

    if data.password != data.confirm_password {
        return Ok(AuthResponse {
            success: false,
            message: "Passwords do not match".to_string(),
            user: None,
        });
    }

    if !["lecturer", "tutor", "student"].contains(&data.role.as_str()) {
        return Ok(AuthResponse {
            success: false,
            message: "Invalid role selected".to_string(),
            user: None,
        });
    }

    // Initialize database connection
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    // Create user request
    let create_request = CreateUserRequest {
        name: data.name.trim().to_string(),
        surname: data.surname.trim().to_string(),
        email: data.email.trim().to_lowercase(),
        password: data.password,
        role: data.role,
    };

    // Create user
    match create_user(&pool, create_request).await {
        Ok(user) => Ok(AuthResponse {
            success: true,
            message: "Account created successfully!".to_string(),
            user: Some(user),
        }),
        Err(e) => Ok(AuthResponse {
            success: false,
            message: e,
            user: None,
        }),
    }
}

#[server(LoginUser, "/api")]
pub async fn login_user(email: String, password: String) -> Result<AuthResponse, ServerFnError> {
    // Validate input
    if email.trim().is_empty() {
        return Ok(AuthResponse {
            success: false,
            message: "Email is required".to_string(),
            user: None,
        });
    }

    if password.trim().is_empty() {
        return Ok(AuthResponse {
            success: false,
            message: "Password is required".to_string(),
            user: None,
        });
    }

    // Initialize database connection
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let identifier = email.trim();
    let candidate_email = if identifier.contains('@') {
        identifier.to_lowercase()
    } else {
        let digits: String = identifier.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            return Ok(AuthResponse {
                success: false,
                message: "Please enter a valid email address or student ID".to_string(),
                user: None,
            });
        }

        let student_id = match digits.parse::<i64>() {
            Ok(id) => id,
            Err(_) => {
                return Ok(AuthResponse {
                    success: false,
                    message: "Invalid student ID format".to_string(),
                    user: None,
                });
            }
        };

        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE userID = ?")
            .bind(student_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

        match user {
            Some(u) => u.email_address.to_lowercase(),
            None => {
                return Ok(AuthResponse {
                    success: false,
                    message: "No account found with that student ID".to_string(),
                    user: None,
                });
            }
        }
    };

    // Authenticate user
    match authenticate_user(&pool, &candidate_email, &password).await {
        Ok(user) => Ok(AuthResponse {
            success: true,
            message: "Login successful!".to_string(),
            user: Some(user),
        }),
        Err(e) => Ok(AuthResponse {
            success: false,
            message: e,
            user: None,
        }),
    }
}

#[server(ResetPassword, "/api")]
pub async fn reset_password(
    email: String,
    new_password: String,
    confirm_password: String,
) -> Result<BasicResponse, ServerFnError> {
    if email.trim().is_empty() {
        return Ok(BasicResponse {
            success: false,
            message: "Email is required".to_string(),
        });
    }

    if new_password.len() < 6 {
        return Ok(BasicResponse {
            success: false,
            message: "Password must be at least 6 characters".to_string(),
        });
    }

    if new_password != confirm_password {
        return Ok(BasicResponse {
            success: false,
            message: "Passwords do not match".to_string(),
        });
    }

    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    match update_user_password_by_email(&pool, &email.trim().to_lowercase(), &new_password).await {
        Ok(_) => Ok(BasicResponse {
            success: true,
            message: "Password updated successfully. You can now sign in with your new password."
                .to_string(),
        }),
        Err(e) => Ok(BasicResponse {
            success: false,
            message: e,
        }),
    }
}
