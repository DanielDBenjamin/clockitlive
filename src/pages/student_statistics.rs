use crate::routes::student_functions::{
    get_student_module_breakdown, get_student_recent_activity, get_student_stats_summary,
    get_student_weekly_attendance, StudentStatsSummary, StudentWeeklyAttendancePoint,
};
use crate::user_context::get_current_user;
use chrono::{Datelike, Local, NaiveDate, NaiveTime};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use urlencoding::encode;

const MODULE_COLORS: [&str; 6] = [
    "#10B981", "#3B82F6", "#F59E0B", "#8B5CF6", "#6366F1", "#F97316",
];

#[component]
pub fn StudentStatisticsPage() -> impl IntoView {
    let navigate = use_navigate();
    let current_user = get_current_user();

    let navigate_clone1 = navigate.clone();
    let navigate_clone2 = navigate.clone();
    let go_to_home = move |_| {
        navigate_clone1("/student/home", Default::default());
    };
    let go_to_home_nav = move |_| {
        navigate_clone2("/student/home", Default::default());
    };
    let go_to_profile = move |_| {
        navigate("/student/profile", Default::default());
    };

    let user_full_name = Signal::derive(move || {
        current_user
            .get()
            .map(|u| format!("{} {}", u.name, u.surname))
            .unwrap_or_else(|| "Student".to_string())
    });

    let user_first_name = Signal::derive(move || {
        current_user
            .get()
            .map(|u| u.name.clone())
            .unwrap_or_else(|| "Student".to_string())
    });

    let avatar_url = Signal::derive(move || {
        current_user.get().map(|u| {
            let full = format!("{} {}", u.name, u.surname);
            let encoded = encode(&full);
            format!(
                "https://ui-avatars.com/api/?name={}&background=14b8a6&color=ffffff&format=svg",
                encoded
            )
        })
    });

    let student_email = Signal::derive(move || current_user.get().map(|u| u.email_address.clone()));

    let summary = Resource::new(
        move || student_email.get(),
        |maybe_email| async move {
            match maybe_email {
                Some(email) => get_student_stats_summary(email).await,
                None => Ok(empty_summary()),
            }
        },
    );

    let weekly = Resource::new(
        move || student_email.get(),
        |maybe_email| async move {
            match maybe_email {
                Some(email) => get_student_weekly_attendance(email).await,
                None => Ok(Vec::new()),
            }
        },
    );

    let modules = Resource::new(
        move || student_email.get(),
        |maybe_email| async move {
            match maybe_email {
                Some(email) => get_student_module_breakdown(email).await,
                None => Ok(Vec::new()),
            }
        },
    );

    let recent = Resource::new(
        move || student_email.get(),
        |maybe_email| async move {
            match maybe_email {
                Some(email) => get_student_recent_activity(email).await,
                None => Ok(Vec::new()),
            }
        },
    );

    view! {
        <div class="student-stats-container">
            <header class="student-stats-header">
                <button class="student-back-button" on:click=go_to_home>
                    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M19 12H5M12 19l-7-7 7-7"/>
                    </svg>
                </button>
                <div class="student-stats-header-title">
                    <h1>"Statistics"</h1>   
                    <p>{move || user_full_name.get()}</p>
                </div>
                <div class="student-stats-header-actions">
                    <button class="student-profile-picture" on:click=go_to_profile>
                        <img
                            prop:src=move || {
                                avatar_url
                                    .get()
                                    .unwrap_or_else(|| "/logo.png".to_string())
                            }
                            alt=move || user_full_name.get()
                        />
                    </button>
                </div>
            </header>

            <div class="student-stats-body">
                <div class="stats-filters">
                    <div class="stats-filter-summary">
                        <span class="stats-greeting">{move || format!("Hi {}, here is your attendance.", user_first_name.get())}</span>
                        <span class="stats-email">{move || student_email.get().unwrap_or_default()}</span>
                    </div>
                </div>

                <Suspense fallback=move || view! { <div class="student-stats-cards loading">"Loading summary..."</div> }>
                    {move || summary.get().map(|result| match result {
                        Ok(stats) => {
                            let weekly_sessions_text = if stats.week_recorded > 0 {
                                format!("{} of {} sessions", stats.week_present, stats.week_recorded)
                            } else {
                                "No sessions recorded this week".to_string()
                            };
                            let overall_sessions_text = if stats.total_recorded > 0 {
                                format!("{} of {} sessions", stats.total_present, stats.total_recorded)
                            } else {
                                "No recorded sessions yet".to_string()
                            };

                            view! {
                                <div class="student-stats-cards">
                                    <div class="student-stat-card">
                                        <div class="student-stat-card-header">
                                            <span class="student-stat-label">"Overall"</span>
                                            <div class="student-stat-icon student-stat-icon-green">
                                                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                    <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"></polyline>
                                                </svg>
                                            </div>
                                        </div>
                                        <div class="student-stat-value">{format!("{:.0}%", stats.overall_attendance_rate)}</div>
                                        <div class="student-stat-change student-stat-change-positive">{overall_sessions_text}</div>
                                    </div>

                                    <div class="student-stat-card">
                                        <div class="student-stat-card-header">
                                            <span class="student-stat-label">"This Week"</span>
                                            <div class="student-stat-icon student-stat-icon-blue">
                                                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                    <rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
                                                    <line x1="16" y1="2" x2="16" y2="6"></line>
                                                    <line x1="8" y1="2" x2="8" y2="6"></line>
                                                    <line x1="3" y1="10" x2="21" y2="10"></line>
                                                </svg>
                                            </div>
                                        </div>
                                        <div class="student-stat-value">{format!("{:.0}%", stats.weekly_attendance_rate)}</div>
                                        <div class="student-stat-change">{weekly_sessions_text}</div>
                                    </div>
                                </div>
                            }
                            .into_any()
                        }
                        Err(err) => view! { <div class="student-stats-cards error">{format!("Error: {}", err)}</div> }.into_any(),
                    }).unwrap_or_else(|| view! { <div class="student-stats-cards">"Sign in to view statistics."</div> }.into_any())}
                </Suspense>

                <div class="student-chart-container">
                    <div class="student-chart-header">
                        <h2>"Weekly Attendance"</h2>
                    </div>
                    <Suspense fallback=move || view! { <div class="student-chart-content">"Loading weekly data..."</div> }>
                        {move || weekly.get().map(|result| match result {
                            Ok(points) => {
                                if points.is_empty() {
                                    return view! { <div class="student-chart-content">"No classes recorded for this week."</div> }.into_any();
                                }

                                let week_label = week_range_label(&points);
                                let worst_day_note = points
                                    .iter()
                                    .filter(|p| p.recorded > 0)
                                    .min_by(|a, b| attendance_percentage(a.present, a.recorded)
                                        .partial_cmp(&attendance_percentage(b.present, b.recorded))
                                        .unwrap_or(std::cmp::Ordering::Equal))
                                    .map(|day| {
                                        let missed = 100.0 - attendance_percentage(day.present, day.recorded);
                                        format!(
                                            "Most absences on {} (missed {:.0}% of sessions)",
                                            format_day_long(&day.date),
                                            missed
                                        )
                                    });

                                view! {
                                    <div class="student-weekly-meta">{format!("Week of {}", week_label)}</div>
                                    <div class="student-weekly-bars">
                                        {points
                                            .iter()
                                            .map(|point| {
                                                let pct = attendance_percentage(point.present, point.recorded);
                                                let label = format_day_short(&point.date);
                                                let meta = if point.recorded > 0 {
                                                    format!("{} / {} present", point.present, point.recorded)
                                                } else {
                                                    "No classes".to_string()
                                                };
                                                let fill_class = progress_fill_class(pct);
                                                view! {
                                                    <div class="student-weekly-bar">
                                                        <div class="student-weekly-bar-header">
                                                            <span class="student-weekly-bar-day">{label}</span>
                                                            <span class="student-weekly-bar-meta">{meta}</span>
                                                        </div>
                                                        <div class="progress-bar">
                                                            <div class={format!("progress-fill {}", fill_class)} style={format!("width: {:.1}%;", pct)}></div>
                                                        </div>
                                                        <span class="percentage-text">{format!("{:.0}%", pct)}</span>
                                                    </div>
                                                }
                                            })
                                            .collect::<Vec<_>>()}
                                    </div>
                                    {match worst_day_note {
                                        Some(ref text) => view! { <div class="student-weekly-note">{text.clone()}</div> }.into_any(),
                                        None => view! { <></> }.into_any(),
                                    }}
                                }
                                .into_any()
                            }
                            Err(err) => view! { <div class="student-chart-content">{format!("Error: {}", err)}</div> }.into_any(),
                        }).unwrap_or_else(|| view! { <div class="student-chart-content">"Sign in to view weekly data."</div> }.into_any())}
                    </Suspense>
                </div>

                <Suspense fallback=move || view! { <div class="module-breakdown">"Loading module breakdown..."</div> }>
                    {move || modules.get().map(|result| match result {
                        Ok(data) => {
                            if data.is_empty() {
                                return view! { <div class="module-breakdown">"No module attendance recorded yet."</div> }.into_any();
                            }

                            let best_module = data
                                .iter()
                                .filter(|m| m.recorded > 0)
                                .max_by(|a, b| attendance_percentage(a.present, a.recorded)
                                    .partial_cmp(&attendance_percentage(b.present, b.recorded))
                                    .unwrap_or(std::cmp::Ordering::Equal))
                                .cloned();

                            let worst_module = data
                                .iter()
                                .filter(|m| m.recorded > 0)
                                .min_by(|a, b| attendance_percentage(a.present, a.recorded)
                                    .partial_cmp(&attendance_percentage(b.present, b.recorded))
                                    .unwrap_or(std::cmp::Ordering::Equal))
                                .cloned();

                            view! {
                                <div class="insights-section">
                                    <h2>"Attendance Insights"</h2>

                                    <div class="insight-card insight-card-danger">
                                        <div class="insight-icon" style="background-color: #FEE2E2;">
                                            <svg width="24" height="24" viewBox="0 0 24 24" fill="#EF4444">
                                                <rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
                                                <line x1="16" y1="2" x2="16" y2="6" stroke="white" stroke-width="2"></line>
                                                <line x1="8" y1="2" x2="8" y2="6" stroke="white" stroke-width="2"></line>
                                                <line x1="3" y1="10" x2="21" y2="10" stroke="white" stroke-width="2"></line>
                                            </svg>
                                        </div>
                                        <div class="insight-content">
                                            <h3>"Most Missed Module"</h3>
                                            <p>{worst_module.clone().map(|m| format!("{} needs attention", m.module_title)).unwrap_or_else(|| "All modules are on track".to_string())}</p>
                                        </div>
                                        <div class="insight-percentage">
                                            <div class="progress-bar">
                                                <div class="progress-fill progress-fill-red" style={format!("width: {:.0}%;", worst_module.clone().map(|m| 100.0 - attendance_percentage(m.present, m.recorded)).unwrap_or(0.0))}></div>
                                            </div>
                                            <span class="percentage-text">{worst_module.clone().map(|m| format!("{:.0}%", 100.0 - attendance_percentage(m.present, m.recorded))).unwrap_or_else(|| "0%".to_string())}</span>
                                        </div>
                                    </div>

                                    <div class="insight-card insight-card-success">
                                        <div class="insight-icon" style="background-color: #D1FAE5;">
                                            <svg width="24" height="24" viewBox="0 0 24 24" fill="#10B981">
                                                <path d="M12 2L15.09 8.26L22 9.27L17 14.14L18.18 21.02L12 17.77L5.82 21.02L7 14.14L2 9.27L8.91 8.26L12 2Z"/>
                                            </svg>
                                        </div>
                                        <div class="insight-content">
                                            <h3>"Best Performance"</h3>
                                            <p>{best_module.clone().map(|m| format!("Great job in {}", m.module_title)).unwrap_or_else(|| "Keep up the consistency".to_string())}</p>
                                        </div>
                                        <div class="insight-percentage">
                                            <div class="progress-bar">
                                                <div class="progress-fill progress-fill-green" style={format!("width: {:.0}%;", best_module.clone().map(|m| attendance_percentage(m.present, m.recorded)).unwrap_or(0.0))}></div>
                                            </div>
                                            <span class="percentage-text">{best_module.clone().map(|m| format!("{:.0}%", attendance_percentage(m.present, m.recorded))).unwrap_or_else(|| "0%".to_string())}</span>
                                        </div>
                                    </div>
                                </div>
                                <div class="module-breakdown">
                                    <div class="module-breakdown-header">
                                        <h2>"Module Breakdown"</h2>
                                    </div>
                                    <div class="bar-chart">
                                        <div class="bar-chart-y-axis">
                                            <span>"100%"</span>
                                            <span>"50%"</span>
                                            <span>"0%"</span>
                                        </div>
                                        <div class="bar-chart-content">
                                            {data
                                                .iter()
                                                .enumerate()
                                                .map(|(index, module)| {
                                                    let pct = attendance_percentage(module.present, module.recorded);
                                                    let color = MODULE_COLORS[index % MODULE_COLORS.len()];
                                                    view! {
                                                        <div class="bar-column">
                                                            <div class="bar-wrapper">
                                                                <div class="bar" style={format!("height: {:.0}%; background-color: {};", pct, color)}></div>
                                                            </div>
                                                            <span class="bar-label">{module.module_title.clone()}</span>
                                                            <span class="bar-tooltip">{format!("{:.0}%", pct)}</span>
                                                        </div>
                                                    }
                                                })
                                                .collect::<Vec<_>>()}
                                        </div>
                                    </div>
                                </div>
                            }
                            .into_any()
                        }
                        Err(err) => view! { <div class="module-breakdown">{format!("Error: {}", err)}</div> }.into_any(),
                    }).unwrap_or_else(|| view! { <div class="module-breakdown">"Sign in to view module data."</div> }.into_any())}
                </Suspense>

                <Suspense fallback=move || view! { <div class="recent-activity">"Loading recent activity..."</div> }>
                    {move || recent.get().map(|result| match result {
                        Ok(items) => {
                            if items.is_empty() {
                                return view! { <div class="recent-activity">"No recent attendance activity."</div> }.into_any();
                            }

                            view! {
                                <div class="recent-activity">
                                    <h2>"Recent Activity"</h2>
                                    <div class="activity-list">
                                        {items
                                            .iter()
                                            .map(|entry| {
                                                let (status_class, status_text) = format_status_label(&entry.status);
                                                let label = format_recent_time(&entry.date, &entry.time);
                                                view! {
                                                    <div class="activity-item">
                                                        <div class={format!("activity-icon {}", status_class)}>
                                                            {if entry.status == "present" {
                                                                view! {
                                                                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#10B981" stroke-width="2">
                                                                        <polyline points="20 6 9 17 4 12"></polyline>
                                                                    </svg>
                                                                }.into_any()
                                                            } else if entry.status == "upcoming" {
                                                                view! {
                                                                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#3B82F6" stroke-width="2">
                                                                        <circle cx="12" cy="12" r="9"></circle>
                                                                        <path d="M12 8v4l2 2"></path>
                                                                    </svg>
                                                                }.into_any()
                                                            } else {
                                                                view! {
                                                                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#EF4444" stroke-width="2">
                                                                        <line x1="18" y1="6" x2="6" y2="18"></line>
                                                                        <line x1="6" y1="6" x2="18" y2="18"></line>
                                                                    </svg>
                                                                }.into_any()
                                                            }}
                                                        </div>
                                                        <div class="activity-content">
                                                            <h4>{entry.title.clone()}</h4>
                                                            <p>{format!("{} · {}", label, status_text)}</p>
                                                        </div>
                                                    </div>
                                                }
                                            })
                                            .collect::<Vec<_>>()}
                                    </div>
                                </div>
                            }
                            .into_any()
                        }
                        Err(err) => view! { <div class="recent-activity">{format!("Error: {}", err)}</div> }.into_any(),
                    }).unwrap_or_else(|| view! { <div class="recent-activity">"Sign in to view recent activity."</div> }.into_any())}
                </Suspense>
            </div>

            <nav class="student-bottom-nav">
                <button class="student-nav-item" on:click=go_to_home_nav>
                    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path>
                        <polyline points="9 22 9 12 15 12 15 22"></polyline>
                    </svg>
                    <span class="student-nav-label">"Home"</span>
                </button>

                <button class="student-nav-item student-nav-item-scan" disabled=true data-testid="scan-button-disabled">
                    <div class="student-scan-button">
                        <img src="/i.png" alt="Scan QR" width="46" height="32" data-testid="qr-icon"/>
                    </div>
                    <span class="student-nav-label">"Scan QR"</span>
                </button>                <button class="student-nav-item student-nav-item-active">
                    <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" stroke-width="2">
                        <line x1="18" y1="20" x2="18" y2="10"></line>
                        <line x1="12" y1="20" x2="12" y2="4"></line>
                        <line x1="6" y1="20" x2="6" y2="14"></line>
                    </svg>
                    <span class="student-nav-label">"Stats"</span>
                </button>
            </nav>
        </div>
    }
}

fn empty_summary() -> StudentStatsSummary {
    StudentStatsSummary {
        overall_attendance_rate: 0.0,
        weekly_attendance_rate: 0.0,
        total_present: 0,
        total_recorded: 0,
        upcoming_classes: 0,
        week_present: 0,
        week_recorded: 0,
    }
}

fn attendance_percentage(present: i64, recorded: i64) -> f64 {
    if recorded > 0 {
        (present as f64) * 100.0 / (recorded as f64)
    } else {
        0.0
    }
}

fn format_day_short(date_iso: &str) -> String {
    NaiveDate::parse_from_str(date_iso, "%Y-%m-%d")
        .map(|date| date.format("%a").to_string())
        .unwrap_or_else(|_| date_iso.to_string())
}

fn format_day_long(date_iso: &str) -> String {
    NaiveDate::parse_from_str(date_iso, "%Y-%m-%d")
        .map(|date| date.format("%A").to_string())
        .unwrap_or_else(|_| date_iso.to_string())
}

fn week_range_label(points: &[StudentWeeklyAttendancePoint]) -> String {
    let dates: Vec<NaiveDate> = points
        .iter()
        .filter_map(|p| NaiveDate::parse_from_str(&p.date, "%Y-%m-%d").ok())
        .collect();

    if let (Some(start), Some(end)) = (dates.first(), dates.last()) {
        if start.month() == end.month() {
            format!("{} - {}", start.format("%d %b"), end.format("%d %b"))
        } else {
            format!("{} - {}", start.format("%d %b"), end.format("%d %b"))
        }
    } else {
        let today = Local::now().naive_local().date();
        let start_of_week =
            today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);
        let end_of_week = start_of_week + chrono::Duration::days(6);
        format!(
            "{} - {}",
            start_of_week.format("%d %b"),
            end_of_week.format("%d %b")
        )
    }
}

fn format_recent_time(date_iso: &str, time: &str) -> String {
    let today = Local::now().naive_local().date();
    let parsed_date = NaiveDate::parse_from_str(date_iso, "%Y-%m-%d").unwrap_or(today);
    let parsed_time = NaiveTime::parse_from_str(time, "%H:%M")
        .unwrap_or(NaiveTime::from_hms_opt(0, 0, 0).unwrap());

    let date_label = if parsed_date == today {
        "Today".to_string()
    } else if parsed_date == today - chrono::Duration::days(1) {
        "Yesterday".to_string()
    } else {
        parsed_date.format("%d %b").to_string()
    };

    format!("{} {}", date_label, parsed_time.format("%H:%M"))
}

fn progress_fill_class(pct: f64) -> &'static str {
    if pct >= 80.0 {
        "progress-fill-green"
    } else if pct >= 50.0 {
        "progress-fill-orange"
    } else {
        "progress-fill-red"
    }
}

fn format_status_label(status: &str) -> (&'static str, &'static str) {
    match status {
        "present" => ("activity-icon-success", "Present"),
        "late" => ("activity-icon-warning", "Late"),
        "excused" => ("activity-icon-warning", "Excused"),
        "upcoming" => ("activity-icon-info", "Upcoming"),
        _ => ("activity-icon-error", "Absent"),
    }
}
