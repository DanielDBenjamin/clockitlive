use crate::components::{Calendar, ClassList, Header, StatTile};
use crate::database::modules::ModuleWithStats;
use crate::routes::class_functions::get_lecturer_classes_fn;
use crate::routes::module_functions::{get_lecturer_modules_fn, get_tutor_modules_fn};
use crate::user_context::get_current_user;
use crate::utils::module_visuals::{module_visual, ModuleVisual};
use chrono::{Local, NaiveDate};
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

#[component]
pub fn HomePage() -> impl IntoView {
    let current_user = get_current_user();
    let refresh_trigger = RwSignal::new(0);

    let greeting = move || -> String {
        let user = current_user.get();
        match user {
            Some(user) => {
                let title = match user.role.as_str() {
                    "lecturer" => "",
                    "tutor" => "",
                    _ => "",
                };
                format!("Welcome back, {} {} {}", title, user.name, user.surname)
            }
            None => "Welcome back".to_string(),
        }
    };

    // Load modules based on user role
    let modules_resource = Resource::new(
        move || current_user.get(),
        |user| async move {
            match user {
                Some(user) => {
                    let response = match user.role.as_str() {
                        "tutor" => get_tutor_modules_fn(user.email_address.clone()).await,
                        _ => get_lecturer_modules_fn(user.email_address.clone()).await,
                    };
                    match response {
                        Ok(response) if response.success => Some(response.modules),
                        _ => None,
                    }
                },
                None => None,
            }
        },
    );

    // Load classes based on user role
    let classes_resource = Resource::new(
        move || (current_user.get(), refresh_trigger.get()),
        |(user, _)| async move {
            match user {
                Some(user) => {
                    match get_lecturer_classes_fn(user.email_address.clone()).await {
                        Ok(response) if response.success => Some(response.classes),
                        _ => None,
                    }
                }
                None => None,
            }
        },
    );

    let today_key = Local::now().format("%Y-%m-%d").to_string();

    let all_classes =
        Signal::derive(move || classes_resource.get().and_then(|c| c).unwrap_or_default());

    // Selected date for calendar
    let selected_date = RwSignal::new(today_key.clone());

    // Classes for selected date
    let filtered_classes = Signal::derive(move || {
        let date = selected_date.get();
        all_classes
            .get()
            .into_iter()
            .filter(|c| c.date == date)
            .collect::<Vec<_>>()
    });

    let todays_minutes = Signal::derive(move || {
        filtered_classes
            .get()
            .into_iter()
            .map(|c| c.duration_minutes.max(0))
            .sum::<i32>()
    });

    let selected_date_label = Signal::derive({
        let selected_date = selected_date;
        let today_key = today_key.clone();
        move || {
            let date_str = selected_date.get();
            if date_str == today_key {
                "Today's Classes".to_string()
            } else {
                NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .map(|date| format!("Classes for {}", date.format("%A, %d %B %Y")))
                    .unwrap_or_else(|_| "Classes".to_string())
            }
        }
    });

    let todays_hours_display = Signal::derive(move || {
        let minutes = todays_minutes.get();
        if minutes <= 0 {
            "0h".to_string()
        } else if minutes % 60 == 0 {
            format!("{}h", minutes / 60)
        } else {
            format!("{:.1}h", minutes as f64 / 60.0)
        }
    });

    let on_date_select = Callback::new(move |date: String| {
        selected_date.set(date);
    });

    // Calculate stats
    let total_students = Signal::derive(move || {
        modules_resource
            .get()
            .and_then(|modules| {
                modules
                    .as_ref()
                    .map(|m| m.iter().map(|mod_| mod_.student_count).sum::<i32>())
            })
            .unwrap_or(0)
            .to_string()
    });

    let total_classes_today = Signal::derive(move || filtered_classes.get().len().to_string());

    view! {
        <section class="home">
            <Header
                title=greeting
                subtitle="Manage your modules and schedule your classes".to_string()
            />

            <div class="dashboard-grid">
                <div class="home-left">
                    <div class="add-module-row">
                        <h3 class="heading">"Your Modules"</h3>
                        {move || {
                            current_user.get().map(|user| {
                                if user.role == "lecturer" {
                                    view! {
                                        <A href="/modules/new" attr:class="btn btn-primary btn-small">"+ Add Module"</A>
                                    }.into_any()
                                } else {
                                    view! { <></> }.into_any()
                                }
                            }).unwrap_or_else(|| view! { <></> }.into_any())
                        }}
                    </div>

                    <div class="home-modules-scroll">
                        <Suspense fallback=move || view! { <div class="loading">"Loading modules..."</div> }>
                            {move || {
                                modules_resource.get().map(|modules_opt| {
                                    match modules_opt {
                                        Some(modules) if !modules.is_empty() => {
                                            view! {
                                                <div class="modules-grid">
                                                    {modules.into_iter().map(|module| {
                                                        let user_role = current_user.get()
                                                            .map(|u| u.role.clone())
                                                            .unwrap_or_else(|| "lecturer".to_string());
                                                        view! { <DynamicModuleCard module=module user_role=user_role/> }
                                                    }).collect_view()}
                                                </div>
                                            }.into_any()
                                        }
                                        Some(_) => {
                                            view! {
                                                <div class="empty-state">
                                                    <p>"No modules yet. Create your first module to get started!"</p>
                                                </div>
                                            }.into_any()
                                        }
                                        None => {
                                            view! {
                                                <div class="empty-state">
                                                    <p>"Please log in to view your modules."</p>
                                                </div>
                                            }.into_any()
                                        }
                                    }
                                })
                            }}
                        </Suspense>
                    </div>

                    <div class="home-summary-stick">
                        <div class="stats-row home-summary-row">
                            <StatTile value=move || total_students.get() label="Total Students"/>
                            <StatTile value=move || total_classes_today.get() label="Classes Today"/>
                            <StatTile value=move || todays_hours_display.get() label="Teaching Hours"/>
                        </div>
                    </div>
                </div>

                <aside class="schedule-panel">
                    <div class="heading">
                        <span>"Schedule"</span>
                    </div>
                    <Calendar classes=all_classes on_date_select=on_date_select/>
                    <h3 class="heading" style="margin-top:16px;">{move || selected_date_label.get()}</h3>
                    <div class="classes-scroll-container">
                    <ClassList classes=filtered_classes/>
                    </div>
                </aside>
            </div>
        </section>
    }
}

#[component]
fn DynamicModuleCard(module: ModuleWithStats, user_role: String) -> impl IntoView {
    let navigate = use_navigate();

    let module_code = module.module_code.clone();
    let ModuleVisual { variant, label } = module_visual(&module_code);
    let icon_classes = format!("module-icon {}", variant);
    let module_code_display = module_code.clone();
    let student_count = module.student_count;
    let href = format!("/classes?module={}", module_code.clone());

    let go_card = {
        let href = href.clone();
        let navigate = navigate.clone();
        move |_| {
            navigate(&href, Default::default());
        }
    };

    let go_card_key = {
        let href = href.clone();
        let navigate = navigate.clone();
        move |e: leptos::ev::KeyboardEvent| {
            let k = e.key();
            if k == "Enter" || k == " " {
                navigate(&href, Default::default());
            }
        }
    };
    let value = navigate.clone();
    let go_new_class = {
        let module_code = module_code.clone();
        move |e: leptos::ev::MouseEvent| {
            e.stop_propagation();
            e.prevent_default();
            value(
                &format!("/classes/new?module={}", module_code),
                Default::default(),
            );
        }
    };

    let go_edit_module = {
        let module_code = module_code.clone();
        move |e: leptos::ev::MouseEvent| {
            e.stop_propagation();
            e.prevent_default();
            navigate(
                &format!("/modules/edit?code={}", module_code),
                Default::default(),
            );
        }
    };

    view! {
        <div class="module-card-link" role="link" tabindex="0" on:click=go_card on:keydown=go_card_key>
            <div class="card module-card">
                {if user_role == "lecturer" {
                    view! {
                        <button
                            class="module-edit-btn"
                            on:click=go_edit_module
                            title="Edit module"
                            aria-label="Edit module"
                        >
                            "✏️"
                        </button>
                    }.into_any()
                } else {
                    view! { <></> }.into_any()
                }}
                <div class=icon_classes aria-hidden="true">{label.clone()}</div>
                <div class="module-body">
                    <div class="module-code">{module_code_display}</div>
                    <div class="module-name">{module.module_title.clone()}</div>
                    <p class="module-desc">
                        {module.description.unwrap_or_else(|| "No description available".to_string())}
                    </p>
                    <div class="module-meta">
                        <span class="meta-left">
                            <span aria-hidden="true">"👥"</span>
                            <span class="muted">{student_count} " students"</span>
                        </span>
                        <button class="btn btn-primary btn-small" on:click=go_new_class>"+ Add Class"</button>
                    </div>
                </div>
            </div>
        </div>
    }
}
