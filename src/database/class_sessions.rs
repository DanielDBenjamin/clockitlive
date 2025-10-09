use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use chrono::Utc;
#[cfg(feature = "ssr")]
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassSession {
    pub session_id: i64,
    pub class_id: i64,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub started_by: Option<String>,
    pub start_latitude: Option<f64>,
    pub start_longitude: Option<f64>,
    pub start_accuracy: Option<f64>,
    pub location_radius: Option<f64>,
}

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, FromRow)]
struct DbClassSession {
    #[sqlx(rename = "sessionID")]
    session_id: i64,
    #[sqlx(rename = "classID")]
    class_id: i64,
    started_at: String,
    ended_at: Option<String>,
    started_by: Option<String>,
    start_latitude: Option<f64>,
    start_longitude: Option<f64>,
    start_accuracy: Option<f64>,
    location_radius: Option<f64>,
}

#[cfg(feature = "ssr")]
impl From<DbClassSession> for ClassSession {
    fn from(db: DbClassSession) -> Self {
        Self {
            session_id: db.session_id,
            class_id: db.class_id,
            started_at: db.started_at,
            ended_at: db.ended_at,
            started_by: db.started_by,
            start_latitude: db.start_latitude,
            start_longitude: db.start_longitude,
            start_accuracy: db.start_accuracy,
            location_radius: db.location_radius,
        }
    }
}

#[cfg(feature = "ssr")]
pub async fn get_active_session(
    pool: &SqlitePool,
    class_id: i64,
) -> Result<Option<ClassSession>, String> {
    let session = sqlx::query_as::<_, DbClassSession>(
        "SELECT * FROM class_sessions WHERE classID = ? AND ended_at IS NULL ORDER BY started_at DESC LIMIT 1"
    )
    .bind(class_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to fetch active session: {}", e))?;

    Ok(session.map(Into::into))
}

#[cfg(feature = "ssr")]
pub async fn create_session(
    pool: &SqlitePool,
    class_id: i64,
    started_by: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    accuracy: Option<f64>,
    radius: Option<f64>,
) -> Result<ClassSession, String> {
    if get_active_session(pool, class_id).await?.is_some() {
        return Err("A session is already active for this class".to_string());
    }

    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "INSERT INTO class_sessions (classID, started_at, started_by, start_latitude, start_longitude, start_accuracy, location_radius) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(class_id)
    .bind(&now)
    .bind(&started_by)
    .bind(latitude)
    .bind(longitude)
    .bind(accuracy)
    .bind(radius)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create session: {}", e))?;

    let session_id = result.last_insert_rowid();
    let session =
        sqlx::query_as::<_, DbClassSession>("SELECT * FROM class_sessions WHERE sessionID = ?")
            .bind(session_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Failed to load created session: {}", e))?;

    Ok(session.into())
}

#[cfg(feature = "ssr")]
pub async fn end_session(pool: &SqlitePool, session_id: i64) -> Result<ClassSession, String> {
    let now = Utc::now().to_rfc3339();

    // First, get the session to find the class_id
    let session =
        sqlx::query_as::<_, DbClassSession>("SELECT * FROM class_sessions WHERE sessionID = ?")
            .bind(session_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Failed to fetch session: {}", e))?;

    // End the session
    sqlx::query("UPDATE class_sessions SET ended_at = ? WHERE sessionID = ? AND ended_at IS NULL")
        .bind(&now)
        .bind(session_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to end session: {}", e))?;

    // Get the module code for this class
    let class_module: (String,) = sqlx::query_as("SELECT moduleCode FROM classes WHERE classID = ?")
        .bind(session.class_id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to fetch class module: {}", e))?;

    // Mark all students who didn't attend as absent
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
    .bind(&now)
    .bind(&class_module.0)
    .bind(session.class_id)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to mark absentees: {}", e))?;

    // Fetch the updated session
    let updated_session =
        sqlx::query_as::<_, DbClassSession>("SELECT * FROM class_sessions WHERE sessionID = ?")
            .bind(session_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Failed to fetch ended session: {}", e))?;

    Ok(updated_session.into())
}

#[cfg(feature = "ssr")]
pub async fn get_session_by_id(
    pool: &SqlitePool,
    session_id: i64,
) -> Result<Option<ClassSession>, String> {
    let session =
        sqlx::query_as::<_, DbClassSession>("SELECT * FROM class_sessions WHERE sessionID = ?")
            .bind(session_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("Failed to fetch session: {}", e))?;

    Ok(session.map(Into::into))
}
