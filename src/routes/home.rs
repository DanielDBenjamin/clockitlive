use leptos::prelude::*;
use crate::components::{Header, Calendar, ClassList, StatTile};
use crate::user_context::get_current_user;
use crate::routes::module_functions::get_lecturer_modules_fn;
use crate::routes::class_functions::get_lecturer_classes_fn;
use crate::database::modules::ModuleWithStats;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use chrono::Local;

#[component]
pub fn HomePage() -> impl IntoView {
    let current_user = get_current_user();

    let greeting = move || -> String {
        let user = current_user.get();
        match user {
            Some(user) => {
                let title = match user.role.as_str() {
                    "lecturer" => "Dr.",
                    "tutor" => "Mr./Ms.",
                    _ => "",
                };
                format!("Welcome back, {} {} {}", title, user.name, user.surname)
            }
            None => "Welcome back".to_string(),
        }
    };

    // Load modules
    let modules_resource = Resource::new(
        move || current_user.get().map(|u| u.email_address.clone()),
        |email| async move {
            match email {
                Some(email) => {
                    match get_lecturer_modules_fn(email).await {
                        Ok(response) if response.success => Some(response.modules),
                        _ => None,
                    }
                }
                None => None,
            }
        },
    );

    // Load all classes for the lecturer
    let classes_resource = Resource::new(
        move || current_user.get().map(|u| u.email_address.clone()),
        |email| async move {
            match email {
                Some(email) => {
                    match get_lecturer_classes_fn(email).await {
                        Ok(response) if response.success => Some(response.classes),
                        _ => None,
                    }
                }
                None => None,
            }
        },
    );

    let all_classes = Signal::derive(move || {
        classes_resource.get()
            .and_then(|c| c)
            .unwrap_or_default()
    });

    // Selected date for calendar
    let selected_date = RwSignal::new(Local::now().format("%Y-%m-%d").to_string());
    
    // Classes for selected date
    let filtered_classes = Signal::derive(move || {
        let date = selected_date.get();
        all_classes.get().into_iter()
            .filter(|c| c.date == date)
            .collect::<Vec<_>>()
    });

    let on_date_select = Callback::new(move |date: String| {
        selected_date.set(date);
    });

    // Calculate stats
    let total_students = Signal::derive(move || {
        modules_resource.get()
            .and_then(|modules| modules.as_ref().map(|m| {
                m.iter().map(|mod_| mod_.student_count).sum::<i32>()
            }))
            .unwrap_or(0)
            .to_string()
    });

    let total_classes = Signal::derive(move || {
        modules_resource.get()
            .and_then(|modules| modules.as_ref().map(|m| {
                m.iter().map(|mod_| mod_.class_count).sum::<i32>()
            }))
            .unwrap_or(0)
            .to_string()
    });

    view! {
        <section class="home">
            <Header
                title=greeting
                subtitle="Manage your modules and schedule your classes".to_string()
            />

            <div class="dashboard-grid">
                <div>
                    <div class="add-module-row">
                        <h3 class="heading">"Your Modules"</h3>
                        <A href="/modules/new" attr:class="btn btn-outline btn-small">"+ Add Module"</A>
                    </div>

                    <Suspense fallback=move || view! { <div class="loading">"Loading modules..."</div> }>
                        {move || {
                            modules_resource.get().map(|modules_opt| {
                                match modules_opt {
                                    Some(modules) if !modules.is_empty() => {
                                        view! {
                                            <div class="modules-grid">
                                                {modules.into_iter().map(|module| {
                                                    view! { <DynamicModuleCard module=module/> }
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

                    <div class="stats-row" style="margin-top:16px;">
                        <StatTile value=move || total_students.get() label="Total Students"/>
                        <StatTile value=move || total_classes.get() label="Total Classes"/>
                        <StatTile value=move || "24h".to_string() label="Teaching Hours"/>
                    </div>
                </div>

                <aside class="schedule-panel">
                    <div class="heading">
                        <span>"Schedule"</span>
                    </div>
                    <Calendar classes=all_classes on_date_select=on_date_select/>
                    <h3 class="heading" style="margin-top:16px;">"Classes for " {move || selected_date.get()}</h3>
                    <ClassList classes=filtered_classes/>
                </aside>
            </div>
        </section>
    }
}

#[component]
fn DynamicModuleCard(module: ModuleWithStats) -> impl IntoView {
    let navigate = use_navigate();
    
    let hash = module.module_code.chars().map(|c| c as u32).sum::<u32>();
    let (icon, variant) = match hash % 4 {
        0 => ("</>" , "mod-purp"),
        1 => ("🗄️", "mod-blue"),
        2 => ("🧩", "mod-orange"),
        _ => ("🍃", "mod-green"),
    };
    
    let icon_classes = format!("module-icon {}", variant);
    let module_code_display = module.module_code.clone();
    let href = format!("/classes?module={}", module.module_code);
    
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
    
    let go_new_class = {
        let module_code = module.module_code.clone();
        move |e: leptos::ev::MouseEvent| {
            e.stop_propagation();
            e.prevent_default();
            navigate(&format!("/classes/new?module={}", module_code), Default::default());
        }
    };

    view! {
        <div class="module-card-link" role="link" tabindex="0" on:click=go_card on:keydown=go_card_key>
            <div class="card module-card">
                <div class=icon_classes aria-hidden="true">{icon}</div>
                <div class="module-body">
                    <div class="module-code">{module_code_display}</div>
                    <div class="module-name">{module.module_title.clone()}</div>
                    <p class="module-desc">
                        {module.description.unwrap_or_else(|| "No description available".to_string())}
                    </p>
                    <div class="module-meta">
                        <span class="meta-left">
                            <span aria-hidden="true">"👥"</span>
                            <span class="muted">{module.student_count} " students"</span>
                        </span>
                        <button class="btn btn-primary btn-small" on:click=go_new_class>"+ Add Class"</button>
                    </div>
                </div>
            </div>
        </div>
    }
}