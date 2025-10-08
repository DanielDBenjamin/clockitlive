use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use crate::database::init_db_pool;
#[cfg(feature = "ssr")]
use chrono::{Datelike, Duration as ChronoDuration, Local, NaiveDate, Utc};

// Student enrollment data structures
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnrollStudentRequest {
    pub student_email: String,
    pub module_code: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StudentInfo {
    pub user_id: i64,
    pub name: String,
    pub surname: String,
    pub email_address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnrollmentResponse {
    pub success: bool,
    pub message: String,
    pub student: Option<StudentInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StudentsListResponse {
    pub success: bool,
    pub message: String,
    pub students: Vec<StudentInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StudentStatsSummary {
    pub overall_attendance_rate: f64,
    pub weekly_attendance_rate: f64,
    pub total_present: i64,
    pub total_recorded: i64,
    pub upcoming_classes: i64,
    pub week_present: i64,
    pub week_recorded: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StudentWeeklyAttendancePoint {
    pub date: String,
    pub present: i64,
    pub recorded: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StudentModuleBreakdown {
    pub module_code: String,
    pub module_title: String,
    pub present: i64,
    pub recorded: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StudentRecentActivity {
    pub class_id: i64,
    pub title: String,
    pub module_code: String,
    pub date: String,
    pub time: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StudentScheduleItem {
    pub class_id: i64,
    pub module_code: String,
    pub module_title: String,
    pub class_title: String,
    pub venue: Option<String>,
    pub date: String,
    pub time: String,
    pub status: String,
    pub venue_updated_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StudentScheduleResponse {
    pub success: bool,
    pub message: String,
    pub classes: Vec<StudentScheduleItem>,
}

#[cfg(feature = "ssr")]
#[derive(sqlx::FromRow)]
struct DbStudentScheduleRow {
    #[sqlx(rename = "class_id")]
    class_id: i64,
    #[sqlx(rename = "module_code")]
    module_code: String,
    #[sqlx(rename = "module_title")]
    module_title: String,
    #[sqlx(rename = "class_title")]
    class_title: String,
    venue: Option<String>,
    #[sqlx(rename = "class_date")]
    class_date: String,
    #[sqlx(rename = "class_time")]
    class_time: String,
    status: String,
    venue_updated_at: Option<String>,
}

// Enroll a single student in a module
#[server(EnrollStudent, "/api")]
pub async fn enroll_student(
    request: EnrollStudentRequest,
) -> Result<EnrollmentResponse, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    // Check if student exists
    let student = sqlx::query_as::<_, (i64, String, String, String)>(
        "SELECT userID, name, surname, emailAddress FROM users WHERE emailAddress = ? AND role = 'student'"
    )
    .bind(&request.student_email)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    let student = match student {
        Some(s) => s,
        None => {
            return Ok(EnrollmentResponse {
                success: false,
                message: "Student not found with this email address".to_string(),
                student: None,
            })
        }
    };

    // Check if module exists
    let module_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM modules WHERE moduleCode = ?)")
            .bind(&request.module_code)
            .fetch_one(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    if !module_exists {
        return Ok(EnrollmentResponse {
            success: false,
            message: "Module not found".to_string(),
            student: None,
        });
    }

    // Check if already enrolled
    let already_enrolled = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM module_students WHERE moduleCode = ? AND studentEmailAddress = ?)"
    )
    .bind(&request.module_code)
    .bind(&request.student_email)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    if already_enrolled {
        return Ok(EnrollmentResponse {
            success: false,
            message: "Student is already enrolled in this module".to_string(),
            student: None,
        });
    }

    // Enroll student
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO module_students (moduleCode, studentEmailAddress, created_at) VALUES (?, ?, ?)"
    )
    .bind(&request.module_code)
    .bind(&request.student_email)
    .bind(&now)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to enroll student: {}", e)))?;

    Ok(EnrollmentResponse {
        success: true,
        message: "Student enrolled successfully!".to_string(),
        student: Some(StudentInfo {
            user_id: student.0,
            name: student.1,
            surname: student.2,
            email_address: student.3,
        }),
    })
}

// Get all students enrolled in a module
#[server(GetModuleStudents, "/api")]
pub async fn get_module_students(
    module_code: String,
) -> Result<StudentsListResponse, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let students = sqlx::query_as::<_, (i64, String, String, String)>(
        r#"
        SELECT u.userID, u.name, u.surname, u.emailAddress
        FROM users u
        INNER JOIN module_students ms ON u.emailAddress = ms.studentEmailAddress
        WHERE ms.moduleCode = ?
        ORDER BY u.surname, u.name
        "#,
    )
    .bind(&module_code)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    Ok(StudentsListResponse {
        success: true,
        message: "Students fetched successfully".to_string(),
        students: students
            .into_iter()
            .map(|(id, name, surname, email)| StudentInfo {
                user_id: id,
                name,
                surname,
                email_address: email,
            })
            .collect(),
    })
}

// Remove a student from a module
#[server(UnenrollStudent, "/api")]
pub async fn unenroll_student(
    module_code: String,
    student_email: String,
) -> Result<EnrollmentResponse, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let result =
        sqlx::query("DELETE FROM module_students WHERE moduleCode = ? AND studentEmailAddress = ?")
            .bind(&module_code)
            .bind(&student_email)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to unenroll student: {}", e)))?;

    if result.rows_affected() == 0 {
        return Ok(EnrollmentResponse {
            success: false,
            message: "Student was not enrolled in this module".to_string(),
            student: None,
        });
    }

    Ok(EnrollmentResponse {
        success: true,
        message: "Student removed successfully!".to_string(),
        student: None,
    })
}

// Bulk enroll students from CSV data
#[server(BulkEnrollStudents, "/api")]
pub async fn bulk_enroll_students(
    module_code: String,
    student_emails: Vec<String>,
) -> Result<EnrollmentResponse, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let now = Utc::now().to_rfc3339();
    let mut enrolled_count = 0;
    let mut errors = Vec::new();

    for email in student_emails {
        let email = email.trim().to_lowercase();
        if email.is_empty() {
            continue;
        }

        // Check if student exists
        let student_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM users WHERE emailAddress = ? AND role = 'student')",
        )
        .bind(&email)
        .fetch_one(&pool)
        .await
        .unwrap_or(false);

        if !student_exists {
            errors.push(format!("{} (not found)", email));
            continue;
        }

        // Check if already enrolled
        let already_enrolled = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM module_students WHERE moduleCode = ? AND studentEmailAddress = ?)"
        )
        .bind(&module_code)
        .bind(&email)
        .fetch_one(&pool)
        .await
        .unwrap_or(false);

        if already_enrolled {
            continue; // Skip already enrolled students silently
        }

        // Enroll student
        match sqlx::query(
            "INSERT INTO module_students (moduleCode, studentEmailAddress, created_at) VALUES (?, ?, ?)"
        )
        .bind(&module_code)
        .bind(&email)
        .bind(&now)
        .execute(&pool)
        .await {
            Ok(_) => enrolled_count += 1,
            Err(_) => errors.push(format!("{} (enrollment failed)", email)),
        }
    }

    let message = if errors.is_empty() {
        format!("Successfully enrolled {} student(s)", enrolled_count)
    } else {
        format!(
            "Enrolled {} student(s). Errors: {}",
            enrolled_count,
            errors.join(", ")
        )
    };

    Ok(EnrollmentResponse {
        success: enrolled_count > 0,
        message,
        student: None,
    })
}

// Get upcoming classes for a student (default: today)
#[server(GetStudentSchedule, "/api")]
pub async fn get_student_schedule(
    student_email: String,
    date: Option<String>,
) -> Result<StudentScheduleResponse, ServerFnError> {
    let trimmed_email = student_email.trim().to_lowercase();
    if trimmed_email.is_empty() {
        return Ok(StudentScheduleResponse {
            success: false,
            message: "Student email is required".to_string(),
            classes: Vec::new(),
        });
    }

    let selected_date = date
        .map(|d| d.trim().to_string())
        .filter(|d| !d.is_empty())
        .unwrap_or_else(|| {
            #[cfg(feature = "ssr")]
            {
                Local::now().naive_local().format("%Y-%m-%d").to_string()
            }

            #[cfg(not(feature = "ssr"))]
            {
                // Fallback for non-SSR builds (should not happen for server functions)
                String::new()
            }
        });

    if selected_date.is_empty() {
        return Ok(StudentScheduleResponse {
            success: false,
            message: "Could not determine the requested date".to_string(),
            classes: Vec::new(),
        });
    }

    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

let rows = sqlx::query_as::<_, DbStudentScheduleRow>(
    r#"
    SELECT
        c.classID AS class_id,
        CAST(c.moduleCode AS TEXT) AS module_code,
        m.moduleTitle AS module_title,
        c.title AS class_title,
        c.venue AS venue,
        c.date AS class_date,
        c.time AS class_time,
        c.status AS status,
        c.venue_updated_at AS venue_updated_at
    FROM classes c
    INNER JOIN module_students ms ON ms.moduleCode = c.moduleCode
    INNER JOIN modules m ON m.moduleCode = c.moduleCode
    WHERE ms.studentEmailAddress = ?
      AND c.date >= ?
    ORDER BY c.date ASC, c.time ASC
    LIMIT 10
    "#,
)
    .bind(&trimmed_email)
    .bind(&selected_date)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to fetch student schedule: {}", e)))?;

    let classes = rows
        .into_iter()
        .map(|row| StudentScheduleItem {
            class_id: row.class_id,
            module_code: row.module_code,
            module_title: row.module_title,
            class_title: row.class_title,
            venue: row.venue,
            date: row.class_date,
            time: row.class_time,
            status: row.status,
            venue_updated_at: row.venue_updated_at,
        })
        .collect::<Vec<_>>();

    Ok(StudentScheduleResponse {
        success: true,
        message: if classes.is_empty() {
            "No upcoming classes found for the selected date".to_string()
        } else {
            "Schedule loaded".to_string()
        },
        classes,
    })
}

#[server(GetStudentStatsSummary, "/api")]
pub async fn get_student_stats_summary(
    student_email: String,
) -> Result<StudentStatsSummary, ServerFnError> {
    let normalized_email = student_email.trim().to_lowercase();
    if normalized_email.is_empty() {
        return Ok(StudentStatsSummary {
            overall_attendance_rate: 0.0,
            weekly_attendance_rate: 0.0,
            total_present: 0,
            total_recorded: 0,
            upcoming_classes: 0,
            week_present: 0,
            week_recorded: 0,
        });
    }

    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let student_id: Option<i64> =
        sqlx::query_scalar("SELECT userID FROM users WHERE LOWER(emailAddress) = ?")
            .bind(&normalized_email)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to lookup user: {}", e)))?;

    let Some(student_id) = student_id else {
        return Ok(StudentStatsSummary {
            overall_attendance_rate: 0.0,
            weekly_attendance_rate: 0.0,
            total_present: 0,
            total_recorded: 0,
            upcoming_classes: 0,
            week_present: 0,
            week_recorded: 0,
        });
    };

    let today: NaiveDate = Local::now().naive_local().date();
    let today_str = today.format("%Y-%m-%d").to_string();

    let (total_present, total_recorded): (i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COALESCE(SUM(CASE WHEN a.status = 'present' THEN 1 ELSE 0 END), 0) AS present_cnt,
            COALESCE(COUNT(a.attendanceID), 0) AS recorded_cnt
        FROM classes c
        INNER JOIN module_students ms ON ms.moduleCode = c.moduleCode
        LEFT JOIN attendance a ON a.classID = c.classID AND a.studentID = ?
        WHERE ms.studentEmailAddress = ?
          AND c.date <= ?
        "#,
    )
    .bind(student_id)
    .bind(&normalized_email)
    .bind(&today_str)
    .fetch_one(&pool)
    .await
    .unwrap_or((0, 0));

    let start_of_week = {
        let weekday_offset = today.weekday().num_days_from_monday() as i64;
        today - ChronoDuration::days(weekday_offset)
    };
    let end_of_week = start_of_week + ChronoDuration::days(6);

    let (week_present, week_recorded): (i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COALESCE(SUM(CASE WHEN a.status = 'present' THEN 1 ELSE 0 END), 0) AS present_cnt,
            COALESCE(COUNT(a.attendanceID), 0) AS recorded_cnt
        FROM classes c
        INNER JOIN module_students ms ON ms.moduleCode = c.moduleCode
        LEFT JOIN attendance a ON a.classID = c.classID AND a.studentID = ?
        WHERE ms.studentEmailAddress = ?
          AND c.date BETWEEN ? AND ?
        "#,
    )
    .bind(student_id)
    .bind(&normalized_email)
    .bind(start_of_week.format("%Y-%m-%d").to_string())
    .bind(end_of_week.format("%Y-%m-%d").to_string())
    .fetch_one(&pool)
    .await
    .unwrap_or((0, 0));

    let upcoming_classes: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM classes c
        INNER JOIN module_students ms ON ms.moduleCode = c.moduleCode
        WHERE ms.studentEmailAddress = ?
          AND c.date > ?
        "#,
    )
    .bind(&normalized_email)
    .bind(&today_str)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    let overall_rate = if total_recorded > 0 {
        (total_present as f64) * 100.0 / (total_recorded as f64)
    } else {
        0.0
    };
    let weekly_rate = if week_recorded > 0 {
        (week_present as f64) * 100.0 / (week_recorded as f64)
    } else {
        0.0
    };

    Ok(StudentStatsSummary {
        overall_attendance_rate: overall_rate,
        weekly_attendance_rate: weekly_rate,
        total_present,
        total_recorded,
        upcoming_classes,
        week_present,
        week_recorded,
    })
}

#[server(GetStudentWeeklyAttendance, "/api")]
pub async fn get_student_weekly_attendance(
    student_email: String,
) -> Result<Vec<StudentWeeklyAttendancePoint>, ServerFnError> {
    let normalized_email = student_email.trim().to_lowercase();
    if normalized_email.is_empty() {
        return Ok(vec![]);
    }

    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let student_id: Option<i64> =
        sqlx::query_scalar("SELECT userID FROM users WHERE LOWER(emailAddress) = ?")
            .bind(&normalized_email)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to lookup user: {}", e)))?;

    let Some(student_id) = student_id else {
        return Ok(vec![]);
    };

    let today: NaiveDate = Local::now().naive_local().date();
    let start_of_week = {
        let weekday_offset = today.weekday().num_days_from_monday() as i64;
        today - ChronoDuration::days(weekday_offset)
    };
    let end_of_week = start_of_week + ChronoDuration::days(6);

    let rows: Vec<(String, i64, i64)> = sqlx::query_as(
        r#"
        SELECT
            c.date,
            COALESCE(SUM(CASE WHEN a.status = 'present' THEN 1 ELSE 0 END), 0) AS present_cnt,
            COALESCE(COUNT(a.attendanceID), 0) AS recorded_cnt
        FROM classes c
        INNER JOIN module_students ms ON ms.moduleCode = c.moduleCode
        LEFT JOIN attendance a ON a.classID = c.classID AND a.studentID = ?
        WHERE ms.studentEmailAddress = ?
          AND c.date BETWEEN ? AND ?
        GROUP BY c.date
        ORDER BY c.date ASC
        "#,
    )
    .bind(student_id)
    .bind(&normalized_email)
    .bind(start_of_week.format("%Y-%m-%d").to_string())
    .bind(end_of_week.format("%Y-%m-%d").to_string())
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let mut result = Vec::new();
    let mut current = start_of_week;
    while current <= end_of_week {
        let date_key = current.format("%Y-%m-%d").to_string();
        if let Some((_, present, recorded)) =
            rows.iter().find(|(date, _, _)| date == &date_key).cloned()
        {
            result.push(StudentWeeklyAttendancePoint {
                date: date_key,
                present,
                recorded,
            });
        } else {
            result.push(StudentWeeklyAttendancePoint {
                date: date_key,
                present: 0,
                recorded: 0,
            });
        }
        current = current + ChronoDuration::days(1);
    }

    Ok(result)
}

#[server(GetStudentModuleBreakdown, "/api")]
pub async fn get_student_module_breakdown(
    student_email: String,
) -> Result<Vec<StudentModuleBreakdown>, ServerFnError> {
    let normalized_email = student_email.trim().to_lowercase();
    if normalized_email.is_empty() {
        return Ok(vec![]);
    }

    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let student_id: Option<i64> =
        sqlx::query_scalar("SELECT userID FROM users WHERE LOWER(emailAddress) = ?")
            .bind(&normalized_email)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to lookup user: {}", e)))?;

    let Some(student_id) = student_id else {
        return Ok(vec![]);
    };

    let today: NaiveDate = Local::now().naive_local().date();
    let today_str = today.format("%Y-%m-%d").to_string();

    let rows: Vec<(String, String, i64, i64)> = sqlx::query_as(
        r#"
        SELECT
            m.moduleCode,
            m.moduleTitle,
            COALESCE(SUM(CASE WHEN a.status = 'present' THEN 1 ELSE 0 END), 0) AS present_cnt,
            COALESCE(COUNT(a.attendanceID), 0) AS recorded_cnt
        FROM modules m
        INNER JOIN module_students ms ON ms.moduleCode = m.moduleCode
        LEFT JOIN classes c ON c.moduleCode = m.moduleCode AND c.date <= ?
        LEFT JOIN attendance a ON a.classID = c.classID AND a.studentID = ?
        WHERE ms.studentEmailAddress = ?
        GROUP BY m.moduleCode, m.moduleTitle
        ORDER BY m.moduleTitle ASC
        "#,
    )
    .bind(&today_str)
    .bind(student_id)
    .bind(&normalized_email)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Ok(rows
        .into_iter()
        .map(|(code, title, present, recorded)| StudentModuleBreakdown {
            module_code: code,
            module_title: title,
            present,
            recorded,
        })
        .collect())
}

#[server(GetStudentRecentActivity, "/api")]
pub async fn get_student_recent_activity(
    student_email: String,
) -> Result<Vec<StudentRecentActivity>, ServerFnError> {
    let normalized_email = student_email.trim().to_lowercase();
    if normalized_email.is_empty() {
        return Ok(vec![]);
    }

    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let student_id: Option<i64> =
        sqlx::query_scalar("SELECT userID FROM users WHERE LOWER(emailAddress) = ?")
            .bind(&normalized_email)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to lookup user: {}", e)))?;

    let Some(student_id) = student_id else {
        return Ok(vec![]);
    };

    let rows: Vec<(i64, String, String, String, String, Option<String>)> = sqlx::query_as(
        r#"
        SELECT
            c.classID,
            c.title,
            c.moduleCode,
            c.date,
            c.time,
            a.status
        FROM classes c
        INNER JOIN module_students ms ON ms.moduleCode = c.moduleCode
        LEFT JOIN attendance a ON a.classID = c.classID AND a.studentID = ?
        WHERE ms.studentEmailAddress = ?
        ORDER BY c.date DESC, c.time DESC
        LIMIT 10
        "#,
    )
    .bind(student_id)
    .bind(&normalized_email)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Ok(rows
        .into_iter()
        .map(
            |(class_id, title, module_code, date, time, status)| StudentRecentActivity {
                class_id,
                title,
                module_code,
                date,
                time,
                status: status.unwrap_or_else(|| "upcoming".to_string()),
            },
        )
        .collect())
}
