#[cfg(feature = "ssr")]
use crate::database::models::User;
#[cfg(feature = "ssr")]
use crate::database::{
    authenticate_user, create_user, init_db_pool, update_user_password_by_email, CreateUserRequest,
};
use crate::types::{AuthResponse, BasicResponse, RegisterData};
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use std::collections::HashMap;
#[cfg(feature = "ssr")]
use std::sync::Mutex;
#[cfg(feature = "ssr")]
use rand::Rng;
#[cfg(feature = "ssr")]
use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};


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

// Simple in-memory OTP storage (in production, use Redis or database)
#[cfg(feature = "ssr")]
lazy_static::lazy_static! {
    static ref OTP_STORE: Mutex<HashMap<String, (String, std::time::SystemTime)>> = Mutex::new(HashMap::new());
}

#[cfg(feature = "ssr")]
async fn send_email_otp(to_email: &str, otp: &str) -> Result<(), String> {
    use std::time::Duration;
    
    // Get email credentials from environment variables
    let smtp_username = std::env::var("SMTP_USERNAME")
        .map_err(|_| "SMTP_USERNAME environment variable not set")?;
    let smtp_password = std::env::var("SMTP_PASSWORD")
        .map_err(|_| "SMTP_PASSWORD environment variable not set")?;
    let smtp_host = std::env::var("SMTP_HOST")
        .unwrap_or_else(|_| "smtp.gmail.com".to_string());
    let smtp_port = std::env::var("SMTP_PORT")
        .unwrap_or_else(|_| "587".to_string())
        .parse::<u16>()
        .unwrap_or(587);
    let from_name = std::env::var("SMTP_FROM_NAME")
        .unwrap_or_else(|_| "Clock It".to_string());

    println!("📧 Attempting to send email via {}:{}", smtp_host, smtp_port);

    // Create email message
    let email = Message::builder()
        .from(format!("{} <{}>", from_name, smtp_username).parse()
            .map_err(|e| format!("Invalid from address: {}", e))?)
        .to(to_email.parse()
            .map_err(|e| format!("Invalid to address: {}", e))?)
        .subject("Your Clock It Verification Code")
        .header(ContentType::TEXT_HTML)
        .body(format!(
            r#"
            <div style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
                <div style="text-align: center; margin-bottom: 30px;">
                    <h1 style="color: #2563eb; margin: 0;">Clock It</h1>
                </div>
                
                <div style="background: #f8fafc; border-radius: 8px; padding: 30px; text-align: center;">
                    <h2 style="color: #1f2937; margin-bottom: 20px;">Verify Your Email</h2>
                    <p style="color: #4b5563; margin-bottom: 30px;">
                        Enter this verification code to complete your Clock It registration:
                    </p>
                    
                    <div style="background: white; border: 2px solid #e5e7eb; border-radius: 8px; padding: 20px; margin: 20px 0; display: inline-block;">
                        <span style="font-size: 32px; font-weight: bold; color: #2563eb; letter-spacing: 8px;">{}</span>
                    </div>
                    
                    <p style="color: #6b7280; font-size: 14px; margin-top: 20px;">
                        This code will expire in 5 minutes for security reasons.
                    </p>
                </div>
                
                <div style="text-align: center; margin-top: 30px; padding-top: 20px; border-top: 1px solid #e5e7eb;">
                    <p style="color: #9ca3af; font-size: 12px;">
                        If you didn't request this code, please ignore this email.
                    </p>
                </div>
            </div>
            "#, otp
        ))
        .map_err(|e| format!("Failed to build email: {}", e))?;

    // Create SMTP transport with timeout and proper configuration
    let creds = Credentials::new(smtp_username, smtp_password);
    
    let mailer = SmtpTransport::relay(&smtp_host)
        .map_err(|e| format!("Failed to create SMTP transport: {}", e))?
        .credentials(creds)
        .timeout(Some(Duration::from_secs(10)))  // 10 second timeout
        .port(smtp_port)  // Use configured port
        .build();

    println!("📤 Sending email to: {}", to_email);
    
    // Send email
    mailer.send(&email)
        .map_err(|e| format!("Failed to send email: {}. Check SMTP credentials and network.", e))?;

    println!("✅ Email sent successfully");
    Ok(())
}

#[cfg(feature = "ssr")]
async fn send_password_reset_email(to_email: &str, otp: &str) -> Result<(), String> {
    use std::time::Duration;
    
    // Get email credentials from environment variables
    let smtp_username = std::env::var("SMTP_USERNAME")
        .map_err(|_| "SMTP_USERNAME environment variable not set")?;
    let smtp_password = std::env::var("SMTP_PASSWORD")
        .map_err(|_| "SMTP_PASSWORD environment variable not set")?;
    let smtp_host = std::env::var("SMTP_HOST")
        .unwrap_or_else(|_| "smtp.gmail.com".to_string());
    let smtp_port = std::env::var("SMTP_PORT")
        .unwrap_or_else(|_| "587".to_string())
        .parse::<u16>()
        .unwrap_or(587);
    let from_name = std::env::var("SMTP_FROM_NAME")
        .unwrap_or_else(|_| "Clock It".to_string());

    println!("📧 Attempting to send password reset email via {}:{}", smtp_host, smtp_port);

    // Create password reset email message
    let email = Message::builder()
        .from(format!("{} <{}>", from_name, smtp_username).parse()
            .map_err(|e| format!("Invalid from address: {}", e))?)
        .to(to_email.parse()
            .map_err(|e| format!("Invalid to address: {}", e))?)
        .subject("Reset Your Clock It Password")
        .header(ContentType::TEXT_HTML)
        .body(format!(
            r#"
            <div style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
                <div style="text-align: center; margin-bottom: 30px;">
                    <h1 style="color: #2563eb; margin: 0;">Clock It</h1>
                </div>
                
                <div style="background: #f8fafc; border-radius: 8px; padding: 30px; text-align: center;">
                    <h2 style="color: #1f2937; margin-bottom: 20px;">Reset Your Password</h2>
                    <p style="color: #4b5563; margin-bottom: 30px;">
                        Enter this verification code to reset your Clock It password:
                    </p>
                    
                    <div style="background: white; border: 2px solid #e5e7eb; border-radius: 8px; padding: 20px; margin: 20px 0; display: inline-block;">
                        <span style="font-size: 32px; font-weight: bold; color: #dc2626; letter-spacing: 8px;">{}</span>
                    </div>
                    
                    <p style="color: #6b7280; font-size: 14px; margin-top: 20px;">
                        This code will expire in 5 minutes for security reasons.
                    </p>
                </div>
                
                <div style="text-align: center; margin-top: 30px; padding-top: 20px; border-top: 1px solid #e5e7eb;">
                    <p style="color: #9ca3af; font-size: 12px;">
                        If you didn't request this password reset, please ignore this email.
                    </p>
                </div>
            </div>
            "#, otp
        ))
        .map_err(|e| format!("Failed to build email: {}", e))?;

    // Create SMTP transport with timeout and proper configuration
    let creds = Credentials::new(smtp_username, smtp_password);
    
    let mailer = SmtpTransport::relay(&smtp_host)
        .map_err(|e| format!("Failed to create SMTP transport: {}", e))?
        .credentials(creds)
        .timeout(Some(Duration::from_secs(10)))  // 10 second timeout
        .port(smtp_port)  // Use configured port
        .build();

    println!("📤 Sending password reset email to: {}", to_email);
    
    // Send email
    mailer.send(&email)
        .map_err(|e| format!("Failed to send email: {}. Check SMTP credentials and network.", e))?;

    println!("✅ Password reset email sent successfully");
    Ok(())
}

#[server(SendOTP, "/api")]
pub async fn send_otp(email: String) -> Result<BasicResponse, ServerFnError> {
    if email.trim().is_empty() {
        return Ok(BasicResponse {
            success: false,
            message: "Email is required".to_string(),
        });
    }

    let email = email.trim().to_lowercase();
    
    // Generate 6-digit OTP (thread-safe)
    let otp: String = {
        let mut rng = rand::thread_rng();
        (0..6).map(|_| rng.gen_range(0..10).to_string()).collect()
    };
    
    // Store OTP with 5-minute expiry
    let expiry = std::time::SystemTime::now() + std::time::Duration::from_secs(300);
    {
        let mut store = OTP_STORE.lock().unwrap();
        store.insert(email.clone(), (otp.clone(), expiry));
    }
    
    // Send email with OTP
    match send_email_otp(&email, &otp).await {
        Ok(()) => {
            println!("✅ OTP email sent successfully to: {}", email);
            Ok(BasicResponse {
                success: true,
                message: format!("Verification code sent to {}. Check your email.", email),
            })
        }
        Err(e) => {
            println!("❌ Failed to send OTP email to {}: {}", email, e);
            // For development, still log the OTP so you can test
            println!("🔑 OTP for testing: {}", otp);
            Ok(BasicResponse {
                success: false,
                message: format!("Failed to send email: {}. Please try again.", e),
            })
        }
    }
}

#[server(VerifyOTP, "/api")]
pub async fn verify_otp(email: String, otp: String) -> Result<BasicResponse, ServerFnError> {
    if email.trim().is_empty() || otp.trim().is_empty() {
        return Ok(BasicResponse {
            success: false,
            message: "Email and OTP are required".to_string(),
        });
    }

    let email = email.trim().to_lowercase();
    let otp = otp.trim();
    
    let mut store = OTP_STORE.lock().unwrap();
    
    if let Some((stored_otp, expiry)) = store.get(&email) {
        // Check if OTP is expired
        if std::time::SystemTime::now() > *expiry {
            store.remove(&email);
            return Ok(BasicResponse {
                success: false,
                message: "OTP has expired. Please request a new one.".to_string(),
            });
        }
        
        // Check if OTP matches
        if stored_otp == otp {
            store.remove(&email); // Remove used OTP
            return Ok(BasicResponse {
                success: true,
                message: "OTP verified successfully!".to_string(),
            });
        }
    }
    
    Ok(BasicResponse {
        success: false,
        message: "Invalid OTP. Please try again.".to_string(),
    })
}

#[server(SendPasswordResetOTP, "/api")]
pub async fn send_password_reset_otp(email: String) -> Result<BasicResponse, ServerFnError> {
    if email.trim().is_empty() {
        return Ok(BasicResponse {
            success: false,
            message: "Email is required".to_string(),
        });
    }

    let email = email.trim().to_lowercase();
    
    // Check if user exists
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;
    
    let user_exists = sqlx::query("SELECT emailAddress FROM users WHERE emailAddress = ?")
        .bind(&email)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;
    
    if user_exists.is_none() {
        return Ok(BasicResponse {
            success: false,
            message: "No account found with that email address".to_string(),
        });
    }
    
    // Generate 6-digit OTP (thread-safe)
    let otp: String = {
        let mut rng = rand::thread_rng();
        (0..6).map(|_| rng.gen_range(0..10).to_string()).collect()
    };
    
    // Store OTP with 5-minute expiry
    let expiry = std::time::SystemTime::now() + std::time::Duration::from_secs(300);
    {
        let mut store = OTP_STORE.lock().unwrap();
        store.insert(email.clone(), (otp.clone(), expiry));
    }
    
    // Send password reset email
    match send_password_reset_email(&email, &otp).await {
        Ok(()) => {
            println!("✅ Password reset OTP email sent successfully to: {}", email);
            Ok(BasicResponse {
                success: true,
                message: format!("Password reset code sent to {}. Check your email.", email),
            })
        }
        Err(e) => {
            println!("❌ Failed to send password reset OTP email to {}: {}", email, e);
            // For development, still log the OTP so you can test
            println!("🔑 Password reset OTP for testing: {}", otp);
            Ok(BasicResponse {
                success: false,
                message: format!("Failed to send email: {}. Please try again.", e),
            })
        }
    }
}