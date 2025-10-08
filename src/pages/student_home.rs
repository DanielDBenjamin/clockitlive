use crate::components::QrScanner;
use crate::routes::class_functions::record_session_attendance_fn;
use crate::routes::student_functions::{get_student_schedule, StudentScheduleItem};
use crate::user_context::get_current_user;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use urlencoding::encode;
use std::collections::HashMap;

fn current_date_iso() -> String {
    chrono::Local::now()
        .naive_local()
        .format("%Y-%m-%d")
        .to_string()
}

fn format_pretty_date(date_iso: &str) -> String {
    chrono::NaiveDate::parse_from_str(date_iso, "%Y-%m-%d")
        .map(|date| date.format("%A, %d %b").to_string())
        .unwrap_or_else(|_| chrono::Local::now().format("%A, %d %b").to_string())
}

fn format_short_date(date_iso: &str) -> String {
    chrono::NaiveDate::parse_from_str(date_iso, "%Y-%m-%d")
        .map(|date| date.format("%d %b").to_string())
        .unwrap_or_else(|_| date_iso.to_string())
}

// Helper function to check if venue was recently updated (within 48 hours)
fn is_venue_recently_updated(venue_updated_at: &Option<String>) -> bool {
    if let Some(updated_at) = venue_updated_at {
        if let Ok(updated_time) = chrono::DateTime::parse_from_rfc3339(updated_at) {
            let now = chrono::Utc::now();
            let duration = now.signed_duration_since(updated_time);
            return duration.num_hours() <= 48;
        }
    }
    false
}

// LocalStorage helpers for dismissed venue alerts
#[cfg(not(feature = "ssr"))]
fn get_dismissed_venue_alerts() -> HashMap<i64, String> {
    use wasm_bindgen::JsValue;
    
    let window = web_sys::window().expect("no global `window` exists");
    let storage = window.local_storage().ok().flatten();
    
    if let Some(storage) = storage {
        if let Ok(Some(data)) = storage.get_item("dismissed_venue_alerts") {
            if let Ok(map) = serde_json::from_str::<HashMap<i64, String>>(&data) {
                return map;
            }
        }
    }
    HashMap::new()
}

#[cfg(not(feature = "ssr"))]
fn add_dismissed_venue_alert(class_id: i64, venue_updated_at: String) {
    use wasm_bindgen::JsValue;
    
    let window = web_sys::window().expect("no global `window` exists");
    let storage = window.local_storage().ok().flatten();
    
    if let Some(storage) = storage {
        let mut dismissed = get_dismissed_venue_alerts();
        dismissed.insert(class_id, venue_updated_at);
        if let Ok(json) = serde_json::to_string(&dismissed) {
            let _ = storage.set_item("dismissed_venue_alerts", &json);
        }
    }
}

#[cfg(feature = "ssr")]
fn get_dismissed_venue_alerts() -> HashMap<i64, String> {
    HashMap::new()
}

#[cfg(feature = "ssr")]
fn add_dismissed_venue_alert(_class_id: i64, _venue_updated_at: String) {}

#[component]
pub fn StudentHomePage() -> impl IntoView {
    let navigate = use_navigate();
    let (show_scanner, set_show_scanner) = signal(false);
    let (_scanned_data, set_scanned_data) = signal(None::<String>);
    let feedback = RwSignal::new(None::<(bool, String)>);
    let current_user = get_current_user();

    let selected_date = RwSignal::new(current_date_iso());
    let schedule_feedback = RwSignal::new(None::<String>);
    let dismissed_alerts = RwSignal::new(get_dismissed_venue_alerts());

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
            let full_name = format!("{} {}", u.name, u.surname);
            let encoded = encode(&full_name);
            format!(
                "https://ui-avatars.com/api/?name={}&background=14b8a6&color=ffffff&format=svg",
                encoded
            )
        })
    });

    let date_label = Signal::derive(move || format_pretty_date(&selected_date.get()));

    let subtitle_text = Signal::derive(move || format!("Welcome back, {}!", user_first_name.get()));

    let schedule_resource = Resource::new(
        move || {
            current_user
                .get()
                .map(|user| (user.email_address.clone(), selected_date.get()))
        },
        {
            let schedule_feedback = schedule_feedback.clone();
            move |params: Option<(String, String)>| async move {
                match params {
                    Some((email, date)) => {
                        match get_student_schedule(email, Some(date.clone())).await {
                            Ok(response) => {
                                if (!response.success || response.classes.is_empty())
                                    && !response.message.is_empty()
                                {
                                    schedule_feedback.set(Some(response.message.clone()));
                                } else {
                                    schedule_feedback.set(None);
                                }
                                Some(response.classes)
                            }
                            Err(err) => {
                                schedule_feedback.set(Some(err.to_string()));
                                None
                            }
                        }
                    }
                    None => {
                        schedule_feedback
                            .set(Some("Please sign in to view your schedule.".to_string()));
                        None
                    }
                }
            }
        },
    );

    let handle_scan = {
        let set_scanned_data = set_scanned_data.clone();
        let set_show_scanner = set_show_scanner.clone();
        let feedback = feedback.clone();
        let current_user = current_user.clone();
        Callback::new(move |data: String| {
            set_scanned_data.set(Some(data.clone()));
            set_show_scanner.set(false);
            if let Some(user) = current_user.get() {
                let email = user.email_address.clone();
                let feedback = feedback.clone();
                let payload = data.clone();
                spawn_local(async move {
                    #[cfg(not(feature = "ssr"))]
                    {
                        match crate::utils::geolocation::get_current_location().await {
                            Ok(location) => {
                                match record_session_attendance_fn(
                                    payload.clone(),
                                    email.clone(),
                                    Some(location.latitude),
                                    Some(location.longitude),
                                    location.accuracy,
                                )
                                .await
                                {
                                    Ok(resp) => {
                                        feedback.set(Some((resp.success, resp.message)));
                                    }
                                    Err(e) => {
                                        feedback.set(Some((false, e.to_string())));
                                    }
                                }
                            }
                            Err(err) => {
                                feedback.set(Some((false, err)));
                            }
                        }
                    }

                    #[cfg(feature = "ssr")]
                    {
                        feedback.set(Some((
                            false,
                            "Location capture requires a browser.".to_string(),
                        )));
                    }
                });
            } else {
                feedback.set(Some((
                    false,
                    "Please log in as a student to record attendance.".to_string(),
                )));
            }
        })
    };

    let handle_close_scanner = Callback::new(move |_| {
        set_show_scanner.set(false);
    });

    let open_scanner = move |_| {
        set_show_scanner.set(true);
    };

    let navigate_clone = navigate.clone();
    let go_to_profile = move |_| {
        navigate_clone("/student/profile", Default::default());
    };

    let go_to_statistics = move |_| {
        navigate("/student/statistics", Default::default());
    };

    view! {
        <div class="student-home-container">
            {/* Header */}
            <header class="student-home-header">
                <div class="student-header-logo">
                    <img src="/logo.png" srcset="/logo.png 1x" alt="Clock It" class="student-brand-logo-img" width="160" height="60" />
                </div>
                <div class="student-header-actions">
                    <button class="student-profile-picture" on:click=go_to_profile>
                        <img
                            alt=move || user_full_name.get()
                            prop:src=move || {
                                avatar_url
                                    .get()
                                    .unwrap_or_else(|| "/logo.png".to_string())
                            }
                        />
                    </button>
                </div>
            </header>

            <div class="student-home-content">
                {/* Date and title section */}
                <section class="student-date-section">
                    <h2 class="student-date-title">{date_label}</h2>
                    <p class="student-date-subtitle">{subtitle_text}</p>
                </section>

                {/* Module cards */}
                <Suspense fallback=move || view! { <div class="student-modules-list"><div class="student-module-card loading">"Loading your schedule…"</div></div> }>
                    {move || {
                        schedule_resource
                            .get()
                            .map(|maybe_classes| {
                                let current_date = selected_date.get();
                                match maybe_classes {
                                    Some(classes) => {
                                        if classes.is_empty() {
                                            let message = schedule_feedback
                                                .get()
                                                .unwrap_or_else(|| {
                                                    "No upcoming classes found.".to_string()
                                                });
                                            view! {
                                                <div class="student-modules-list">
                                                    <div class="student-module-empty">{message}</div>
                                                </div>
                                            }
                                            .into_any()
                                        } else {
                                            view! {
                                                <div class="student-modules-list">
                                                    {classes
                                                        .into_iter()
                                                        .enumerate()
                                                        .map(|(index, class)| {
                                                            schedule_card(class, index, current_date.clone(), dismissed_alerts)
                                                        })
                                                        .collect::<Vec<_>>()}
                                                </div>
                                            }
                                            .into_any()
                                        }
                                    }
                                    None => {
                                        let message = schedule_feedback
                                            .get()
                                            .unwrap_or_else(|| "No schedule data available.".to_string());
                                        view! {
                                            <div class="student-modules-list">
                                                <div class="student-module-empty">{message}</div>
                                            </div>
                                        }
                                        .into_any()
                                    }
                                }
                            })
                            .unwrap_or_else(|| {
                                view! {
                                    <div class="student-modules-list">
                                        <div class="student-module-empty">"Sign in to view your schedule."</div>
                                    </div>
                                }
                                .into_any()
                            })
                    }}
                </Suspense>
            </div>

            {move || feedback.get().map(|(ok, msg)| {
                let class_name = if ok { "student-feedback success" } else { "student-feedback error" };
                view! { <div class=class_name>{msg}</div> }.into_any()
            }).unwrap_or_else(|| view! { <></> }.into_any())}

            {/* Bottom Navigation */}
            <nav class="student-bottom-nav">
                <button class="student-nav-item student-nav-item-active">
                    <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path>
                        <polyline points="9 22 9 12 15 12 15 22"></polyline>
                    </svg>
                    <span class="student-nav-label">"Home"</span>
                </button>

                <button class="student-nav-item student-nav-item-scan" on:click=open_scanner>
                    <div class="student-scan-button">
                        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <rect x="3" y="3" width="7" height="7"></rect>
                            <rect x="14" y="3" width="7" height="7"></rect>
                            <rect x="14" y="14" width="7" height="7"></rect>
                            <rect x="3" y="14" width="7" height="7"></rect>
                        </svg>
                    </div>
                    <span class="student-nav-label">"Scan"</span>
                </button>

                <button class="student-nav-item" on:click=go_to_statistics>
                    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <line x1="18" y1="20" x2="18" y2="10"></line>
                        <line x1="12" y1="20" x2="12" y2="4"></line>
                        <line x1="6" y1="20" x2="6" y2="14"></line>
                    </svg>
                    <span class="student-nav-label">"Stats"</span>
                </button>
            </nav>

            {/* QR Scanner Modal */}
            {move || if show_scanner.get() {
                view! {
                    <QrScanner
                        on_scan=handle_scan
                        on_close=handle_close_scanner
                    />
                }.into_any()
            } else {
                view! { <></> }.into_any()
            }}
        </div>
    }
}

fn schedule_card(
    class: StudentScheduleItem,
    index: usize,
    current_date_iso: String,
    dismissed_alerts: RwSignal<HashMap<i64, String>>,
) -> impl IntoView {
    let StudentScheduleItem {
        class_id,
        module_code,
        module_title,
        class_title,
        venue,
        date,
        time,
        status: _,
        venue_updated_at,
    } = class;

    let color_class = match index % 4 {
        0 => "purple",
        1 => "red",
        2 => "yellow",
        _ => "teal",
    };

    let icon_text = module_code
        .chars()
        .find(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase().to_string())
        .unwrap_or_else(|| "•".to_string());

    let venue_text = venue.unwrap_or_else(|| "Location TBA".to_string());
    let details = if class_title.trim().is_empty() {
        venue_text.clone()
    } else if venue_text.trim().is_empty() {
        class_title.clone()
    } else {
        format!("{} · {}", class_title, venue_text)
    };

    let display_line = if date == current_date_iso {
        details
    } else {
        format!("{} · {}", format_short_date(&date), details)
    };

    // Check if we should show the venue alert dot
// Check if we should show the venue alert dot
// Check if we should show the venue alert dot
let venue_updated_at_clone = venue_updated_at.clone();
let show_venue_alert = Signal::derive(move || {
    if !is_venue_recently_updated(&venue_updated_at_clone) {
        return false;
    }
    
    // Check if we've seen THIS specific venue update
    if let Some(updated_at) = &venue_updated_at_clone {
        let dismissed = dismissed_alerts.get();
        match dismissed.get(&class_id) {
            Some(seen_timestamp) => {
                // Show dot if venue was updated AFTER we dismissed it
                seen_timestamp != updated_at
            }
            None => {
                true
            }
        }
    } else {
        false
    }
});

let handle_card_click = move |_| {
    if show_venue_alert.get() {
        if let Some(updated_at) = &venue_updated_at {
            add_dismissed_venue_alert(class_id, updated_at.clone());
            let mut alerts = dismissed_alerts.get();
            alerts.insert(class_id, updated_at.clone());
            dismissed_alerts.set(alerts);
        }
    }
};
    view! {
        <button class="student-module-card" on:click=handle_card_click>
            {move || show_venue_alert.get().then(|| view! {
                <span class="venue-alert-dot"></span>
            })}
            <div class="student-module-time">{time}</div>
            <div class={format!("student-module-icon student-module-icon-{}", color_class)}>
                {icon_text}
            </div>
            <div class="student-module-info">
                <div class="student-module-name">{module_title}</div>
                <div class="student-module-location">{display_line}</div>
            </div>
        </button>
    }
}