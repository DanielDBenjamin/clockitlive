use chrono::{Duration, Local, NaiveDate, NaiveTime};
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_query_map};
use qrcode::{render::svg, QrCode};
use urlencoding::encode;

use crate::routes::{
    class_functions::{end_class_session_fn, get_active_class_session_fn, get_class_fn},
    helpers::build_return_path,
};

fn format_date_label(date_str: &str) -> (String, String) {
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        let today = Local::now().naive_local().date();
        let label = if date == today {
            "Today".to_string()
        } else if date == today + Duration::days(1) {
            "Tomorrow".to_string()
        } else if date == today - Duration::days(1) {
            "Yesterday".to_string()
        } else {
            date.format("%A").to_string()
        };
        (label, date.format("%b %e, %Y").to_string())
    } else {
        ("Date".to_string(), date_str.to_string())
    }
}

fn format_time_label(time_str: &str, duration_minutes: i32) -> (String, String) {
    if let Ok(time) = NaiveTime::parse_from_str(time_str, "%H:%M") {
        let formatted = time.format("%I:%M %p").to_string();
        let duration_display = if duration_minutes % 60 == 0 {
            format!("{} hours", duration_minutes / 60)
        } else {
            format!("{} min", duration_minutes)
        };
        (formatted, duration_display)
    } else {
        (time_str.to_string(), format!("{} min", duration_minutes))
    }
}

fn format_session_started(iso: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(iso)
        .map(|dt| dt.with_timezone(&Local).format("%I:%M %p").to_string())
        .unwrap_or_else(|_| iso.to_string())
}

fn build_qr_svg(data: &str, size: u32) -> Option<String> {
    QrCode::new(data.as_bytes()).ok().map(|code| {
        code.render::<svg::Color>()
            .min_dimensions(size, size)
            .dark_color(svg::Color("#1f2937"))
            .light_color(svg::Color("#ffffff"))
            .build()
    })
}

#[component]
pub fn ClassQrPage() -> impl IntoView {
    let navigate = use_navigate();
    let query = use_query_map();
    let query_for_id = query.clone();
    let class_id = Signal::derive(move || {
        query_for_id
            .with(|q| q.get("id").and_then(|id| id.parse::<i64>().ok()))
            .unwrap_or(0)
    });

    let origin_signal =
        Signal::derive(move || query.with(|q| q.get("origin").map(|s| s.to_string())));

    let last_return_path = RwSignal::new(String::new());

    let class_resource = Resource::new(
        move || class_id.get(),
        |id| async move {
            if id == 0 {
                None
            } else {
                match get_class_fn(id).await {
                    Ok(response) if response.success => response.class,
                    _ => None,
                }
            }
        },
    );

    let session_resource = Resource::new(
        move || class_id.get(),
        |id| async move { get_active_class_session_fn(id).await },
    );

    let end_session_action = Action::new(move |session_id: &i64| {
        let id = *session_id;
        async move { end_class_session_fn(id).await }
    });
    let end_session_pending = end_session_action.pending();

    Effect::new(move |_| {
        if let Some(result) = end_session_action.value().get() {
            leptos::logging::log!("=== END SESSION RESPONSE ===");
            match result {
                Ok(response) => {
                    leptos::logging::log!("Success: {}", response.success);
                    leptos::logging::log!("Message: {}", response.message);

                    if response.success {
                        let dest = last_return_path.get();
                        leptos::logging::log!("Navigating to: {}", dest);

                        if !dest.is_empty() {
                            // Clone navigate for the timeout closure
                            let nav = navigate.clone();
                            // Add a small delay to ensure state updates
                            set_timeout(
                                move || {
                                    nav(&dest, Default::default());
                                },
                                std::time::Duration::from_millis(100),
                            );
                        }
                    } else {
                        leptos::logging::log!("❌ End session failed: {}", response.message);
                    }
                }
                Err(e) => {
                    leptos::logging::log!("❌ End session error: {}", e);
                }
            }
            leptos::logging::log!("===========================");
        }
    });

    view! {
        <section class="class-qr">
            <div class="qr-shell">
                <Suspense fallback=move || view! { <div class="loading">"Loading session..."</div> }>
                    {move || {
                        let origin_value = origin_signal.get();
                        class_resource.get().map(|maybe_class| {
                            match maybe_class {
                                Some(class) => {
                                    let return_path = build_return_path(origin_value.clone(), &class.module_code);
                                    last_return_path.set(return_path.clone());
                                    let (day_label, pretty_date) = format_date_label(&class.date);
                                    let (time_display, duration_display) = format_time_label(&class.time, class.duration_minutes.max(15));
                                    let time_display_for_meta = time_display.clone();
                                    let venue_display = class.venue.clone().unwrap_or_else(|| "TBA".to_string());
                                    let module_code = class.module_code.clone();
                                    let active_session = session_resource.get().and_then(|resp| match resp {
                                        Ok(response) => response.session.clone(),
                                        Err(e) => {
                                            leptos::logging::log!("Failed to load active session: {}", e);
                                            None
                                        }
                                    });
                                    let session_payload = active_session
                                        .as_ref()
                                        .map(|s| {
                                            let payload = format!("session:{}:class:{}", s.session_id, class.class_id);
                                            leptos::logging::log!("=== QR CODE DEBUG ===");
                                            leptos::logging::log!("Session ID: {}", s.session_id);
                                            leptos::logging::log!("Class ID: {}", class.class_id);
                                            leptos::logging::log!("Full Payload: {}", payload);
                                            leptos::logging::log!("====================");
                                            payload
                                        })
                                        .unwrap_or_else(|| {
                                            leptos::logging::log!("❌ NO ACTIVE SESSION - QR will be invalid");
                                            format!("class:{}:inactive", class.class_id)
                                        });
                                    let qr_svg = if active_session.is_some() {
                                        build_qr_svg(&session_payload, 220).unwrap_or_default()
                                    } else {
                                        String::new()
                                    };
                                    let download_url = if qr_svg.is_empty() {
                                        String::new()
                                    } else {
                                        format!("data:image/svg+xml;utf8,{}", encode(&qr_svg))
                                    };
                                    let image_url = if download_url.is_empty() {
                                        "data:image/svg+xml;utf8,%3Csvg xmlns='http://www.w3.org/2000/svg' width='220' height='220'%3E%3Crect width='100%25' height='100%25' fill='%23f1f5f9'/%3E%3C/svg%3E".to_string()
                                    } else {
                                        download_url.clone()
                                    };
                                    let session_label = active_session
                                        .as_ref()
                                        .map(|s| format!("Session ID: {}", s.session_id))
                                        .unwrap_or_else(|| "No active session".to_string());
                                    let session_started_label = active_session
                                        .as_ref()
                                        .map(|s| format!("Session started at {}", format_session_started(&s.started_at)))
                                        .unwrap_or_else(|| "Start the session to enable attendance tracking.".to_string());
                                    let origin_param = origin_value.clone().unwrap_or_else(|| "classes".to_string());
                                    let enlarge_href = format!("/classes/qr/large?id={}&origin={}", class.class_id, origin_param);
                                    let active_session_id = active_session.as_ref().map(|s| s.session_id);
                                    let session_is_active = active_session.is_some();
                                    let download_name = active_session.as_ref().map(|s| format!("session-{}.svg", s.session_id));
                                    let session_status_text = if session_is_active { "Active Session" } else { "No Active Session" };

                                    view! {
                                        <div class="qr-card">
                                            <div class="qr-header">
                                                <h1>{format!("{} – {}", module_code, class.title)}</h1>
                                                <p class="muted">{class.description.clone().unwrap_or_else(|| "Manage attendance for this session.".to_string())}</p>
                                            </div>

                                            <div class="qr-meta">
                                                <div class="meta-item">
                                                    <span class="icon">"📅"</span>
                                                    <div>
                                                        <div class="meta-label">{day_label}</div>
                                                        <div class="meta-value">{pretty_date}</div>
                                                    </div>
                                                </div>
                                                <div class="meta-item">
                                                    <span class="icon">"⏰"</span>
                                                    <div>
                                                        <div class="meta-label">"Start"</div>
                                                        <div class="meta-value">{time_display_for_meta}</div>
                                                    </div>
                                                </div>
                                                <div class="meta-item">
                                                    <span class="icon">"🕒"</span>
                                                    <div>
                                                        <div class="meta-label">"Duration"</div>
                                                        <div class="meta-value">{duration_display}</div>
                                                    </div>
                                                </div>
                                                <div class="meta-item">
                                                    <span class="icon">"📍"</span>
                                                    <div>
                                                        <div class="meta-label">"Room"</div>
                                                        <div class="meta-value">{venue_display}</div>
                                                    </div>
                                                </div>
                                            </div>

                                            <div class="qr-body">
                                                <p class="muted center">"Students can scan this QR code to check in"</p>
                                                <p class="session-id">{session_label.clone()}</p>
                                                <A href=enlarge_href attr:class="qr-image-link" attr:aria-label="View QR code full screen">
                                                    <div class="qr-image">
                                                        <img src=image_url.clone() alt="QR code for session" width="220" height="220"/>
                                                    </div>
                                                </A>
                                                <p class="session-status">
                                                    <span class="dot"></span>
                                                    {session_status_text}
                                                </p>
                                                <p class="session-start">{session_started_label}</p>
                                                
                                            </div>

                                            <div class="qr-actions">
                                                <A href=return_path.clone() attr:class="btn btn-outline">"Close"</A>
                                                {if let Some(session_id) = active_session_id {
                                                    let return_path_clone = return_path.clone();
                                                    view! {
                                                        <button
                                                            class="btn btn-danger"
                                                            disabled=move || end_session_pending.get()
                                                            on:click=move |_| {
                                                                leptos::logging::log!("🔴 END SESSION CLICKED");
                                                                leptos::logging::log!("Session ID: {}", session_id);
                                                                leptos::logging::log!("Return path: {}", return_path_clone.clone());
                                                                last_return_path.set(return_path_clone.clone());
                                                                end_session_action.dispatch(session_id);
                                                            }
                                                        >{move || if end_session_pending.get() { "Ending..." } else { "End Session" }}</button>
                                                    }.into_any()
                                                } else {
                                                    view! { <></> }.into_any()
                                                }}
                                            </div>
                                        </div>
                                    }.into_any()
                                }
                                None => view! { <div class="empty-state"><p>"Class not found."</p></div> }.into_any()
                            }
                        })
                    }}
                </Suspense>
            </div>
        </section>
    }
}

#[component]
pub fn ClassQrFullscreenPage() -> impl IntoView {
    let navigate = use_navigate();
    let query = use_query_map();
    let query_for_id = query.clone();
    let class_id = Signal::derive(move || {
        query_for_id
            .with(|q| q.get("id").and_then(|id| id.parse::<i64>().ok()))
            .unwrap_or(0)
    });

    let origin_signal =
        Signal::derive(move || query.with(|q| q.get("origin").map(|s| s.to_string())));

    let last_return_path = RwSignal::new(String::new());

    let class_resource = Resource::new(
        move || class_id.get(),
        |id| async move {
            if id == 0 {
                None
            } else {
                match get_class_fn(id).await {
                    Ok(response) if response.success => response.class,
                    _ => None,
                }
            }
        },
    );

    let session_resource = Resource::new(
        move || class_id.get(),
        |id| async move { get_active_class_session_fn(id).await },
    );

    let end_session_action = Action::new(move |session_id: &i64| {
        let id = *session_id;
        async move { end_class_session_fn(id).await }
    });
    let end_session_pending = end_session_action.pending();

    Effect::new(move |_| {
        if let Some(result) = end_session_action.value().get() {
            leptos::logging::log!("=== END SESSION RESPONSE (FULLSCREEN) ===");
            match result {
                Ok(response) => {
                    leptos::logging::log!("Success: {}", response.success);
                    leptos::logging::log!("Message: {}", response.message);

                    if response.success {
                        let dest = last_return_path.get();
                        leptos::logging::log!("Navigating to: {}", dest);

                        if !dest.is_empty() {
                            let nav = navigate.clone();
                            set_timeout(
                                move || {
                                    nav(&dest, Default::default());
                                },
                                std::time::Duration::from_millis(100),
                            );
                        }
                    }
                }
                Err(e) => {
                    leptos::logging::log!("❌ End session error: {}", e);
                }
            }
            leptos::logging::log!("===========================");
        }
    });

    view! {
        <section class="class-qr-fullscreen">
            <Suspense fallback=move || view! { <div class="loading">"Loading QR..."</div> }>
                {move || {
                    let origin_value = origin_signal.get();
                    class_resource.get().map(|maybe_class| {
                        match maybe_class {
                            Some(class) => {
                                let return_path = build_return_path(origin_value.clone(), &class.module_code);
                                last_return_path.set(return_path.clone());
                                let active_session = session_resource.get().and_then(|resp| match resp {
                                    Ok(response) => response.session.clone(),
                                    Err(e) => {
                                        leptos::logging::log!("Failed to load active session: {}", e);
                                        None
                                    }
                                });
                                let session_payload = active_session
                                    .as_ref()
                                    .map(|s| {
                                        let payload = format!("session:{}:class:{}", s.session_id, class.class_id);
                                        leptos::logging::log!("=== QR CODE DEBUG ===");
                                        leptos::logging::log!("Session ID: {}", s.session_id);
                                        leptos::logging::log!("Class ID: {}", class.class_id);
                                        leptos::logging::log!("Full Payload: {}", payload);
                                        leptos::logging::log!("====================");
                                        payload
                                    })
                                    .unwrap_or_else(|| {
                                        leptos::logging::log!("❌ NO ACTIVE SESSION - QR will be invalid");
                                        format!("class:{}:inactive", class.class_id)
                                    });
                                let qr_svg = if active_session.is_some() {
                                    build_qr_svg(&session_payload, 360).unwrap_or_default()
                                } else {
                                    String::new()
                                };
                                let image_url = if qr_svg.is_empty() {
                                    "data:image/svg+xml;utf8,%3Csvg xmlns='http://www.w3.org/2000/svg' width='360' height='360'%3E%3Crect width='100%25' height='100%25' fill='%23f1f5f9'/%3E%3C/svg%3E".to_string()
                                } else {
                                    format!("data:image/svg+xml;utf8,{}", encode(&qr_svg))
                                };
                                let origin_param = origin_value.clone().unwrap_or_else(|| "classes".to_string());
                                let qr_page_path = format!("/classes/qr?id={}&origin={}", class.class_id, origin_param);
                                let active_session_id = active_session.as_ref().map(|s| s.session_id);

                                view! {
                                    <div class="qr-full-card">
                                        <div class="qr-full-wrapper">
                                            <img src=image_url.clone() alt="QR code for session" width="360" height="360"/>
                                        </div>
                                        <div class="qr-full-actions">
                                            <A href=qr_page_path attr:class="btn btn-outline">"Close"</A>
                                            {if let Some(session_id) = active_session_id {
                                                let return_path_clone = return_path.clone();
                                                view! {
                                                    <button
                                                        class="btn btn-danger"
                                                        disabled=move || end_session_pending.get()
                                                        on:click=move |_| {
                                                            leptos::logging::log!("🔴 END SESSION CLICKED");
                                                            leptos::logging::log!("Session ID: {}", session_id);
                                                            leptos::logging::log!("Return path: {}", return_path_clone.clone());
                                                            last_return_path.set(return_path_clone.clone());
                                                            end_session_action.dispatch(session_id);
                                                        }
                                                    >{move || if end_session_pending.get() { "Ending..." } else { "End Session" }}</button>
                                                }.into_any()
                                            } else {
                                                view! { <></> }.into_any()
                                            }}
                                        </div>
                                    </div>
                                }.into_any()
                            }
                            None => view! { <div class="empty-state"><p>"Class not found."</p></div> }.into_any()
                        }
                    })
                }}
            </Suspense>
        </section>
    }
}
