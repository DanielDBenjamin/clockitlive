use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use crate::database::init_db_pool;

// Statistics data structures
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OverallStats {
    pub attendance_rate: f64,
    pub total_students: i64,
    pub total_classes: i64,
    pub absent_today: i64,
    pub avg_class_size: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeeklyTrend {
    pub week: String,
    pub attendance_rate: f64,
    pub class_count: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModuleAbsence {
    pub module_title: String,
    pub absence_rate: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModuleOption {
    pub module_code: String, // Changed from i64 to String
    pub module_title: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClassOption {
    pub class_id: i64,
    pub title: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StudentAttendance {
    pub user_id: i64,
    pub name: String,
    pub surname: String,
    pub email_address: String,
    pub present: i64,
    pub total: i64,
    pub attendance_rate: f64,
}

// Server function to get overall statistics with optional filters
#[server(GetOverallStats, "/api")]
pub async fn get_overall_stats(
    lecturer_email: String,
    module_code: Option<String>,
    class_id: Option<i64>,
) -> Result<OverallStats, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    // Get overall attendance rate
    let attendance_rate: f64 = if let Some(cid) = class_id {
        sqlx::query_scalar(
            r#"
            SELECT 
                COALESCE(
                    CAST(SUM(CASE WHEN status = 'present' THEN 1 ELSE 0 END) AS REAL) * 100.0 / 
                    NULLIF(CAST(COUNT(*) AS REAL), 0),
                    0.0
                )
            FROM attendance
            WHERE classID = ?
            "#,
        )
        .bind(cid)
        .fetch_one(&pool)
        .await
        .unwrap_or(0.0)
    } else if let Some(mc) = &module_code {
        sqlx::query_scalar(
            r#"
            SELECT 
                COALESCE(
                    CAST(SUM(CASE WHEN a.status = 'present' THEN 1 ELSE 0 END) AS REAL) * 100.0 / 
                    NULLIF(CAST(COUNT(*) AS REAL), 0),
                    0.0
                )
            FROM attendance a
            JOIN classes c ON a.classID = c.classID
            WHERE c.moduleCode = ?
            "#,
        )
        .bind(mc)
        .fetch_one(&pool)
        .await
        .unwrap_or(0.0)
    } else {
        sqlx::query_scalar(
            r#"
            SELECT 
                COALESCE(
                    CAST(SUM(CASE WHEN a.status = 'present' THEN 1 ELSE 0 END) AS REAL) * 100.0 / 
                    NULLIF(CAST(COUNT(*) AS REAL), 0),
                    0.0
                )
            FROM attendance a
            JOIN classes c ON a.classID = c.classID
            JOIN lecturer_module lm ON c.moduleCode = lm.moduleCode
            WHERE lm.lecturerEmailAddress = ?
            "#,
        )
        .bind(&lecturer_email)
        .fetch_one(&pool)
        .await
        .unwrap_or(0.0)
    };

    // Get total students, scoped to filter (class/module/lecturer)
    let total_students: i64 = if let Some(cid) = class_id {
        // Count distinct enrolled students for the class's module
        sqlx::query_scalar(
            r#"
            SELECT COUNT(DISTINCT ms.studentEmailAddress)
            FROM classes c
            JOIN module_students ms ON c.moduleCode = ms.moduleCode
            WHERE c.classID = ?
            "#,
        )
        .bind(cid)
        .fetch_one(&pool)
        .await
        .unwrap_or(0)
    } else if let Some(mc) = &module_code {
        // Count distinct students enrolled in this module
        sqlx::query_scalar(
            r#"SELECT COUNT(DISTINCT studentEmailAddress) FROM module_students WHERE moduleCode = ?"#
        )
        .bind(mc)
        .fetch_one(&pool)
        .await
        .unwrap_or(0)
    } else {
        // Count distinct students across all modules taught by this user (lecturer or tutor)
        sqlx::query_scalar(
            r#"
            SELECT COUNT(DISTINCT ms.studentEmailAddress)
            FROM module_students ms
            LEFT JOIN lecturer_module lm ON ms.moduleCode = lm.moduleCode
            LEFT JOIN module_tutor mt ON ms.moduleCode = mt.moduleCode
            WHERE lm.lecturerEmailAddress = ? OR mt.tutorEmailAddress = ?
            "#,
        )
        .bind(&lecturer_email)
        .bind(&lecturer_email)
        .fetch_one(&pool)
        .await
        .unwrap_or(0)
    };

    // Get total classes (filtered by lecturer)
    let total_classes: i64 = if let Some(cid) = class_id {
        sqlx::query_scalar("SELECT COUNT(*) FROM classes WHERE classID = ?")
            .bind(cid)
            .fetch_one(&pool)
            .await
            .unwrap_or(0)
    } else if let Some(mc) = &module_code {
        sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM classes WHERE moduleCode = ?"#,
        )
        .bind(mc)
        .fetch_one(&pool)
        .await
        .unwrap_or(0)
    } else {
        sqlx::query_scalar(
            r#"
            SELECT COUNT(*) 
            FROM classes c
            LEFT JOIN lecturer_module lm ON c.moduleCode = lm.moduleCode
            LEFT JOIN module_tutor mt ON c.moduleCode = mt.moduleCode
            WHERE lm.lecturerEmailAddress = ? OR mt.tutorEmailAddress = ?
            "#,
        )
        .bind(&lecturer_email)
        .bind(&lecturer_email)
        .fetch_one(&pool)
        .await
        .unwrap_or(0)
    };

    // Get absent today (filtered by lecturer)
    let absent_today: i64 = if let Some(cid) = class_id {
        sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM attendance a
            JOIN classes c ON a.classID = c.classID
            WHERE c.date = date('now')
            AND a.status IN ('absent', 'late')
            AND a.classID = ?
            "#,
        )
        .bind(cid)
        .fetch_one(&pool)
        .await
        .unwrap_or(0)
    } else if let Some(mc) = &module_code {
        sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM attendance a
            JOIN classes c ON a.classID = c.classID
            WHERE c.date = date('now')
            AND a.status IN ('absent', 'late')
            AND c.moduleCode = ?
            "#,
        )
        .bind(mc)
        .fetch_one(&pool)
        .await
        .unwrap_or(0)
    } else {
        sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM attendance a
            JOIN classes c ON a.classID = c.classID
            LEFT JOIN lecturer_module lm ON c.moduleCode = lm.moduleCode
            LEFT JOIN module_tutor mt ON c.moduleCode = mt.moduleCode
            WHERE c.date = date('now')
            AND a.status IN ('absent', 'late')
            AND (lm.lecturerEmailAddress = ? OR mt.tutorEmailAddress = ?)
            "#,
        )
        .bind(&lecturer_email)
        .bind(&lecturer_email)
        .fetch_one(&pool)
        .await
        .unwrap_or(0)
    };

    // Get average class size (filtered by lecturer)
    let avg_class_size: f64 = if let Some(cid) = class_id {
        sqlx::query_scalar(
            r#"
            SELECT COALESCE(COUNT(DISTINCT CASE WHEN status = 'present' THEN studentID END), 0)
            FROM attendance
            WHERE classID = ?
            "#,
        )
        .bind(cid)
        .fetch_one(&pool)
        .await
        .map(|count: i64| count as f64)
        .unwrap_or(0.0)
    } else if let Some(mc) = &module_code {
        sqlx::query_scalar(
            r#"
            SELECT COALESCE(AVG(student_count), 0.0)
            FROM (
                SELECT COUNT(DISTINCT CASE WHEN a.status = 'present' THEN a.studentID END) as student_count
                FROM classes c
                LEFT JOIN attendance a ON c.classID = a.classID
                WHERE c.moduleCode = ?
                GROUP BY c.classID
            )
            "#
        )
        .bind(mc)
        .fetch_one(&pool)
        .await
        .unwrap_or(0.0)
    } else {
        sqlx::query_scalar(
            r#"
            SELECT COALESCE(AVG(student_count), 0.0)
            FROM (
                SELECT COUNT(DISTINCT CASE WHEN a.status = 'present' THEN a.studentID END) as student_count
                FROM classes c
                LEFT JOIN lecturer_module lm ON c.moduleCode = lm.moduleCode
                LEFT JOIN module_tutor mt ON c.moduleCode = mt.moduleCode
                LEFT JOIN attendance a ON c.classID = a.classID
                WHERE lm.lecturerEmailAddress = ? OR mt.tutorEmailAddress = ?
                GROUP BY c.classID
            )
            "#
        )
        .bind(&lecturer_email)
        .bind(&lecturer_email)
        .fetch_one(&pool)
        .await
        .unwrap_or(0.0)
    };

    Ok(OverallStats {
        attendance_rate,
        total_students,
        total_classes,
        absent_today,
        avg_class_size,
    })
}

// Server function to get weekly attendance trends
#[server(GetWeeklyTrends, "/api")]
pub async fn get_weekly_trends(
    lecturer_email: String,
    module_code: Option<String>,
    timeframe: Option<String>, // "Weekly" | "Monthly"
    month: Option<String>,     // when Weekly: filter like "YYYY-MM"
) -> Result<Vec<WeeklyTrend>, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let is_monthly = timeframe.as_deref() == Some("Monthly");

    let query: Vec<(String, f64, i64)> = if is_monthly {
        // Monthly trend for current year, up to current month
        if let Some(mc) = &module_code {
            sqlx::query_as(
                r#"
                SELECT strftime('%Y-%m', c.date) as label,
                    COALESCE(
                        CAST(SUM(CASE WHEN a.status = 'present' THEN 1 ELSE 0 END) AS REAL) * 100.0 /
                        NULLIF(CAST(COUNT(*) AS REAL), 0),
                        0.0
                    ) as rate,
                    COUNT(DISTINCT c.classID) as class_cnt
                FROM classes c
                LEFT JOIN attendance a ON c.classID = a.classID
                WHERE c.moduleCode = ?
                  AND strftime('%Y', c.date) = strftime('%Y','now')
                GROUP BY strftime('%Y-%m', c.date)
                HAVING COUNT(a.attendanceID) > 0
                ORDER BY label ASC
                "#
            )
            .bind(mc)
            .fetch_all(&pool)
            .await
            .unwrap_or_default()
        } else {
            sqlx::query_as(
                r#"
                SELECT strftime('%Y-%m', c.date) as label,
                    COALESCE(
                        CAST(SUM(CASE WHEN a.status = 'present' THEN 1 ELSE 0 END) AS REAL) * 100.0 /
                        NULLIF(CAST(COUNT(*) AS REAL), 0),
                        0.0
                    ) as rate,
                    COUNT(DISTINCT c.classID) as class_cnt
                FROM classes c
                LEFT JOIN lecturer_module lm ON c.moduleCode = lm.moduleCode
                LEFT JOIN module_tutor mt ON c.moduleCode = mt.moduleCode
                LEFT JOIN attendance a ON c.classID = a.classID
                WHERE (lm.lecturerEmailAddress = ? OR mt.tutorEmailAddress = ?)
                  AND strftime('%Y', c.date) = strftime('%Y','now')
                GROUP BY strftime('%Y-%m', c.date)
                HAVING COUNT(a.attendanceID) > 0
                ORDER BY label ASC
                "#
            )
            .bind(&lecturer_email)
            .bind(&lecturer_email)
            .fetch_all(&pool)
            .await
            .unwrap_or_default()
        }
    } else {
        // Weekly trend within a selected month (YYYY-MM). Always return W1..W4.
        let month = month.unwrap_or_else(|| chrono::Utc::now().format("%Y-%m").to_string());
        if let Some(mc) = &module_code {
            // aggregate existing weeks, then fill to W1..W5 in Rust
            let rows: Vec<(i64, f64, i64)> = sqlx::query_as(
                r#"
                SELECT (((CAST(strftime('%d', c.date) AS INTEGER) - 1) / 7) + 1) AS w,
                       COALESCE(
                           CAST(SUM(CASE WHEN a.status = 'present' THEN 1 ELSE 0 END) AS REAL) * 100.0 /
                           NULLIF(CAST(COUNT(a.attendanceID) AS REAL), 0),
                           0.0
                       ) AS rate,
                       COUNT(DISTINCT c.classID) AS class_cnt
                FROM classes c
                LEFT JOIN attendance a ON c.classID = a.classID
                WHERE c.moduleCode = ?
                  AND strftime('%Y-%m', c.date) = ?
                GROUP BY w
                ORDER BY w ASC
                "#
            )
            .bind(mc)
            .bind(&month)
            .fetch_all(&pool)
            .await
            .unwrap_or_default();

            // fill in Rust
            let mut by_w = std::collections::HashMap::new();
            for (w, rate, cnt) in rows {
                by_w.insert(w, (rate, cnt));
            }
            (1..=5)
                .map(|w| {
                    let (rate, cnt) = by_w.get(&w).cloned().unwrap_or((0.0, 0));
                    (format!("Week {}", w), rate, cnt)
                })
                .collect()
        } else {
            let rows: Vec<(i64, f64, i64)> = sqlx::query_as(
                r#"
                SELECT (((CAST(strftime('%d', c.date) AS INTEGER) - 1) / 7) + 1) AS w,
                       COALESCE(
                           CAST(SUM(CASE WHEN a.status = 'present' THEN 1 ELSE 0 END) AS REAL) * 100.0 /
                           NULLIF(CAST(COUNT(a.attendanceID) AS REAL), 0),
                           0.0
                       ) AS rate,
                       COUNT(DISTINCT c.classID) AS class_cnt
                FROM classes c
                LEFT JOIN lecturer_module lm ON c.moduleCode = lm.moduleCode
                LEFT JOIN module_tutor mt ON c.moduleCode = mt.moduleCode
                LEFT JOIN attendance a ON c.classID = a.classID
                WHERE (lm.lecturerEmailAddress = ? OR mt.tutorEmailAddress = ?)
                  AND strftime('%Y-%m', c.date) = ?
                GROUP BY w
                ORDER BY w ASC
                "#
            )
            .bind(&lecturer_email)
            .bind(&lecturer_email)
            .bind(&month)
            .fetch_all(&pool)
            .await
            .unwrap_or_default();

            let mut by_w = std::collections::HashMap::new();
            for (w, rate, cnt) in rows {
                by_w.insert(w, (rate, cnt));
            }
            (1..=5)
                .map(|w| {
                    let (rate, cnt) = by_w.get(&w).cloned().unwrap_or((0.0, 0));
                    (format!("Week {}", w), rate, cnt)
                })
                .collect()
        }
    };

    let mut trends: Vec<WeeklyTrend> = query
        .into_iter()
        .map(|(label, rate, class_cnt): (String, f64, i64)| WeeklyTrend {
            week: label,
            attendance_rate: rate,
            class_count: class_cnt,
        })
        .collect();
    // If monthly, fill from Jan..current month even if missing
    if is_monthly {
        use chrono::{Datelike, Utc as CUtc};
        let now = CUtc::now();
        let year = now.year();
        let mut filled: Vec<WeeklyTrend> = Vec::new();
        for m in 1..=now.month() {
            let label = format!("{}-{:02}", year, m);
            if let Some(t) = trends.iter().find(|t| t.week == label) {
                filled.push(t.clone());
            } else {
                filled.push(WeeklyTrend {
                    week: label,
                    attendance_rate: 0.0,
                    class_count: 0,
                });
            }
        }
        return Ok(filled);
    }

    Ok(trends)
}

// Server function to get most missed modules
#[server(GetMostMissedModules, "/api")]
pub async fn get_most_missed_modules(
    lecturer_email: String,
    module_code: Option<String>,
) -> Result<Vec<ModuleAbsence>, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let rows: Vec<(String, f64)> = if let Some(mc) = &module_code {
        sqlx::query_as(
            r#"
            SELECT 
                m.moduleTitle as title,
                COALESCE(
                    CAST(SUM(CASE WHEN a.status IN ('absent', 'late') THEN 1 ELSE 0 END) AS REAL) * 100.0 / 
                    NULLIF(CAST(COUNT(*) AS REAL), 0),
                    0.0
                ) as absence_rate
            FROM modules m
            JOIN classes c ON m.moduleCode = c.moduleCode
            LEFT JOIN lecturer_module lm ON m.moduleCode = lm.moduleCode
            LEFT JOIN module_tutor mt ON m.moduleCode = mt.moduleCode
            LEFT JOIN attendance a ON c.classID = a.classID
            WHERE m.moduleCode = ? AND (lm.lecturerEmailAddress = ? OR mt.tutorEmailAddress = ?)
            GROUP BY m.moduleCode, m.moduleTitle
            HAVING COUNT(a.attendanceID) > 0
            ORDER BY absence_rate DESC
            LIMIT 5
            "#
        )
        .bind(mc)
        .bind(&lecturer_email)
        .bind(&lecturer_email)
        .fetch_all(&pool)
        .await
        .unwrap_or_default()
    } else {
        sqlx::query_as(
            r#"
            SELECT 
                m.moduleTitle as title,
                COALESCE(
                    CAST(SUM(CASE WHEN a.status IN ('absent', 'late') THEN 1 ELSE 0 END) AS REAL) * 100.0 / 
                    NULLIF(CAST(COUNT(*) AS REAL), 0),
                    0.0
                ) as absence_rate
            FROM modules m
            JOIN classes c ON m.moduleCode = c.moduleCode
            LEFT JOIN lecturer_module lm ON m.moduleCode = lm.moduleCode
            LEFT JOIN module_tutor mt ON m.moduleCode = mt.moduleCode
            LEFT JOIN attendance a ON c.classID = a.classID
            WHERE (lm.lecturerEmailAddress = ? OR mt.tutorEmailAddress = ?)
            GROUP BY m.moduleCode, m.moduleTitle
            HAVING COUNT(a.attendanceID) > 0
            ORDER BY absence_rate DESC
            LIMIT 5
            "#
        )
        .bind(&lecturer_email)
        .bind(&lecturer_email)
        .fetch_all(&pool)
        .await
        .unwrap_or_default()
    };

    Ok(rows
        .into_iter()
        .map(|(title, rate)| ModuleAbsence {
            module_title: title,
            absence_rate: rate,
        })
        .collect())
}

// Server function to get module options for dropdown (supports both lecturers and tutors)
#[server(GetModuleOptions, "/api")]
pub async fn get_module_options(
    user_email: String,
) -> Result<Vec<ModuleOption>, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let rows: Vec<(String, String)> = sqlx::query_as(
        r#"
        SELECT DISTINCT m.moduleCode, m.moduleTitle
        FROM modules m
        LEFT JOIN lecturer_module lm ON m.moduleCode = lm.moduleCode
        LEFT JOIN module_tutor mt ON m.moduleCode = mt.moduleCode
        WHERE lm.lecturerEmailAddress = ? OR mt.tutorEmailAddress = ?
        ORDER BY m.moduleTitle
        "#,
    )
    .bind(&user_email)
    .bind(&user_email)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Ok(rows
        .into_iter()
        .map(|(code, title)| ModuleOption {
            module_code: code,
            module_title: title,
        })
        .collect())
}

// Server function to get class options for dropdown
#[server(GetClassOptions, "/api")]
pub async fn get_class_options(
    module_code: Option<String>,
) -> Result<Vec<ClassOption>, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let query = if let Some(mc) = &module_code {
        sqlx::query_as(
            r#"
            SELECT classID, title
            FROM classes
            WHERE status IN ('upcoming', 'in_progress', 'completed')
            AND moduleCode = ?
            ORDER BY date DESC, time DESC
            "#,
        )
        .bind(mc)
        .fetch_all(&pool)
        .await
        .unwrap_or_default()
    } else {
        vec![]
    };

    Ok(query
        .into_iter()
        .map(|(id, title): (i64, String)| ClassOption {
            class_id: id,
            title,
        })
        .collect())
}

// Per-student attendance for a module (optionally for a specific class)
#[server(GetModuleStudentAttendance, "/api")]
pub async fn get_module_student_attendance(
    lecturer_email: String,
    module_code: String,
    class_id: Option<i64>,
) -> Result<Vec<StudentAttendance>, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    // Only allow for modules taught by this lecturer or where they are a tutor
    let teaches: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM lecturer_module WHERE moduleCode = ? AND lecturerEmailAddress = ?) OR EXISTS(SELECT 1 FROM module_tutor WHERE moduleCode = ? AND tutorEmailAddress = ?)"
    )
    .bind(&module_code)
    .bind(&lecturer_email)
    .bind(&module_code)
    .bind(&lecturer_email)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);
    if !teaches {
        return Ok(vec![]);
    }

    // Only students enrolled in module_students for this module
    let rows: Vec<(i64, String, String, String, i64, i64, f64)> = if let Some(cid) = class_id {
        sqlx::query_as(
            r#"
            SELECT u.userID,
                   u.name,
                   u.surname,
                   u.emailAddress,
                   COALESCE(SUM(CASE WHEN a.status = 'present' THEN 1 ELSE 0 END), 0) AS present_cnt,
                   COALESCE(COUNT(a.attendanceID), 0) AS total_cnt,
                   COALESCE(
                     CASE WHEN COUNT(a.attendanceID) = 0 THEN 0.0
                          ELSE (CAST(SUM(CASE WHEN a.status = 'present' THEN 1 ELSE 0 END) AS REAL) * 100.0)
                               / CAST(COUNT(a.attendanceID) AS REAL)
                     END, 0.0
                   ) AS rate
            FROM module_students ms
            JOIN users u ON u.emailAddress = ms.studentEmailAddress
            LEFT JOIN classes c ON c.classID = ? AND c.moduleCode = ms.moduleCode
            LEFT JOIN attendance a ON a.classID = c.classID AND a.studentID = u.userID
            WHERE ms.moduleCode = ?
            GROUP BY u.userID, u.name, u.surname, u.emailAddress
            ORDER BY u.surname, u.name
            "#
        )
        .bind(cid)
        .bind(&module_code)
        .fetch_all(&pool)
        .await
        .unwrap_or_default()
    } else {
        sqlx::query_as(
            r#"
            SELECT u.userID,
                   u.name,
                   u.surname,
                   u.emailAddress,
                   COALESCE(SUM(CASE WHEN a.status = 'present' THEN 1 ELSE 0 END), 0) AS present_cnt,
                   COALESCE(COUNT(a.attendanceID), 0) AS total_cnt,
                   COALESCE(
                     CASE WHEN COUNT(a.attendanceID) = 0 THEN 0.0
                          ELSE (CAST(SUM(CASE WHEN a.status = 'present' THEN 1 ELSE 0 END) AS REAL) * 100.0)
                               / CAST(COUNT(a.attendanceID) AS REAL)
                     END, 0.0
                   ) AS rate
            FROM module_students ms
            JOIN users u ON u.emailAddress = ms.studentEmailAddress
            LEFT JOIN classes c ON c.moduleCode = ms.moduleCode
            LEFT JOIN attendance a ON a.classID = c.classID AND a.studentID = u.userID
            WHERE ms.moduleCode = ?
            GROUP BY u.userID, u.name, u.surname, u.emailAddress
            ORDER BY u.surname, u.name
            "#
        )
        .bind(&module_code)
        .fetch_all(&pool)
        .await
        .unwrap_or_default()
    };

    Ok(rows
        .into_iter()
        .map(
            |(id, name, surname, email, present, total, rate)| StudentAttendance {
                user_id: id,
                name,
                surname,
                email_address: email,
                present,
                total,
                attendance_rate: rate,
            },
        )
        .collect())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StudentClassAttendance {
    pub class_id: i64,
    pub title: String,
    pub date: String,
    pub time: String,
    pub status: String,
}

#[server(GetStudentModuleAttendanceDetail, "/api")]
pub async fn get_student_module_attendance_detail(
    lecturer_email: String,
    module_code: String,
    student_id: i64,
) -> Result<Vec<StudentClassAttendance>, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    // Confirm lecturer teaches module or is a tutor for the module
    let teaches: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM lecturer_module WHERE moduleCode = ? AND lecturerEmailAddress = ?) OR EXISTS(SELECT 1 FROM module_tutor WHERE moduleCode = ? AND tutorEmailAddress = ?)"
    )
    .bind(&module_code)
    .bind(&lecturer_email)
    .bind(&module_code)
    .bind(&lecturer_email)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);
    if !teaches {
        return Ok(vec![]);
    }

    let rows: Vec<(i64, String, String, String, Option<String>)> = sqlx::query_as(
        r#"
        SELECT c.classID, c.title, c.date, c.time,
               a.status
        FROM classes c
        LEFT JOIN attendance a ON a.classID = c.classID AND a.studentID = ?
        WHERE c.moduleCode = ?
        ORDER BY c.date ASC, c.time ASC
        "#,
    )
    .bind(student_id)
    .bind(&module_code)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Ok(rows
        .into_iter()
        .map(
            |(id, title, date, time, status_opt)| StudentClassAttendance {
                class_id: id,
                title,
                date,
                time,
                status: status_opt.unwrap_or_else(|| "absent".to_string()),
            },
        )
        .collect())
}

// Enrollment count for a module (number of students in module_students)
#[server(GetModuleEnrollmentCount, "/api")]
pub async fn get_module_enrollment_count(module_code: String) -> Result<i64, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM module_students WHERE moduleCode = ?")
            .bind(&module_code)
            .fetch_one(&pool)
            .await
            .unwrap_or(0);

    Ok(count)
}

// Export attendance data to CSV format
#[server(ExportAttendanceData, "/api")]
pub async fn export_attendance_data(
    lecturer_email: String,
    module_code: Option<String>,
    timeframe: Option<String>,
    month: Option<String>,
) -> Result<String, ServerFnError> {
    let _pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let mut csv_content = String::new();

    if let Some(mc) = &module_code {
        // Export student attendance data for specific module
        csv_content.push_str("Student Name,Email,Present Classes,Total Classes,Attendance Rate (%)\n");
        
        let students = get_module_student_attendance(lecturer_email.clone(), mc.clone(), None).await?;
        for student in students {
            csv_content.push_str(&format!(
                "\"{} {}\",{},{},{},{:.1}\n",
                student.name,
                student.surname,
                student.email_address,
                student.present,
                student.total,
                student.attendance_rate
            ));
        }
    } else {
        // Export overall statistics and trends
        csv_content.push_str("Type,Period,Attendance Rate (%),Class Count\n");
        
        let trends = get_weekly_trends(lecturer_email.clone(), module_code.clone(), timeframe, month).await?;
        for trend in trends {
            csv_content.push_str(&format!(
                "Trend,{},{:.1},{}\n",
                trend.week,
                trend.attendance_rate,
                trend.class_count
            ));
        }
        
        // Add overall stats
        let stats = get_overall_stats(lecturer_email, module_code, None).await?;
        csv_content.push_str(&format!(
            "Overall,All Time,{:.1},{}\n",
            stats.attendance_rate,
            stats.total_classes
        ));
    }

    Ok(csv_content)
}
