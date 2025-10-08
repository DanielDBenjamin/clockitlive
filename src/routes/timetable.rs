use crate::database::classes::Class;
use crate::routes::class_functions::{
    check_all_active_sessions_fn, get_active_class_session_fn, get_lecturer_classes_fn, start_class_session_fn,
};
use crate::routes::stats_functions::get_module_enrollment_count;
use crate::user_context::get_current_user;
use chrono::{Local, NaiveTime};
use leptos::prelude::*;
use leptos::server_fn::ServerFnError;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use std::collections::HashSet;

#[component]
pub fn Timetable() -> impl IntoView {
    let current_user = get_current_user();

    // Auto-refresh sessions every 30 seconds to check if they should end
    let session_refresh_trigger = RwSignal::new(0);

    // Check all active sessions when page loads to auto-end expired ones
    let _session_cleanup = Resource::new(
        move || current_user.get().map(|u| u.email_address.clone()),
        |email| async move {
            if email.is_some() {
                let _ = check_all_active_sessions_fn().await;
            }
        },
    );

    // Load all classes for the lecturer
    let classes_resource = Resource::new(
        move || (current_user.get().map(|u| u.email_address.clone()), session_refresh_trigger.get()),
        |(email, _)| async move {
            match email {
                Some(email) => match get_lecturer_classes_fn(email).await {
                    Ok(response) if response.success => Some(response.classes),
                    _ => None,
                },
                None => None,
            }
        },
    );

    // Clock display on the right (formatted like 09:45 AM)
    let current_time = Signal::derive(move || Local::now().format("%I:%M %p").to_string());
    
    // Set up automatic session checking 
    #[cfg(not(feature = "ssr"))]
    {
        // Run cleanup immediately on page load
        leptos::task::spawn_local(async {
            let _ = check_all_active_sessions_fn().await;
        });
        
        // Set up periodic cleanup every 30 seconds
        let trigger = session_refresh_trigger.clone();
        leptos::task::spawn_local(async move {
            use gloo_timers::future::sleep;
            use std::time::Duration;
            
            loop {
                sleep(Duration::from_secs(30)).await;
                let _ = check_all_active_sessions_fn().await;
                trigger.update(|n| *n += 1);
            }
        });
    }

    // Simple day filter for the timetable header
    let filter_choice = RwSignal::new("Today".to_string());

    // Compute selected date (iso) based on filter
    let selected_date_iso = Signal::derive(move || {
        let base = Local::now().naive_local().date();
        let offset = match filter_choice.get().as_str() {
            "Yesterday" => -1,
            "Tomorrow" => 1,
            _ => 0,
        };
        (base + chrono::Duration::days(offset))
            .format("%Y-%m-%d")
            .to_string()
    });

    // Pretty label like: "Today, Wednesday, October 9 2024"
    let selected_date_pretty = Signal::derive(move || {
        let base = Local::now().naive_local().date();
        let (prefix, offset) = match filter_choice.get().as_str() {
            "Yesterday" => ("Yesterday", -1),
            "Tomorrow" => ("Tomorrow", 1),
            _ => ("Today", 0),
        };
        let date = base + chrono::Duration::days(offset);
        format!("{}, {}", prefix, date.format("%A, %B %e %Y"))
    });

    // Filter classes by selected date
    let filtered_classes = Signal::derive(move || {
        let sel = selected_date_iso.get();
        classes_resource
            .get()
            .and_then(|opt| opt)
            .unwrap_or_default()
            .into_iter()
            .filter(|c| c.date == sel)
            .collect::<Vec<_>>()
    });

    let total_lectures = Signal::derive(move || filtered_classes.get().len());
    let total_minutes = Signal::derive(move || {
        filtered_classes
            .get()
            .into_iter()
            .map(|c| c.duration_minutes.max(0))
            .sum::<i32>()
    });
    let teaching_hours = Signal::derive(move || {
        let minutes = total_minutes.get();
        if minutes <= 0 {
            "0h".to_string()
        } else if minutes % 60 == 0 {
            format!("{}h", minutes / 60)
        } else {
            format!("{:.1}h", minutes as f64 / 60.0)
        }
    });

    // Aggregate total students by fetching enrollment for unique modules shown today
    let total_students_resource = Resource::new(
        move || {
            filtered_classes
                .get()
                .into_iter()
                .map(|c| c.module_code.clone())
                .collect::<Vec<_>>()
        },
        |module_codes| async move {
            let mut seen = HashSet::new();
            let mut total = 0;
            for code in module_codes {
                if seen.insert(code.clone()) {
                    if let Ok(count) = get_module_enrollment_count(code).await {
                        total += count;
                    }
                }
            }
            total
        },
    );

    view! {
        <section class="timetable">
            <div class="timetable-topbar">
                <div class="header-row">
                    <div class="page-title-group">
                        <h1 class="page-title">"My Timetable"</h1>
                        <p class="page-subtitle">"Manage your lectures and sessions"</p>
                    </div>
                    <div class="header-actions">
                        <select class="btn btn-outline select-dropdown" bind:value=filter_choice>
                            <option value="Today">"Today"</option>
                            <option value="Tomorrow">"Tomorrow"</option>
                            <option value="Yesterday">"Yesterday"</option>
                        </select>
                        <A href="/classes/new" attr:class="btn btn-primary">"+ Add Class"</A>
                    </div>
                </div>
                <div class="info-bar">
                    <div class="bar-left">
                        <div class="day-date">{move || selected_date_pretty.get()}</div>
                        <div class="bar-meta muted">{move || format!("{} lectures scheduled", total_lectures.get())}</div>
                    </div>
                    <div class="bar-right">
                        <div class="label">"Current Time"</div>
                        <div class="time">{move || current_time.get()}</div>
                    </div>
                </div>
            </div>

            <div class="timetable-body">
                <Suspense fallback=move || view! { <div class="loading">"Loading classes..."</div> }>
                    {move || {
                        let mut list = filtered_classes.get();
                        list.sort_by(|a,b| a.time.cmp(&b.time));
                        if list.is_empty() {
                            view! { <div class="empty-state"><p>"No classes scheduled for the selected day."</p></div> }.into_any()
                        } else {
                            view! { <div class="list">
                                {list.into_iter().map(|c| view! { <TimetableRow class=c/> }).collect_view()}
                            </div> }.into_any()
                        }
                    }}
                </Suspense>
            </div>

            <div class="summary-bar">
                <div class="summary-card">
                    <div class="summary-header">
                        <div class="summary-title">
                            <h3>"Today's Summary"</h3>
                            <span class="muted">{move || selected_date_pretty.get()}</span>
                        </div>
                        <A href="/statistics" attr:class="btn summary-btn">"View Analytics"</A>
                    </div>

                    <div class="summary-stats">
                        <div class="summary-stat">
                            <div class="label">"Total Lectures"</div>
                            <div class="value">{move || total_lectures.get().to_string()}</div>
                        </div>
                        <Suspense fallback=|| view! {
                            <div class="summary-stat">
                                <div class="label">"Total Students"</div>
                                <div class="value">"…"</div>
                            </div>
                        }>
                            {move || total_students_resource.get().map(|total| {
                                view! {
                                    <div class="summary-stat">
                                        <div class="label">"Total Students"</div>
                                        <div class="value">{total.to_string()}</div>
                                    </div>
                                }.into_any()
                            })}
                        </Suspense>
                        <div class="summary-stat">
                            <div class="label">"Teaching Hours"</div>
                            <div class="value">{teaching_hours}</div>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}

#[component]
fn TimetableRow(class: Class) -> impl IntoView {
    let class_id = class.class_id;
    let navigate = use_navigate();
    let current_status = RwSignal::new(class.status.clone());
    let status_label = Signal::derive(move || match current_status.get().as_str() {
        "completed" => "Completed".to_string(),
        "in_progress" => "In Progress".to_string(),
        "upcoming" => "Upcoming".to_string(),
        _ => "Unknown".to_string(),
    });
    let status_in_progress = Signal::derive(move || current_status.get() == "in_progress");
    let status_upcoming = Signal::derive(move || current_status.get() == "upcoming");

    let time_range = {
        // We only store start time; render as start - (start + duration)
        let start = NaiveTime::parse_from_str(&class.time, "%H:%M")
            .unwrap_or(NaiveTime::from_hms_opt(10, 0, 0).unwrap());
        let minutes = class.duration_minutes.max(15) as i64;
        let end = start + chrono::Duration::minutes(minutes);
        format!("{} - {}", start.format("%H:%M"), end.format("%H:%M"))
    };
    let duration_display = {
        let minutes = class.duration_minutes.max(15);
        if minutes % 60 == 0 {
            format!("{}h", minutes / 60)
        } else {
            format!("{} min", minutes)
        }
    };
    let start_href = format!("/classes/qr?id={}&origin=timetable", class.class_id);
    let edit_href = format!("/classes/edit?id={}&origin=timetable", class.class_id);
    let venue = class.venue.clone().unwrap_or_else(|| "TBA".to_string());
    let module_code_display = class.module_code.clone();

    const LOCATION_RADIUS_METERS: f64 = 30.0;
    let location_status = RwSignal::new(None::<String>);
    let location_error = RwSignal::new(None::<String>);

    #[cfg(not(feature = "ssr"))]
    let start_session_action = Action::new_local({
        let location_status = location_status.clone();
        let location_error = location_error.clone();
        move |id: &i64| {
            let id = *id;
            let location_status = location_status.clone();
            let location_error = location_error.clone();
            async move {
                location_error.set(None);
                location_status.set(Some("Requesting location...".to_string()));

                match crate::utils::geolocation::get_current_location().await {
                    Ok(loc) => {
                        location_status
                            .set(Some("Location captured. Starting session...".to_string()));
                        let response = start_class_session_fn(
                            id,
                            Some(loc.latitude),
                            Some(loc.longitude),
                            loc.accuracy,
                            Some(LOCATION_RADIUS_METERS),
                        )
                        .await;

                        if response.is_err() {
                            location_status.set(None);
                        }

                        response
                    }
                    Err(err) => {
                        location_status.set(None);
                        location_error.set(Some(err.clone()));
                        Err(ServerFnError::new(err))
                    }
                }
            }
        }
    });

    #[cfg(feature = "ssr")]
    let start_session_action = Action::new_local({
        let location_status = location_status.clone();
        let location_error = location_error.clone();
        move |_id: &i64| {
            let location_status = location_status.clone();
            let location_error = location_error.clone();
            async move {
                location_status.set(None);
                let msg = "Location capture requires a browser context.".to_string();
                location_error.set(Some(msg.clone()));
                Err::<crate::routes::class_functions::ClassSessionResponse, ServerFnError>(
                    ServerFnError::new(msg),
                )
            }
        }
    });
    let start_pending = start_session_action.pending();

    Effect::new({
        let navigate = navigate.clone();
        let href = start_href.clone();
        let current_status = current_status.clone();
        let location_status = location_status.clone();
        let location_error = location_error.clone();
        move |_| {
            if let Some(result) = start_session_action.value().get() {
                match result {
                    Ok(response) => {
                        if let Some(status) = response.class_status.clone() {
                            current_status.set(status);
                        }
                        if response.success {
                            location_status.set(None);
                            location_error.set(None);
                            navigate(&href, Default::default());
                        } else {
                            location_status.set(None);
                            location_error.set(Some(response.message.clone()));
                            leptos::logging::log!("Start session failed: {}", response.message);
                        }
                    }
                    Err(e) => {
                        location_status.set(None);
                        location_error.set(Some(e.to_string()));
                        leptos::logging::log!("Start session error: {}", e);
                    }
                }
            }
        }
    });

    let session_check = Resource::new(
        move || class_id,
        |id| async move { get_active_class_session_fn(id).await },
    );

    Effect::new({
        let current_status = current_status.clone();
        move |_| {
            if let Some(result) = session_check.get() {
                match result {
                    Ok(response) => {
                        if let Some(status) = response.class_status.clone() {
                            current_status.set(status);
                        }
                    }
                    Err(e) => {
                        leptos::logging::log!("Session check error: {}", e);
                    }
                }
            }
        }
    });

    // Enrollment count for the module of this class
    let enroll_count = Resource::new(
        move || class.module_code.clone(),
        |code| async move { get_module_enrollment_count(code).await },
    );

    // Determine color variant like Home page; use module initials as icon (ASCII only)
    let hash = module_code_display.chars().map(|c| c as u32).sum::<u32>();
    let (icon, variant) = match hash % 4 {
        0 => ("</>", "mod-purp"),
        1 => ("🗄️", "mod-blue"),
        2 => ("🧩", "mod-orange"),
        _ => ("🍃", "mod-green"),
    };
    let initials = {
        let letters: String = module_code_display
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .take(3)
            .collect();
        let up = letters.to_uppercase();
        if up.is_empty() {
            "MOD".to_string()
        } else {
            up
        }
    };
    let class_icon_classes = format!("class-icon {}", variant);

    let location_status_display = location_status.clone();
    let location_error_display = location_error.clone();

    view! {
        <div class="card timetable-row">
            <div class="row-left">
                <div class=class_icon_classes>{initials}</div>
                <div class="row-info">
                    <div class="row-title">{format!("{} - {}", module_code_display, class.title.clone())}</div>
                    <div class="row-meta muted">
                        <span class="meta">{time_range.clone()}</span>
                        <span class="meta">{duration_display}</span>
                        <span class="meta">{venue}</span>
                        <span class="meta">{move || status_label.get()}</span>
                        <Suspense fallback=|| view! { <span class="meta"><span>"..."</span></span> }>
                            {move || enroll_count.get().map(|res| match res {
                                Ok(n) => view! { <span class="meta">{format!("{} students", n)}</span> }.into_any(),
                                Err(_) => view! { <span class="meta">"—"</span> }.into_any(),
                            })}
                        </Suspense>
                    </div>
                    <div class="muted">{class.description.clone().unwrap_or_default()}</div>
                </div>
            </div>
            <div class="row-actions">
                <Show when=move || status_upcoming.get()>
                    <button
                        class="btn btn-primary"
                        disabled=move || start_pending.get()
                        on:click=move |_| {
                            if !start_pending.get() {
                                start_session_action.dispatch(class_id);
                            }
                        }
                    >{move || if start_pending.get() { "Starting..." } else { "Start Session" }}</button>
                </Show>
                {move || location_status_display
                    .get()
                    .map(|msg| view! { <div class="location-status muted">{msg}</div> }.into_any())
                    .unwrap_or_else(|| view! { <></> }.into_any())}
                {move || location_error_display
                    .get()
                    .map(|msg| view! { <div class="location-error">{msg}</div> }.into_any())
                    .unwrap_or_else(|| view! { <></> }.into_any())}
                <Show when=move || status_in_progress.get()>
                    <A href=start_href.clone() attr:class="btn btn-outline">"View Session"</A>
                </Show>
                <A href=edit_href attr:class="btn btn-outline">"Edit"</A>
            </div>
        </div>
    }
}
