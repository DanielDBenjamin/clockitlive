#[cfg(feature = "ssr")]
use chrono::Utc;
#[cfg(feature = "ssr")]
use sqlx::SqlitePool;

use serde::{Deserialize, Serialize};

// Module types available for both client and server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub module_code: String,
    pub module_title: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModuleRequest {
    pub module_code: String,
    pub module_title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateModuleRequest {
    pub module_code: String,
    pub module_title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleWithStats {
    pub module_code: String,
    pub module_title: String,
    pub description: Option<String>,
    pub student_count: i32,
    pub class_count: i32,
}

// Server-side implementation
#[cfg(feature = "ssr")]
use sqlx::FromRow;

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, FromRow)]
struct DbModule {
    #[sqlx(rename = "moduleCode")]
    module_code: String,
    #[sqlx(rename = "moduleTitle")]
    module_title: String,
    description: Option<String>,
    created_at: String,
    updated_at: String,
}

#[cfg(feature = "ssr")]
impl From<DbModule> for Module {
    fn from(db: DbModule) -> Self {
        Module {
            module_code: db.module_code,
            module_title: db.module_title,
            description: db.description,
            created_at: db.created_at,
            updated_at: db.updated_at,
        }
    }
}

/// Create a new module
#[cfg(feature = "ssr")]
pub async fn create_module(
    pool: &SqlitePool,
    lecturer_email: &str,
    request: CreateModuleRequest,
) -> Result<Module, String> {
    // Check if module code already exists
    let existing = sqlx::query_as::<_, DbModule>("SELECT * FROM modules WHERE moduleCode = ?")
        .bind(&request.module_code)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    if existing.is_some() {
        return Err("Module with this code already exists".to_string());
    }

    let now = Utc::now().to_rfc3339();

    // Insert module
    sqlx::query(
        r#"
        INSERT INTO modules (moduleCode, moduleTitle, description, created_at, updated_at)
        VALUES (CAST(? AS TEXT), ?, ?, ?, ?)
        "#,
    )
    .bind(&request.module_code)
    .bind(&request.module_title)
    .bind(&request.description)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create module: {}", e))?;

    // Link module to lecturer
    sqlx::query(
        r#"
        INSERT INTO lecturer_module (moduleCode, lecturerEmailAddress, created_at)
        VALUES (CAST(? AS TEXT), ?, ?)
        "#,
    )
    .bind(&request.module_code)
    .bind(lecturer_email)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to link module to lecturer: {}", e))?;

    // Fetch and return the created module
    let module = sqlx::query_as::<_, DbModule>("SELECT * FROM modules WHERE moduleCode = ?")
        .bind(&request.module_code)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to fetch created module: {}", e))?;

    Ok(module.into())
}

/// Get all modules for a lecturer
#[cfg(feature = "ssr")]
pub async fn get_lecturer_modules(
    pool: &SqlitePool,
    lecturer_email: &str,
) -> Result<Vec<Module>, String> {
    let modules = sqlx::query_as::<_, DbModule>(
        r#"
        SELECT CAST(moduleCode AS TEXT) as moduleCode, moduleTitle, description, created_at, updated_at 
        FROM modules m
        INNER JOIN lecturer_module lm ON m.moduleCode = lm.moduleCode
        WHERE lm.lecturerEmailAddress = ?
        ORDER BY m.moduleTitle
        "#,
    )
    .bind(lecturer_email)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    Ok(modules.into_iter().map(|m| m.into()).collect())
}

#[cfg(feature = "ssr")]
pub async fn get_lecturer_modules_with_stats(
    pool: &SqlitePool,
    lecturer_email: &str,
) -> Result<Vec<ModuleWithStats>, String> {
    let modules = sqlx::query_as::<_, (String, String, Option<String>, i32, i32)>(
        r#"
        SELECT 
            CAST(m.moduleCode AS TEXT),
            m.moduleTitle,
            m.description,
            COUNT(DISTINCT c.classID) as class_count,
            COUNT(DISTINCT ms.studentEmailAddress) as student_count
        FROM modules m
        LEFT JOIN lecturer_module lm ON m.moduleCode = lm.moduleCode
        LEFT JOIN module_tutor mt ON m.moduleCode = mt.moduleCode
        LEFT JOIN classes c ON m.moduleCode = c.moduleCode
        LEFT JOIN module_students ms ON m.moduleCode = ms.moduleCode
        WHERE lm.lecturerEmailAddress = ? OR mt.tutorEmailAddress = ?
        GROUP BY m.moduleCode, m.moduleTitle, m.description
        ORDER BY m.moduleTitle
        "#,
    )
    .bind(lecturer_email)
    .bind(lecturer_email)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    Ok(modules
        .into_iter()
        .map(
            |(code, title, desc, class_count, student_count)| ModuleWithStats {
                module_code: code,
                module_title: title,
                description: desc,
                student_count,
                class_count,
            },
        )
        .collect())
}

/// Get a single module by code
#[cfg(feature = "ssr")]
pub async fn get_module(pool: &SqlitePool, module_code: &str) -> Result<Option<Module>, String> {
    let module = sqlx::query_as::<_, DbModule>("SELECT * FROM modules WHERE moduleCode = ?")
        .bind(module_code)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(module.map(|m| m.into()))
}

/// Update a module
#[cfg(feature = "ssr")]
pub async fn update_module(
    pool: &SqlitePool,
    request: UpdateModuleRequest,
) -> Result<Module, String> {
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        UPDATE modules
        SET moduleTitle = ?, description = ?, updated_at = ?
        WHERE moduleCode = ?
        "#,
    )
    .bind(&request.module_title)
    .bind(&request.description)
    .bind(&now)
    .bind(&request.module_code)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update module: {}", e))?;

    let module = sqlx::query_as::<_, DbModule>("SELECT * FROM modules WHERE moduleCode = ?")
        .bind(&request.module_code)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to fetch updated module: {}", e))?;

    Ok(module.into())
}

/// Delete a module
#[cfg(feature = "ssr")]
pub async fn delete_module(pool: &SqlitePool, module_code: &str) -> Result<(), String> {
    // Delete related records first (foreign key constraints)
    sqlx::query("DELETE FROM lecturer_module WHERE moduleCode = ?")
        .bind(module_code)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to delete lecturer-module links: {}", e))?;

    sqlx::query("DELETE FROM module_tutor WHERE moduleCode = ?")
        .bind(module_code)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to delete tutor-module links: {}", e))?;

    // Note: You might want to handle classes and attendance differently
    // For now, we'll delete them as well
    sqlx::query("DELETE FROM attendance WHERE classID IN (SELECT classID FROM classes WHERE moduleCode = ?)")
        .bind(module_code)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to delete attendance records: {}", e))?;

    sqlx::query("DELETE FROM classes WHERE moduleCode = ?")
        .bind(module_code)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to delete classes: {}", e))?;

    // Finally delete the module
    sqlx::query("DELETE FROM modules WHERE moduleCode = ?")
        .bind(module_code)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to delete module: {}", e))?;

    Ok(())
}
