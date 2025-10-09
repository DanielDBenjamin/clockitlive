use crate::database::classes::Class;
use crate::routes::class_functions::{
    delete_class_fn, get_active_class_session_fn, get_module_classes_fn, start_class_session_fn,
};
use crate::routes::module_functions::get_module_fn;
use crate::routes::student_functions::get_module_students;
use crate::user_context::get_current_user;
use leptos::prelude::*;
use leptos::server_fn::ServerFnError;
use leptos::web_sys::window;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_query_map};

#[component]
pub fn ClassesPage() -> impl IntoView {
    let query = use_query_map();
    let current_user = get_current_user();

    let module_code = Signal::derive(move || query.with(|q| q.get("module").unwrap_or_default()));

    // Load module details
    let module_resource = Resource::new(
        move || module_code.get(),
        |code| async move {
            if code.is_empty() {
                return None;
            }
            match get_module_fn(code.clone()).await {
                Ok(response) if response.success => response.module,
                _ => None,
            }
        },
    );

    // Load classes for the module - all users see all classes in the module
    let classes_resource = Resource::new(
        move || (current_user.get(), module_code.get()),
        |(user, code)| async move {
            if code.is_empty() {
                return None;
            }
            
            match user {
                Some(_user) => {
                    let response = get_module_classes_fn(code.clone()).await;
                    match response {
                        Ok(response) if response.success => Some(response.classes),
                        _ => None,
                    }
                }
                None => None,
            }
        },
    );

    // Load students enrolled in this module
    let students_resource = Resource::new(
        move || module_code.get(),
        |code| async move {
            if code.is_empty() {
                return None;
            }
            match get_module_students(code).await {
                Ok(response) if response.success => Some(response.students),
                _ => None,
            }
        },
    );

    // Local UI state for search and status filter
    let search_term = RwSignal::new(String::new());
    let status_filter = RwSignal::new("All".to_string());

    // Calculate stats
    let total_classes = Signal::derive(move || {
        classes_resource
            .get()
            .and_then(|c| c.as_ref().map(|classes| classes.len()))
            .unwrap_or(0)
    });

    let completed_classes = Signal::derive(move || {
        classes_resource
            .get()
            .and_then(|c| {
                c.as_ref()
                    .map(|classes| classes.iter().filter(|c| c.status == "completed").count())
            })
            .unwrap_or(0)
    });

    let upcoming_classes = Signal::derive(move || {
        classes_resource
            .get()
            .and_then(|c| {
                c.as_ref()
                    .map(|classes| classes.iter().filter(|c| c.status == "upcoming").count())
            })
            .unwrap_or(0)
    });

    // Derived count of enrolled students
    let enrolled_students = Signal::derive(move || {
        students_resource
            .get()
            .and_then(|s| s.as_ref().map(|v| v.len()))
            .unwrap_or(0)
    });

    view! {
        <section class="classes-page">
            <div class="classes-header">
                <div class="header-content">
                    <A href="/home" attr:class="link">"←"</A>
                    <div class="header-text">
                        <Suspense fallback=move || view! { <h1 class="page-title">"Loading..."</h1> }>
                            {move || module_resource.get().map(|module_opt| {
                                match module_opt {
                                    Some(module) => view! {
                                        <h1 class="page-title">{module.module_title.clone()}</h1>
                                        <p class="page-subtitle">{format!("{} • 2025", module.module_code)}</p>
                                    }.into_any(),
                                    None => view! {
                                        <h1 class="page-title">"Module Not Found"</h1>
                                        <p class="page-subtitle">{move || format!("{} • 2025", module_code.get())}</p>
                                    }.into_any()
                                }
                            })}
                        </Suspense>
                    </div>
                </div>
                <div class="header-actions">
                    <button class="btn btn-outline">"⭳ Export"</button>
                    <A href=move || format!("/classes/new?module={}", module_code.get()) attr:class="btn btn-primary">"+ Add Class"</A>
                </div>
            </div>

            <div class="stats-row">
                <div class="stat-tile">
                    <div class="stat-value">{move || total_classes.get().to_string()}</div>
                    <div class="stat-label">"Total Classes"</div>
                </div>
                <div class="stat-tile">
                    <div class="stat-value" style="color:#10b981;">{move || completed_classes.get().to_string()}</div>
                    <div class="stat-label">"Completed"</div>
                </div>
                <div class="stat-tile">
                    <div class="stat-value" style="color:#2563eb;">{move || upcoming_classes.get().to_string()}</div>
                    <div class="stat-label">"Upcoming"</div>
                </div>
                <div class="stat-tile">
                    <div class="stat-value">{move || enrolled_students.get().to_string()}</div>
                    <div class="stat-label">"Enrolled Students"</div>
                </div>
            </div>

            <Suspense fallback=move || view! { <div class="loading">"Loading classes..."</div> }>
                {move || {
                    classes_resource.get().map(|classes_opt| {
                        match classes_opt {
                            Some(classes) if !classes.is_empty() => {
                                view! {
                                    <div class="classes-section">
                                        <div class="section-header">
                                            <h3 class="heading">"Classes Schedule"</h3>
                                            <div class="search-controls">
                                                <input
                                                    class="input search-input"
                                                    placeholder="Search classes..."
                                                    bind:value=search_term
                                                />
                                                <select class="btn btn-outline" bind:value=status_filter>
                                                    <option value="All">"All"</option>
                                                    <option value="Upcoming">"Upcoming"</option>
                                                    <option value="In Progress">"In Progress"</option>
                                                    <option value="Completed">"Completed"</option>
                                                </select>
                                            </div>
                                        </div>

                                        <div class="classes-table-wrapper">
                                            <table class="classes-table">
                                                <thead>
                                                    <tr>
                                                        <th>"Class Title"</th>
                                                        <th>"Date"</th>
                                                        <th>"Time"</th>
                                                        <th>"Venue"</th>
                                                        <th>"Status"</th>
                                                        <th>"Actions"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {
                                                        // Build a derived filtered list that reacts to search + status
                                                        let filtered_classes = {
                                                            let classes_cloned = classes.clone();
                                                            Signal::derive(move || {
                                                                let query = search_term.get().to_lowercase();
                                                                let status = status_filter.get();
                                                                let status_token: Option<&'static str> = match status.as_str() {
                                                                    "Upcoming" => Some("upcoming"),
                                                                    "In Progress" => Some("in_progress"),
                                                                    "Completed" => Some("completed"),
                                                                    _ => None,
                                                                };

                                                                classes_cloned
                                                                    .iter()
                                                                    .cloned()
                                                                    .filter(|c| {
                                                                        let matches_search = if query.trim().is_empty() {
                                                                            true
                                                                        } else {
                                                                            let q = query.as_str();
                                                                            c.title.to_lowercase().contains(q)
                                                                                || c.module_code.to_lowercase().contains(q)
                                                                                || c.date.to_lowercase().contains(q)
                                                                                || c.time.to_lowercase().contains(q)
                                                                                || c.status.replace('_', " ").to_lowercase().contains(q)
                                                                                || c.venue.as_deref().unwrap_or("").to_lowercase().contains(q)
                                                                                || c.description.as_deref().unwrap_or("").to_lowercase().contains(q)
                                                                        };
                                                                        let matches_status = match status_token {
                                                                            Some(tok) => c.status == tok,
                                                                            None => true,
                                                                        };
                                                                        matches_search && matches_status
                                                                    })
                                                                    .collect::<Vec<_>>()
                                                            })
                                                        };

                                                        view! {
                                                            <Show
                                                                when=move || !filtered_classes.get().is_empty()
                                                                fallback=move || view! {
                                                                    <tr>
                                                                        <td colspan="6" style="text-align:center; padding:16px;" class="muted">
                                                                            {"No classes match your filters"}
                                                                        </td>
                                                                    </tr>
                                                                }
                                                            >
                                                                {move || filtered_classes.get()
                                                                    .into_iter()
                                                                    .map(|class| view! { <ClassRow class=class/> })
                                                                    .collect_view()
                                                                }
                                                            </Show>
                                                        }
                                                    }
                                                </tbody>
                                            </table>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                            Some(_) => {
                                view! {
                                    <div class="empty-state">
                                        <p>"No classes yet. Create your first class to get started!"</p>
                                    </div>
                                }.into_any()
                            }
                            None => {
                                view! {
                                    <div class="empty-state">
                                        <p>"Module not found or no classes available."</p>
                                    </div>
                                }.into_any()
                            }
                        }
                    })
                }}
            </Suspense>
        </section>
    }
}

#[component]
fn ClassRow(class: Class) -> impl IntoView {
    let class_id = class.class_id;
    let class_title = class.title.clone();
    let navigate = use_navigate();
    let current_status = RwSignal::new(class.status.clone());
    let show_delete_modal = RwSignal::new(false);

    let status_in_progress = Signal::derive(move || current_status.get() == "in_progress");
    let status_upcoming = Signal::derive(move || current_status.get() == "upcoming");
    let status_label = Signal::derive(move || match current_status.get().as_str() {
        "completed" => "Completed".to_string(),
        "in_progress" => "In Progress".to_string(),
        "upcoming" => "Upcoming".to_string(),
        _ => "Unknown".to_string(),
    });
    let badge_class_signal = Signal::derive(move || match current_status.get().as_str() {
        "completed" => "status-badge completed".to_string(),
        "in_progress" => "status-badge in-progress".to_string(),
        _ => "status-badge upcoming".to_string(),
    });

    // Delete action
    let delete_action = Action::new(move |id: &i64| {
        let id = *id;
        async move { delete_class_fn(id).await }
    });

    Effect::new(move |_| {
        if let Some(result) = delete_action.value().get() {
            match result {
                Ok(response) if response.success => {
                    _ = window().unwrap().location().reload();
                }
                Ok(response) => {
                    leptos::logging::log!("Delete failed: {}", response.message);
                }
                Err(e) => {
                    leptos::logging::log!("Delete error: {}", e);
                }
            }
        }
    });

    let on_delete_click = move |_| {
        show_delete_modal.set(true);
    };

    let on_confirm_delete = move |_| {
        delete_action.dispatch(class_id);
        show_delete_modal.set(false);
    };

    let on_cancel_delete = move |_| {
        show_delete_modal.set(false);
    };

    let start_session_href = format!("/classes/qr?id={}&origin=classes", class_id);
    let view_session_href = start_session_href.clone();
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
    let start_session_pending = start_session_action.pending();

    Effect::new({
        let current_status = current_status.clone();
        let navigate = navigate.clone();
        let href = start_session_href.clone();
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

    let duration_display = {
        let minutes = class.duration_minutes.max(15);
        if minutes % 60 == 0 {
            format!("{}h", minutes / 60)
        } else {
            format!("{} min", minutes)
        }
    };

    let location_status_display = location_status.clone();
    let location_error_display = location_error.clone();

    view! {
        <>
            <tr>
                <td>
                    <div class="class-cell">
                        <div class="class-title">{class.title.clone()}</div>

                    </div>
                </td>
                <td>
                    <div class="date-cell">
                        <div>{class.date.clone()}</div>
                    </div>
                </td>
                <td>
                    <div class="time-cell">
                        <div>{class.time.clone()}</div>
                        <div class="muted time-duration">{duration_display}</div>
                    </div>
                </td>
                <td>
                    <div class="venue-cell">
                        <div>{class.venue.clone().unwrap_or_else(|| "TBA".to_string())}</div>

                    </div>
                </td>
                <td>
                    <div class="status-cell">
                        <span class=move || badge_class_signal.get()>{move || status_label.get()}</span>
                    </div>
                </td>
                <td>
                    <div class="actions-cell">
                        <Show when=move || status_upcoming.get()>
                            <button
                                class="btn btn-primary start-session-link"
                                disabled=move || start_session_pending.get()
                                on:click=move |_| {
                                    if !start_session_pending.get() {
                                        start_session_action.dispatch(class_id);
                                    }
                                }
                            >{move || if start_session_pending.get() { "Starting..." } else { "Start Session" }}</button>
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
                            <A href=view_session_href.clone() attr:class="btn btn-outline alt start-session-link">"View Session"</A>
                        </Show>
                        <A href=format!("/classes/edit?id={}&origin=classes", class_id) attr:class="btn-icon edit">
                            <span>"✏"</span>
                            "Edit"
                        </A>
                        <button
                            class="btn-icon remove"
                            on:click=on_delete_click
                            disabled=move || delete_action.pending().get()
                        >
                            {move || if delete_action.pending().get() {
                                "⏳".to_string()
                            } else {
                                "🗑 Remove".to_string()
                            }}
                        </button>
                    </div>
                </td>
            </tr>

            <Show when=move || show_delete_modal.get()>
                <div class="modal-overlay" on:click=move |_| show_delete_modal.set(false)>
                    <div class="modal-content" on:click=|e| e.stop_propagation()>
                        <h2 class="modal-title">"Delete Class"</h2>
                        <p class="modal-text">
                            "Are you sure you want to delete "
                            <strong>{class_title.clone()}</strong>
                            " Class?"
                        </p>
                        <div class="modal-actions">
                            <button class="btn btn-outline" on:click=on_cancel_delete>"Cancel"</button>
                            <button class="btn btn-danger" on:click=on_confirm_delete>"Delete"</button>
                        </div>
                    </div>
                </div>
            </Show>
        </>
    }
}
