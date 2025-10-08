use crate::database::class_sessions::ClassSession;
use crate::database::classes::{Class, CreateClassRequest, UpdateClassRequest};
use gloo_net::http::Request;
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::database::{
    class_sessions::{create_session, end_session, get_active_session, get_session_by_id},
    classes::{
        create_class, delete_class, get_class_by_id, get_lecturer_classes, get_module_classes,
        update_class,
    },
    init_db_pool,
};
#[cfg(feature = "ssr")]
use chrono::{DateTime, Duration as ChronoDuration, Local, NaiveDate, NaiveTime, Utc};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ClassResponse {
    pub success: bool,
    pub message: String,
    pub class: Option<Class>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ClassesListResponse {
    pub success: bool,
    pub message: String,
    pub classes: Vec<Class>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ClassSessionResponse {
    pub success: bool,
    pub message: String,
    pub session: Option<ClassSession>,
    pub class_status: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RecordAttendanceResponse {
    pub success: bool,
    pub message: String,
}

#[cfg(feature = "ssr")]
async fn ensure_session_state(
    pool: &sqlx::SqlitePool,
    class_id: i64,
) -> Result<Option<ClassSession>, String> {
    let class = get_class_by_id(pool, class_id).await?;
    let date = NaiveDate::parse_from_str(&class.date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid class date: {}", e))?;
    let time = NaiveTime::parse_from_str(&class.time, "%H:%M")
        .map_err(|e| format!("Invalid class time: {}", e))?;
    let start_naive = date.and_time(time);
    let duration_minutes = class.duration_minutes.max(15) as i64;
    let now_naive = Local::now().naive_local();

    let mut session = get_active_session(pool, class_id).await?;
    let expected_end = start_naive + ChronoDuration::minutes(duration_minutes);

    // Check if class has completely passed without being started
    if session.is_none() && now_naive >= expected_end {
        // Class time has completely passed, mark as completed
        let now_utc = Utc::now().to_rfc3339();
        sqlx::query("UPDATE classes SET status = 'completed', updated_at = ? WHERE classID = ?")
            .bind(&now_utc)
            .bind(class_id)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to update class status: {}", e))?;
        return Ok(None);
    }

    // DO NOT auto-start sessions - classes should only be started manually by lecturers
    // Only auto-end sessions when their time has passed

    if let Some(existing) = session.clone() {
        if existing.ended_at.is_none() {
            let started_naive = DateTime::parse_from_rfc3339(&existing.started_at)
                .map(|dt| dt.with_timezone(&Local).naive_local())
                .unwrap_or(start_naive);
            let session_expected_end = started_naive + ChronoDuration::minutes(duration_minutes);
            
            // End session if either the session duration has passed OR the class end time has passed
            if now_naive >= session_expected_end || now_naive >= expected_end {
                end_session(pool, existing.session_id).await?;
                let now_utc = Utc::now().to_rfc3339();
                sqlx::query(
                    "UPDATE classes SET status = 'completed', updated_at = ? WHERE classID = ?",
                )
                .bind(&now_utc)
                .bind(class_id)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to update class status: {}", e))?;
                session = None;
            }
        }
    }

    Ok(session)
}

/// Create a new class (with optional recurring pattern)
#[server(CreateClass, "/api")]
pub async fn create_class_fn(
    module_code: String,
    title: String,
    venue: Option<String>,
    description: Option<String>,
    recurring: Option<String>,
    date: String,
    time: String,
    duration_minutes: i32,
    recurrence_count: Option<i32>, // How many instances to create
) -> Result<ClassResponse, ServerFnError> {
    // Add logging
    println!("Creating class for module: '{}'", module_code);

    if title.trim().is_empty() {
        return Ok(ClassResponse {
            success: false,
            message: "Class title is required".to_string(),
            class: None,
        });
    }

    if date.trim().is_empty() || time.trim().is_empty() {
        return Ok(ClassResponse {
            success: false,
            message: "Date and time are required".to_string(),
            class: None,
        });
    }

    if duration_minutes <= 0 {
        return Ok(ClassResponse {
            success: false,
            message: "Please choose a valid duration".to_string(),
            class: None,
        });
    }

    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    // Verify the module exists
    let module_exists = sqlx::query("SELECT 1 FROM modules WHERE moduleCode = ?")
        .bind(&module_code)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to check module: {}", e)))?;

    if module_exists.is_none() {
        println!("Module '{}' not found in database!", module_code);
        return Ok(ClassResponse {
            success: false,
            message: format!("Module '{}' does not exist", module_code),
            class: None,
        });
    }

    let request = CreateClassRequest {
        module_code: module_code.clone(),
        title: title.trim().to_string(),
        venue: venue.as_ref().and_then(|s| {
            if s.trim().is_empty() {
                None
            } else {
                Some(s.clone())
            }
        }),
        description: description.as_ref().and_then(|s| {
            if s.trim().is_empty() {
                None
            } else {
                Some(s.clone())
            }
        }),
        recurring: recurring
            .clone()
            .filter(|s| !s.trim().is_empty() && s != "No repeat"),
        date: date.clone(),
        time: time.clone(),
        duration_minutes,
    };

    // Create the first class
    let first_class = match create_class(&pool, request).await {
        Ok(class) => class,
        Err(e) => {
            println!("Failed to create class: {}", e);
            return Ok(ClassResponse {
                success: false,
                message: e,
                class: None,
            });
        }
    };

    // If recurring, create additional instances
    if let Some(recur_pattern) = &recurring {
        if recur_pattern != "No repeat" {
            let count = recurrence_count.unwrap_or(8); // Default to 8 weeks if not specified

            // Parse the start date
            if let Ok(start_date) = NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
                let interval = match recur_pattern.as_str() {
                    "Daily" => ChronoDuration::days(1),
                    "Weekly" => ChronoDuration::weeks(1),
                    "Monthly" => ChronoDuration::days(30), // Approximate
                    _ => ChronoDuration::weeks(1),         // Default to weekly
                };

                // Create additional class instances
                for i in 1..count {
                    let next_date = start_date + (interval * (i as i32));
                    let next_date_str = next_date.format("%Y-%m-%d").to_string();

                    let recurring_request = CreateClassRequest {
                        module_code: module_code.clone(),
                        title: title.trim().to_string(),
                        venue: venue.as_ref().and_then(|s| {
                            if s.trim().is_empty() {
                                None
                            } else {
                                Some(s.clone())
                            }
                        }),
                        description: description.as_ref().and_then(|s| {
                            if s.trim().is_empty() {
                                None
                            } else {
                                Some(s.clone())
                            }
                        }),
                        recurring: Some(recur_pattern.clone()),
                        date: next_date_str,
                        time: time.clone(),
                        duration_minutes,
                    };

                    // Create each recurring instance
                    if let Err(e) = create_class(&pool, recurring_request).await {
                        println!("Warning: Failed to create recurring instance {}: {}", i, e);
                        // Continue creating other instances even if one fails
                    }
                }
            }
        }
    }

    let message = if recurring.is_some() && recurring.as_ref().unwrap() != "No repeat" {
        let count = recurrence_count.unwrap_or(8);
        format!("Created {} recurring class instances successfully!", count)
    } else {
        "Class created successfully!".to_string()
    };

    Ok(ClassResponse {
        success: true,
        message,
        class: Some(first_class),
    })
}

/// Get all classes for a module
#[server(GetModuleClasses, "/api")]
pub async fn get_module_classes_fn(
    module_code: String,
) -> Result<ClassesListResponse, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    match get_module_classes(&pool, &module_code).await {
        Ok(classes) => Ok(ClassesListResponse {
            success: true,
            message: "Classes fetched successfully".to_string(),
            classes,
        }),
        Err(e) => Ok(ClassesListResponse {
            success: false,
            message: e,
            classes: vec![],
        }),
    }
}

/// Delete a class
#[server(DeleteClass, "/api")]
pub async fn delete_class_fn(class_id: i64) -> Result<ClassResponse, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    match delete_class(&pool, class_id).await {
        Ok(_) => Ok(ClassResponse {
            success: true,
            message: "Class deleted successfully!".to_string(),
            class: None,
        }),
        Err(e) => Ok(ClassResponse {
            success: false,
            message: e,
            class: None,
        }),
    }
}

/// Get all classes for a lecturer
#[server(GetLecturerClasses, "/api")]
pub async fn get_lecturer_classes_fn(
    lecturer_email: String,
) -> Result<ClassesListResponse, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    match get_lecturer_classes(&pool, &lecturer_email).await {
        Ok(classes) => Ok(ClassesListResponse {
            success: true,
            message: "Classes fetched successfully".to_string(),
            classes,
        }),
        Err(e) => Ok(ClassesListResponse {
            success: false,
            message: e,
            classes: vec![],
        }),
    }
}

/// Get a single class by ID
#[server(GetClass, "/api")]
pub async fn get_class_fn(class_id: i64) -> Result<ClassResponse, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let class = sqlx::query_as::<_, crate::database::classes::DbClass>(
        "SELECT * FROM classes WHERE classID = ?",
    )
    .bind(class_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    match class {
        Some(c) => Ok(ClassResponse {
            success: true,
            message: "Class found".to_string(),
            class: Some(c.into()),
        }),
        None => Ok(ClassResponse {
            success: false,
            message: "Class not found".to_string(),
            class: None,
        }),
    }
}

/// Update a class
#[server(UpdateClassFn, "/api")]
pub async fn update_class_fn(
    class_id: i64,
    title: String,
    description: Option<String>,
    date: String,
    time: String,
    duration_minutes: i32,
    venue: Option<String>,
    recurring: Option<String>,
) -> Result<ClassResponse, ServerFnError> {
    if title.trim().is_empty() {
        return Ok(ClassResponse {
            success: false,
            message: "Class title is required".to_string(),
            class: None,
        });
    }

    if date.trim().is_empty() {
        return Ok(ClassResponse {
            success: false,
            message: "Class date is required".to_string(),
            class: None,
        });
    }

    if duration_minutes <= 0 {
        return Ok(ClassResponse {
            success: false,
            message: "Please choose a valid duration".to_string(),
            class: None,
        });
    }

    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let request = UpdateClassRequest {
        title: title.trim().to_string(),
        description: description.filter(|s| !s.trim().is_empty()),
        date: date.trim().to_string(),
        time: time.trim().to_string(),
        duration_minutes,
        venue: venue.filter(|s| !s.trim().is_empty()),
        recurring: recurring.filter(|s| !s.trim().is_empty()),
    };

    match update_class(&pool, class_id, request).await {
        Ok(class) => Ok(ClassResponse {
            success: true,
            message: "Class updated successfully!".to_string(),
            class: Some(class),
        }),
        Err(e) => Ok(ClassResponse {
            success: false,
            message: e,
            class: None,
        }),
    }
}

/// Update a single class that's part of a series (removes it from the series)
#[server(UpdateSingleInSeries, "/api")]
pub async fn update_single_in_series_fn(
    class_id: i64,
    title: String,
    description: Option<String>,
    date: String,
    time: String,
    duration_minutes: i32,
    venue: Option<String>,
) -> Result<ClassResponse, ServerFnError> {
    if title.trim().is_empty() {
        return Ok(ClassResponse {
            success: false,
            message: "Class title is required".to_string(),
            class: None,
        });
    }

    if date.trim().is_empty() {
        return Ok(ClassResponse {
            success: false,
            message: "Class date is required".to_string(),
            class: None,
        });
    }

    if duration_minutes <= 0 {
        return Ok(ClassResponse {
            success: false,
            message: "Please choose a valid duration".to_string(),
            class: None,
        });
    }

    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    // Update the class and remove it from the series by setting recurring to NULL
    let request = UpdateClassRequest {
        title: title.trim().to_string(),
        description: description.filter(|s| !s.trim().is_empty()),
        date: date.trim().to_string(),
        time: time.trim().to_string(),
        duration_minutes,
        venue: venue.filter(|s| !s.trim().is_empty()),
        recurring: None, // Remove from series
    };

    match update_class(&pool, class_id, request).await {
        Ok(class) => Ok(ClassResponse {
            success: true,
            message: "Class updated successfully and removed from series!".to_string(),
            class: Some(class),
        }),
        Err(e) => Ok(ClassResponse {
            success: false,
            message: e,
            class: None,
        }),
    }
}

/// Update all classes in a recurring series
#[server(UpdateRecurringSeries, "/api")]
pub async fn update_recurring_series_fn(
    module_code: String,
    original_title: String,
    original_recurring: String,
    new_title: String,
    new_description: Option<String>,
    new_venue: Option<String>,
    new_time: String,
) -> Result<ClassResponse, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let now = chrono::Utc::now().to_rfc3339();

    // Update all classes that match the original series
    let result = sqlx::query(
        r#"
        UPDATE classes 
        SET title = ?, description = ?, venue = ?, time = ?, updated_at = ?
        WHERE moduleCode = ? AND title = ? AND recurring = ? AND status = 'upcoming'
        "#,
    )
    .bind(&new_title)
    .bind(&new_description)
    .bind(&new_venue)
    .bind(&new_time)
    .bind(&now)
    .bind(&module_code)
    .bind(&original_title)
    .bind(&original_recurring)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to update series: {}", e)))?;

    let count = result.rows_affected();

    Ok(ClassResponse {
        success: true,
        message: format!("Updated {} classes in the series", count),
        class: None,
    })
}

/// Rewrite a recurring series when the recurrence pattern changes
#[server(RewriteRecurringSeries, "/api")]
pub async fn rewrite_recurring_series_fn(
    class_id: i64,
    module_code: String,
    original_title: String,
    original_recurring: Option<String>,
    new_title: String,
    new_description: Option<String>,
    new_venue: Option<String>,
    new_date: String,
    new_time: String,
    new_duration_minutes: i32,
    new_recurring: Option<String>,
    new_recurrence_count: Option<i32>,
) -> Result<ClassResponse, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    // 1) Find all upcoming classes that belong to the original series
    let series_class_ids: Vec<i64> = if let Some(orig_rec) = &original_recurring {
        sqlx::query_scalar(
            r#"SELECT classID FROM classes
               WHERE moduleCode = ? AND title = ? AND recurring = ? AND status = 'upcoming'"#,
        )
        .bind(&module_code)
        .bind(&original_title)
        .bind(orig_rec)
        .fetch_all(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to query original series: {}", e)))?
    } else {
        sqlx::query_scalar(
            r#"SELECT classID FROM classes
               WHERE moduleCode = ? AND title = ? AND recurring IS NULL AND status = 'upcoming'"#,
        )
        .bind(&module_code)
        .bind(&original_title)
        .fetch_all(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to query original series: {}", e)))?
    };

    let total_in_series = series_class_ids.len();

    // 2) Update the currently edited class to the new details
    let update_request = UpdateClassRequest {
        title: new_title.clone(),
        description: new_description.clone(),
        date: new_date.clone(),
        time: new_time.clone(),
        duration_minutes: new_duration_minutes,
        venue: new_venue.clone(),
        recurring: new_recurring.clone(),
    };

    let updated_class = match update_class(&pool, class_id, update_request).await {
        Ok(class) => class,
        Err(e) => {
            return Ok(ClassResponse {
                success: false,
                message: e,
                class: None,
            })
        }
    };

    // 3) Remove all other upcoming classes from the original series (they will be recreated)
    for id in series_class_ids {
        if id != class_id {
            if let Err(e) = delete_class(&pool, id).await {
                // Continue but report failure in the message later
                leptos::logging::log!(
                    "Failed to delete class {} while rewriting series: {}",
                    id,
                    e
                );
            }
        }
    }

    // 4) Recreate the remaining instances using the new recurrence
    let mut created = 0usize;
    if let Some(recur) = &new_recurring {
        // If user provided a new recurrence count, use it; otherwise keep original series length
        let target_total: usize = new_recurrence_count
            .map(|c| c.max(1) as usize)
            .unwrap_or_else(|| total_in_series.max(1));

        if target_total > 1 {
            if let Ok(start_date) = NaiveDate::parse_from_str(&new_date, "%Y-%m-%d") {
                let interval = match recur.as_str() {
                    "Daily" => ChronoDuration::days(1),
                    "Weekly" => ChronoDuration::weeks(1),
                    "Monthly" => ChronoDuration::days(30),
                    _ => ChronoDuration::weeks(1),
                };

                for i in 1..target_total {
                    let next_date = start_date + (interval * (i as i32));
                    let next_date_str = next_date.format("%Y-%m-%d").to_string();

                    let req = CreateClassRequest {
                        module_code: module_code.clone(),
                        title: new_title.clone(),
                        venue: new_venue.clone(),
                        description: new_description.clone(),
                        recurring: new_recurring.clone(),
                        date: next_date_str,
                        time: new_time.clone(),
                        duration_minutes: new_duration_minutes,
                    };

                    match create_class(&pool, req).await {
                        Ok(_) => created += 1,
                        Err(e) => {
                            leptos::logging::log!(
                                "Warning: failed to create rewritten instance {}: {}",
                                i,
                                e
                            );
                        }
                    }
                }
            }
        }
    }

    let msg = if let Some(r) = &new_recurring {
        let total_instances = created + 1; // include the edited class
        format!(
            "Series updated to '{}' recurrence. Now {} instance(s) in total.",
            r, total_instances
        )
    } else {
        "Series updated to 'No repeat'. Removed future instances.".to_string()
    };

    Ok(ClassResponse {
        success: true,
        message: msg,
        class: Some(updated_class),
    })
}

/// Update class status
#[server(UpdateClassStatus, "/api")]
pub async fn update_class_status_fn(
    class_id: i64,
    status: String,
) -> Result<ClassResponse, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        UPDATE classes 
        SET status = ?, updated_at = ?
        WHERE classID = ?
        "#,
    )
    .bind(&status)
    .bind(&now)
    .bind(class_id)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to update status: {}", e)))?;

    let class = sqlx::query_as::<_, crate::database::classes::DbClass>(
        "SELECT * FROM classes WHERE classID = ?",
    )
    .bind(class_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to fetch updated class: {}", e)))?;

    Ok(ClassResponse {
        success: true,
        message: format!("Class status updated to {}", status),
        class: Some(class.into()),
    })
}

#[server(StartClassSession, "/api")]
pub async fn start_class_session_fn(
    class_id: i64,
    latitude: Option<f64>,
    longitude: Option<f64>,
    accuracy: Option<f64>,
    radius: Option<f64>,
) -> Result<ClassSessionResponse, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let (lat, lng) = match (latitude, longitude) {
        (Some(lat), Some(lng)) => (lat, lng),
        _ => return Ok(ClassSessionResponse {
            success: false,
            message:
                "Unable to determine lecturer location. Please allow location access and try again."
                    .to_string(),
            session: None,
            class_status: None,
        }),
    };

    let accuracy = accuracy.filter(|value| value.is_finite() && *value >= 0.0);
    let radius = radius
        .filter(|value| value.is_finite() && *value > 0.0)
        .unwrap_or(30.0);

    match create_session(
        &pool,
        class_id,
        None,
        Some(lat),
        Some(lng),
        accuracy,
        Some(radius),
    )
    .await
    {
        Ok(session) => {
            let now = Utc::now().to_rfc3339();
            sqlx::query(
                "UPDATE classes SET status = 'in_progress', updated_at = ? WHERE classID = ?",
            )
            .bind(&now)
            .bind(class_id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to update class status: {}", e)))?;

            Ok(ClassSessionResponse {
                success: true,
                message: "Session started".to_string(),
                session: Some(session),
                class_status: Some("in_progress".to_string()),
            })
        }
        Err(e) => Ok(ClassSessionResponse {
            success: false,
            message: e,
            session: None,
            class_status: None,
        }),
    }
}

#[server(EndClassSession, "/api")]
pub async fn end_class_session_fn(session_id: i64) -> Result<ClassSessionResponse, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    // Manual end session should always work, regardless of timing logic
    leptos::logging::log!("Manual end session requested for session_id: {}", session_id);
    
    // First, check if the session exists and its current state
    match get_session_by_id(&pool, session_id).await {
        Ok(Some(session)) => {
            leptos::logging::log!("Session {} found - class_id: {}, ended_at: {:?}", 
                session_id, session.class_id, session.ended_at);
            
            if session.ended_at.is_some() {
                leptos::logging::log!("Session {} is already ended, but updating class status anyway", session_id);
                let now = Utc::now().to_rfc3339();
                let _ = sqlx::query("UPDATE classes SET status = 'completed', updated_at = ? WHERE classID = ?")
                    .bind(&now)
                    .bind(session.class_id)
                    .execute(&pool)
                    .await;
                    
                return Ok(ClassSessionResponse {
                    success: true,
                    message: "Session was already completed".to_string(),
                    session: Some(session),
                    class_status: Some("completed".to_string()),
                });
            }
        }
        Ok(None) => {
            leptos::logging::log!("Session {} not found!", session_id);
            return Ok(ClassSessionResponse {
                success: false,
                message: "Session not found".to_string(),
                session: None,
                class_status: None,
            });
        }
        Err(e) => {
            leptos::logging::log!("Error fetching session {}: {}", session_id, e);
        }
    }

    match end_session(&pool, session_id).await {
        Ok(session) => {
            let now = Utc::now().to_rfc3339();
            
            // Always update class status to completed, even if session was already ended
            sqlx::query(
                "UPDATE classes SET status = 'completed', updated_at = ? WHERE classID = ?",
            )
            .bind(&now)
            .bind(session.class_id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to update class status: {}", e)))?;
            
            leptos::logging::log!("Session {} manually ended for class {}", session_id, session.class_id);

            // Ensure students who didn't scan are marked absent to keep attendance complete.
            let class_info = get_class_by_id(&pool, session.class_id)
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to fetch class info: {}", e)))?;

            let absent_recorded_at = Utc::now().to_rfc3339();
            sqlx::query(
                r#"
                INSERT INTO attendance (studentID, classID, status, recorded_at, notes)
                SELECT u.userID, ?, 'absent', ?, 'Marked absent when session ended'
                FROM users u
                INNER JOIN module_students ms ON ms.studentEmailAddress = u.emailAddress
                WHERE ms.moduleCode = ? AND u.role = 'student'
                  AND NOT EXISTS (
                      SELECT 1 FROM attendance a WHERE a.classID = ? AND a.studentID = u.userID
                  )
                "#,
            )
            .bind(session.class_id)
            .bind(&absent_recorded_at)
            .bind(&class_info.module_code)
            .bind(session.class_id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to mark absentees: {}", e)))?;

            Ok(ClassSessionResponse {
                success: true,
                message: "Session ended manually".to_string(),
                session: Some(session),
                class_status: Some("completed".to_string()),
            })
        }
        Err(e) => {
            leptos::logging::log!("Error ending session {}: {}", session_id, e);
            
            // Even if we can't end the session properly, try to get session info for response
            match get_session_by_id(&pool, session_id).await {
                Ok(Some(session)) => {
                    // Session exists but couldn't be ended - might already be ended
                    let now = Utc::now().to_rfc3339();
                    let _ = sqlx::query("UPDATE classes SET status = 'completed', updated_at = ? WHERE classID = ?")
                        .bind(&now)
                        .bind(session.class_id)
                        .execute(&pool)
                        .await;
                    
                    Ok(ClassSessionResponse {
                        success: true,
                        message: "Session was already ended or completed".to_string(),
                        session: Some(session),
                        class_status: Some("completed".to_string()),
                    })
                }
                _ => Ok(ClassSessionResponse {
                    success: false,
                    message: format!("Failed to end session: {}", e),
                    session: None,
                    class_status: None,
                }),
            }
        },
    }
}

#[server(GetActiveClassSession, "/api")]
pub async fn get_active_class_session_fn(
    class_id: i64,
) -> Result<ClassSessionResponse, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let session = ensure_session_state(&pool, class_id)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    let class_status = get_class_by_id(&pool, class_id)
        .await
        .map(|c| c.status)
        .unwrap_or_else(|_| "upcoming".to_string());

    match session {
        Some(session) => Ok(ClassSessionResponse {
            success: true,
            message: "Active session found".to_string(),
            session: Some(session),
            class_status: Some(class_status),
        }),
        None => Ok(ClassSessionResponse {
            success: false,
            message: "No active session".to_string(),
            session: None,
            class_status: Some(class_status),
        }),
    }
}

#[server(CheckAllActiveSessions, "/api")]
pub async fn check_all_active_sessions_fn() -> Result<String, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;
    
    // Get all classes that might need status updates (in_progress OR upcoming classes that might have passed)
    let classes: Vec<(i64, String)> = sqlx::query_as("SELECT classID, status FROM classes WHERE status IN ('in_progress', 'upcoming')")
        .fetch_all(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch classes: {}", e)))?;
    
    let total_classes = classes.len();
    let mut updated_count = 0;
    for (class_id, old_status) in classes {
        if let Err(e) = ensure_session_state(&pool, class_id).await {
            leptos::logging::log!("Error checking session for class {}: {}", class_id, e);
        } else {
            // Check if the class status changed
            let new_status: Option<String> = sqlx::query_scalar("SELECT status FROM classes WHERE classID = ?")
                .bind(class_id)
                .fetch_optional(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to check class status: {}", e)))?;
            
            if let Some(new_status) = new_status {
                if new_status != old_status {
                    updated_count += 1;
                    leptos::logging::log!("Class {} status changed from {} to {}", class_id, old_status, new_status);
                }
            }
        }
    }
    
    Ok(format!("Checked {} classes, updated {} statuses", total_classes, updated_count))
}

// Helper function to save a single instance
pub async fn save_single_instance(
    class_id: i32,
    class_data: web_sys::FormData,
) -> Result<(), ServerFnError> {
    let url = format!("/api/classes/{}", class_id);

    let response = Request::put(&url)
        .body(class_data)?
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Request failed: {}", e)))?;

    if response.ok() {
        Ok(())
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(ServerFnError::new(format!(
            "Failed to update class: {}",
            error_text
        )))
    }
}

// Helper function to save all future instances in a series
pub async fn save_recurring_series(
    series_id: String,
    class_date: String,
    class_data: web_sys::FormData,
) -> Result<(), ServerFnError> {
    let url = format!("/api/classes/recurring/{}/from/{}", series_id, class_date);

    let response = Request::put(&url)
        .body(class_data)?
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Request failed: {}", e)))?;

    if response.ok() {
        Ok(())
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(ServerFnError::new(format!(
            "Failed to update recurring classes: {}",
            error_text
        )))
    }
}

#[server(RecordSessionAttendance, "/api")]
pub async fn record_session_attendance_fn(
    payload: String,
    student_email: String,
    latitude: Option<f64>,
    longitude: Option<f64>,
    accuracy: Option<f64>,
) -> Result<RecordAttendanceResponse, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let parts: Vec<&str> = payload.split(':').collect();
    if parts.len() != 4 || parts[0] != "session" || parts[2] != "class" {
        return Ok(RecordAttendanceResponse {
            success: false,
            message: "Invalid QR code".to_string(),
        });
    }

    let session_id: i64 = parts[1]
        .parse()
        .map_err(|_| ServerFnError::new("Invalid session id"))?;
    let class_id: i64 = parts[3]
        .parse()
        .map_err(|_| ServerFnError::new("Invalid class id"))?;

    let _ = ensure_session_state(&pool, class_id)
        .await
        .map_err(|e| ServerFnError::new(e))?;

    let session = match get_session_by_id(&pool, session_id).await {
        Ok(Some(session)) => session,
        Ok(None) => {
            return Ok(RecordAttendanceResponse {
                success: false,
                message: "Session not found".to_string(),
            })
        }
        Err(e) => {
            return Ok(RecordAttendanceResponse {
                success: false,
                message: e,
            })
        }
    };

    if session.ended_at.is_some() {
        return Ok(RecordAttendanceResponse {
            success: false,
            message: "Session has ended".to_string(),
        });
    }

    if session.class_id != class_id {
        return Ok(RecordAttendanceResponse {
            success: false,
            message: "Session mismatch".to_string(),
        });
    }

    let student_latitude = latitude.filter(|value| value.is_finite());
    let student_longitude = longitude.filter(|value| value.is_finite());
    let student_accuracy = accuracy.filter(|value| value.is_finite() && *value >= 0.0);

    if let (Some(lect_lat), Some(lect_lng), Some(radius)) = (
        session.start_latitude,
        session.start_longitude,
        session.location_radius,
    ) {
        let (Some(student_lat), Some(student_lng)) = (student_latitude, student_longitude) else {
            return Ok(RecordAttendanceResponse {
                success: false,
                message: "Location permission is required to record attendance for this session."
                    .to_string(),
            });
        };

        let lecturer_accuracy = session.start_accuracy.unwrap_or(0.0).max(0.0);
        let student_accuracy_value = student_accuracy.unwrap_or(0.0);
        let distance = haversine_distance(lect_lat, lect_lng, student_lat, student_lng);
        let allowed_distance = radius + lecturer_accuracy + student_accuracy_value;

        if distance > allowed_distance {
            return Ok(RecordAttendanceResponse {
                success: false,
                message: format!(
                    "You are too far from the lecturer ({:.0}m away, must be within {:.0}m)",
                    distance, allowed_distance
                ),
            });
        }
    }

    if student_email.trim().is_empty() {
        return Ok(RecordAttendanceResponse {
            success: false,
            message: "Missing student email".to_string(),
        });
    }

    let student_id: Option<i64> =
        sqlx::query_scalar("SELECT userID FROM users WHERE emailAddress = ? AND role = 'student'")
            .bind(&student_email)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to lookup student: {}", e)))?;

    let student_id = match student_id {
        Some(id) => id,
        None => {
            return Ok(RecordAttendanceResponse {
                success: false,
                message: "Student not found".to_string(),
            })
        }
    };

    let now = Utc::now().to_rfc3339();

    let existing: Option<i64> = sqlx::query_scalar(
        "SELECT attendanceID FROM attendance WHERE classID = ? AND studentID = ?",
    )
    .bind(class_id)
    .bind(student_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to check attendance: {}", e)))?;

    if let Some(attendance_id) = existing {
        sqlx::query(
            "UPDATE attendance SET status = 'present', recorded_at = ?, notes = NULL, check_latitude = ?, check_longitude = ?, location_accuracy = ? WHERE attendanceID = ?"
        )
        .bind(&now)
        .bind(student_latitude)
        .bind(student_longitude)
        .bind(student_accuracy)
        .bind(attendance_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update attendance: {}", e)))?;
    } else {
        sqlx::query(
            "INSERT INTO attendance (studentID, classID, status, recorded_at, notes, check_latitude, check_longitude, location_accuracy) VALUES (?, ?, 'present', ?, NULL, ?, ?, ?)"
        )
        .bind(student_id)
        .bind(class_id)
        .bind(&now)
        .bind(student_latitude)
        .bind(student_longitude)
        .bind(student_accuracy)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to insert attendance: {}", e)))?;
    }

    Ok(RecordAttendanceResponse {
        success: true,
        message: "Attendance recorded".to_string(),
    })
}

#[cfg(feature = "ssr")]
fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let earth_radius_m = 6_371_000.0_f64;
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();

    let a =
        (d_lat / 2.0).sin().powi(2) + lat1_rad.cos() * lat2_rad.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    earth_radius_m * c
}
