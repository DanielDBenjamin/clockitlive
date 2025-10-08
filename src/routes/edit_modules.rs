use crate::routes::module_functions::{get_module_fn, update_module_fn};
use crate::routes::student_functions::*;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_query_map};

#[component]
pub fn EditModule() -> impl IntoView {
    let navigate = use_navigate();
    let query = use_query_map();

    let module_code = Signal::derive(move || query.with(|q| q.get("code").unwrap_or_default()));

    let title = RwSignal::new(String::new());
    let desc = RwSignal::new(String::new());
    let message = RwSignal::new(String::new());
    let success = RwSignal::new(false);

    // Student management state
    let students = RwSignal::new(Vec::<StudentInfo>::new());
    let new_student_email = RwSignal::new(String::new());
    let show_add_modal = RwSignal::new(false);
    let show_import_modal = RwSignal::new(false);
    let show_delete_modal = RwSignal::new(false);
    let show_remove_student_modal = RwSignal::new(false);
    let student_to_remove = RwSignal::new(String::new());
    let student_name_to_remove = RwSignal::new(String::new());
    let csv_content = RwSignal::new(String::new());
    let student_message = RwSignal::new(String::new());

    // Tutor management state
    let tutors = RwSignal::new(Vec::<TutorInfo>::new());
    let new_tutor_email = RwSignal::new(String::new());
    let show_add_tutor_modal = RwSignal::new(false);
    let show_remove_tutor_modal = RwSignal::new(false);
    let tutor_to_remove = RwSignal::new(String::new());
    let tutor_name_to_remove = RwSignal::new(String::new());
    let tutor_message = RwSignal::new(String::new());

    // Load module data
    let module_resource = Resource::new(
        move || module_code.get(),
        |code| async move {
            if code.is_empty() {
                return None;
            }
            match get_module_fn(code).await {
                Ok(response) if response.success => response.module,
                _ => None,
            }
        },
    );

    // Load students
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

    // Load tutors
    let tutors_resource = Resource::new(
        move || module_code.get(),
        |code| async move {
            if code.is_empty() {
                return Vec::new();
            }
            match get_module_tutors(code).await {
                Ok(tutors) => tutors,
                _ => Vec::new(),
            }
        },
    );

    // Populate form when module loads
    Effect::new(move |_| {
        if let Some(Some(module)) = module_resource.get() {
            title.set(module.module_title.clone());
            desc.set(module.description.unwrap_or_default());
        }
    });

    // Populate students when they load
    Effect::new(move |_| {
        if let Some(Some(student_list)) = students_resource.get() {
            students.set(student_list);
        }
    });

    // Populate tutors when they load
    Effect::new(move |_| {
        let tutor_list = tutors_resource.get();
        tutors.set(tutor_list);
    });

    let update_action = Action::new(
        move |(code, title_val, desc_val): &(String, String, Option<String>)| {
            let code = code.clone();
            let title_val = title_val.clone();
            let desc_val = desc_val.clone();
            async move { update_module_fn(code, title_val, desc_val).await }
        },
    );

    let enroll_action = Action::new(move |request: &EnrollStudentRequest| {
        let request = request.clone();
        async move { enroll_student(request).await }
    });

    let bulk_enroll_action = Action::new(move |(module_code, emails): &(String, Vec<String>)| {
        let module_code = module_code.clone();
        let emails = emails.clone();
        async move { bulk_enroll_students(module_code, emails).await }
    });

    let unenroll_action = Action::new(move |(module_code, email): &(String, String)| {
        let module_code = module_code.clone();
        let email = email.clone();
        async move { unenroll_student(module_code, email).await }
    });

    // Tutor actions
    let enroll_tutor_action = Action::new(move |request: &EnrollTutorRequest| {
        let request = request.clone();
        async move { enroll_tutor(request).await }
    });

    let unenroll_tutor_action = Action::new(move |request: &EnrollTutorRequest| {
        let request = request.clone();
        async move { unenroll_tutor(request).await }
    });

    let delete_module_action = Action::new(move |module_code: &String| {
        let module_code = module_code.clone();
        async move { crate::routes::module_functions::delete_module_fn(module_code).await }
    });

    let on_submit = move |_| {
        message.set(String::new());
        success.set(false);

        if title.get().trim().is_empty() {
            message.set("Please enter a module title".to_string());
            return;
        }

        let code = module_code.get();
        let desc_val = if desc.get().trim().is_empty() {
            None
        } else {
            Some(desc.get())
        };

        update_action.dispatch((code, title.get(), desc_val));
    };

    // Handle delete module
    let on_delete_module = move |_| {
        show_delete_modal.set(true);
    };

    let on_confirm_delete = move |_| {
        delete_module_action.dispatch(module_code.get());
    };

    let on_cancel_delete = move |_| {
        show_delete_modal.set(false);
    };

    // Handle delete response
    Effect::new({
        let navigate = navigate.clone();
        move |_| {
            if let Some(result) = delete_module_action.value().get() {
                match result {
                    Ok(response) => {
                        if response.success {
                            let nav = navigate.clone();
                            set_timeout(
                                move || {
                                    nav("/home", Default::default());
                                },
                                std::time::Duration::from_millis(500),
                            );
                        } else {
                            message.set(response.message);
                            success.set(false);
                        }
                    }
                    Err(e) => {
                        message.set(format!("Error: {}", e));
                        success.set(false);
                    }
                }
            }
        }
    });

    // Handle update response
    Effect::new({
        let navigate = navigate.clone();
        move |_| {
            if let Some(result) = update_action.value().get() {
                match result {
                    Ok(response) => {
                        message.set(response.message);
                        success.set(response.success);

                        if response.success {
                            let nav = navigate.clone();
                            set_timeout(
                                move || {
                                    nav("/home", Default::default());
                                },
                                std::time::Duration::from_millis(1500),
                            );
                        }
                    }
                    Err(e) => {
                        message.set(format!("Error: {}", e));
                        success.set(false);
                    }
                }
            }
        }
    });

    // Handle adding single student
    let on_add_student = move |_| {
        let email = new_student_email.get().trim().to_lowercase();
        if email.is_empty() {
            student_message.set("Please enter a student email".to_string());
            return;
        }

        enroll_action.dispatch(EnrollStudentRequest {
            student_email: email,
            module_code: module_code.get(),
        });
    };

    // Handle enrollment response
    Effect::new(move |_| {
        if let Some(result) = enroll_action.value().get() {
            match result {
                Ok(response) => {
                    student_message.set(response.message.clone());

                    if response.success {
                        if let Some(student) = response.student {
                            students.update(|s| s.push(student));
                        }
                        new_student_email.set(String::new());
                    }
                    // Always close modal on response (success or failure)
                    show_add_modal.set(false);
                }
                Err(e) => {
                    student_message.set(format!("Error: {}", e));
                    show_add_modal.set(false);
                }
            }
        }
    });

    // Handle bulk import
    let on_import_csv = move |_| {
        let content = csv_content.get();
        if content.trim().is_empty() {
            student_message.set("Please paste CSV content".to_string());
            return;
        }

        let emails: Vec<String> = content
            .lines()
            .flat_map(|line| line.split(','))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s.contains('@'))
            .collect();

        if emails.is_empty() {
            student_message.set("No valid email addresses found".to_string());
            return;
        }

        bulk_enroll_action.dispatch((module_code.get(), emails));
    };

    // Handle bulk enrollment response
    Effect::new(move |_| {
        if let Some(result) = bulk_enroll_action.value().get() {
            match result {
                Ok(response) => {
                    student_message.set(response.message.clone());

                    if response.success {
                        csv_content.set(String::new());
                        show_import_modal.set(false);

                        // Reload students
                        let code = module_code.get();
                        leptos::task::spawn_local(async move {
                            if let Ok(response) = get_module_students(code).await {
                                if response.success {
                                    students.set(response.students);
                                }
                            }
                        });
                    }
                }
                Err(e) => {
                    student_message.set(format!("Error: {}", e));
                }
            }
        }
    });

    // Handle student removal - open confirmation modal
    let handle_remove = move |email: String, name: String| {
        leptos::logging::log!("=== HANDLE REMOVE CALLED ===");
        leptos::logging::log!("Email: {}", email);
        leptos::logging::log!("Name: {}", name);
        student_to_remove.set(email);
        student_name_to_remove.set(name);
        leptos::logging::log!("Setting show_remove_student_modal to true");
        show_remove_student_modal.set(true);
        leptos::logging::log!(
            "Modal should now be visible: {}",
            show_remove_student_modal.get()
        );
    };

    // Confirm student removal
    let on_confirm_remove_student = move |_| {
        let email = student_to_remove.get();
        let module = module_code.get();
        unenroll_action.dispatch((module, email));
        show_remove_student_modal.set(false);
    };

    let on_cancel_remove_student = move |_| {
        show_remove_student_modal.set(false);
    };

    // Handle unenroll response - COPY THE EXACT PATTERN FROM ENROLL!
    Effect::new(move |_| {
        if let Some(result) = unenroll_action.value().get() {
            match result {
                Ok(response) => {
                    student_message.set(response.message.clone());

                    if response.success {
                        // Reload the entire student list from server (same as bulk import does)
                        let code = module_code.get();
                        leptos::task::spawn_local(async move {
                            if let Ok(response) = get_module_students(code).await {
                                if response.success {
                                    students.set(response.students);
                                }
                            }
                        });
                    }
                }
                Err(e) => {
                    student_message.set(format!("Error: {}", e));
                }
            }
        }
    });

    // NOW the view starts here
    view! {
        <section class="edit-module">
            <div class="page-header" style="display:flex;align-items:center;gap:8px;">
                <A href="/home" attr:class="link">"←"</A>
                <h1 class="page-title">"Edit Module"</h1>
            </div>

            <Suspense fallback=move || view! { <div class="loading">"Loading module..."</div> }>
                {move || {
                    module_resource.get().map(move |_| {
                        view! {
                            <div class="form-card">
                                <h3 class="heading">"Module Information"</h3>

                                <label class="label" style="margin-top:6px;">"Module Code"</label>
                                <input
                                    class="input"
                                    type="text"
                                    value=move || module_code.get()
                                    disabled=true
                                />

                                <label class="label" style="margin-top:10px;">"Module Title "<span style="color:#ef4444;">"*"</span></label>
                                <input
                                    class="input"
                                    placeholder="e.g., Introduction to Programming"
                                    bind:value=title
                                />

                                <label class="label" style="margin-top:10px;">"Description"</label>
                                <textarea
                                    class="textarea"
                                    placeholder="Enter module description..."
                                    bind:value=desc
                                ></textarea>

                                <Show when=move || !message.get().is_empty()>
                                    <p class=move || if success.get() { "success center" } else { "error center" } style="margin-top:12px;">
                                        {message}
                                    </p>
                                </Show>

                                <div class="actions-row">
                                    <button
                                        class="btn btn-accent"
                                        on:click=on_submit
                                        disabled=move || update_action.pending().get()
                                    >
                                        {move || if update_action.pending().get() {
                                            "Saving...".into_view()
                                        } else {
                                            "Save Changes".into_view()
                                        }}
                                    </button>
                                    <button
                                        class="btn btn-danger"
                                        on:click=on_delete_module
                                    >
                                        "Delete Module"
                                    </button>
                                    <A href="/home" attr:class="btn btn-outline">"Cancel"</A>
                                </div>

                                <div class="divider"></div>

                                <div class="heading" style="display:flex; align-items:center; justify-content:space-between;">
                                    <span>"Student Management"</span>
                                    <div style="display:flex; gap:8px;">
                                        <button
                                            class="btn btn-outline"
                                            on:click=move |_| show_import_modal.set(true)
                                        >"⭳ Import Class List"</button>
                                        <button
                                            class="btn btn-accent"
                                            on:click=move |_| {
                                                new_student_email.set(String::new());
                                                student_message.set(String::new());
                                                show_add_modal.set(true);
                                            }
                                        >"+ Add Student"</button>
                                    </div>
                                </div>

                                <Show when=move || !student_message.get().is_empty()>
                                    <p class="success center" style="margin-top:8px;">
                                        {student_message}
                                    </p>
                                </Show>

                                <div class="card" style="padding:0; margin-top:10px;">
                                    <Show
                                        when=move || !students.get().is_empty()
                                        fallback=|| view! {
                                            <p style="padding:20px; text-align:center; color:#6b7280;">
                                                "No students enrolled yet. Add students to get started."
                                            </p>
                                        }
                                    >
                                        <table class="table">
                                            <thead>
                                                <tr>
                                                    <th>"Name"</th>
                                                    <th>"Email"</th>
                                                    <th>"Action"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {move || students.get().into_iter().map(|student| {
                                                    let email = student.email_address.clone();
                                                    let full_name = format!("{} {}", student.name, student.surname);
                                                    view! {
                                                        <tr>
                                                            <td>{full_name.clone()}</td>
                                                            <td>{email.clone()}</td>
                                                            <td>
                                                                <button
                                                                    class="btn btn-outline btn-small"
                                                                    style="color:#ef4444; border-color:#fecaca;"
                                                                    on:click=move |_| {
                                                                        leptos::logging::log!("Remove button clicked!");
                                                                        leptos::logging::log!("Email: {}, Name: {}", email, full_name);
                                                                        handle_remove(email.clone(), full_name.clone());
                                                                    }
                                                                >"🗑 Remove"</button>
                                                            </td>
                                                        </tr>
                                                    }
                                                }).collect_view()}
                                            </tbody>
                                        </table>
                                    </Show>
                                </div>
                            </div>

                            // Add Student Modal
                            <Show when=move || show_add_modal.get()>
                                <div class="modal-overlay" on:click=move |_| show_add_modal.set(false)>
                                    <div class="modal-content" on:click=|e| e.stop_propagation()>
                                        <h2 class="modal-title">"Add Student"</h2>
                                        <p class="modal-text">"Enter the student's email address:"</p>

                                        <input
                                            class="input"
                                            type="email"
                                            placeholder="student@university.ac.za"
                                            bind:value=new_student_email
                                            style="margin-bottom:16px;"
                                        />

                                        <div class="modal-actions">
                                            <button class="btn btn-outline" on:click=move |_| show_add_modal.set(false)>"Cancel"</button>
                                            <button
                                                class="btn btn-accent"
                                                on:click=on_add_student
                                                disabled=move || enroll_action.pending().get()
                                            >
                                                {move || if enroll_action.pending().get() {
                                                    "Adding...".into_view()
                                                } else {
                                                    "Add Student".into_view()
                                                }}
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            </Show>

                            // Import CSV Modal
                            <Show when=move || show_import_modal.get()>
                                <div class="modal-overlay" on:click=move |_| show_import_modal.set(false)>
                                    <div class="modal-content" on:click=|e| e.stop_propagation()>
                                        <h2 class="modal-title">"Import Class List"</h2>
                                        <p class="modal-text">"Paste email addresses (one per line or comma-separated):"</p>

                                        <textarea
                                            class="textarea"
                                            placeholder="student1@university.ac.za\nstudent2@university.ac.za"
                                            bind:value=csv_content
                                            style="margin-bottom:16px; min-height:200px;"
                                        ></textarea>

                                        <div class="modal-actions">
                                            <button class="btn btn-outline" on:click=move |_| show_import_modal.set(false)>"Cancel"</button>
                                            <button
                                                class="btn btn-accent"
                                                on:click=on_import_csv
                                                disabled=move || bulk_enroll_action.pending().get()
                                            >
                                                {move || if bulk_enroll_action.pending().get() {
                                                    "Importing...".into_view()
                                                } else {
                                                    "Import Students".into_view()
                                                }}
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            </Show>
                        }.into_any()
                    })
                }}

                // Remove Student Confirmation Modal
                <Show when=move || show_remove_student_modal.get()>
                    <div class="modal-overlay" on:click=move |_| show_remove_student_modal.set(false)>
                        <div class="modal-content" on:click=|e| e.stop_propagation()>
                            <h2 class="modal-title">"Remove Student?"</h2>
                            <p class="modal-text">
                                "Are you sure you want to remove "
                                <strong>{move || student_name_to_remove.get()}</strong>
                                " from this module?"
                            </p>

                            <div class="modal-actions">
                                <button class="btn btn-outline" on:click=on_cancel_remove_student>"Cancel"</button>
                                <button
                                    class="btn btn-danger"
                                    on:click=on_confirm_remove_student
                                    disabled=move || unenroll_action.pending().get()
                                >
                                    {move || if unenroll_action.pending().get() {
                                        "Removing...".into_view()
                                    } else {
                                        "Remove Student".into_view()
                                    }}
                                </button>
                            </div>
                        </div>
                    </div>
                </Show>

                // Delete Confirmation Modal
                <Show when=move || show_delete_modal.get()>
                    <div class="modal-overlay" on:click=move |_| show_delete_modal.set(false)>
                        <div class="modal-content" on:click=|e| e.stop_propagation()>
                            <h2 class="modal-title">"Delete Module?"</h2>
                            <p class="modal-text">
                                "Are you sure you want to delete this module? This action cannot be undone and will remove all associated data."
                            </p>

                            <div class="modal-actions">
                                <button class="btn btn-outline" on:click=on_cancel_delete>"Cancel"</button>
                                <button
                                    class="btn btn-danger"
                                    on:click=on_confirm_delete
                                    disabled=move || delete_module_action.pending().get()
                                >
                                    {move || if delete_module_action.pending().get() {
                                        "Deleting...".into_view()
                                    } else {
                                        "Delete Module".into_view()
                                    }}
                                </button>
                            </div>
                        </div>
                    </div>
                </Show>
            </Suspense>
        </section>
    }
}
