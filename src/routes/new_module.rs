use crate::routes::module_functions::create_module_fn;
use crate::routes::student_functions::*;
use crate::user_context::get_current_user;
use leptos::prelude::*;
use leptos::web_sys;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

#[component]
pub fn NewModule() -> impl IntoView {
    let current_user = get_current_user();
    let navigate = use_navigate();

    // Check if user is a tutor and redirect if so
    Effect::new(move || {
        if let Some(user) = current_user.get() {
            if user.role == "tutor" {
                navigate("/home", Default::default());
            }
        }
    });

    let module_code = RwSignal::new(String::new());
    let title = RwSignal::new(String::new());
    let desc = RwSignal::new(String::new());
    let message = RwSignal::new(String::new());
    let success = RwSignal::new(false);

    // Student management state
    let students = RwSignal::new(Vec::<StudentInfo>::new());
    let new_student_email = RwSignal::new(String::new());
    let show_add_modal = RwSignal::new(false);
    let show_import_modal = RwSignal::new(false);
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
    let tutor_message_success = RwSignal::new(true);

    let created_module_code = RwSignal::new(String::new());

    let create_action = Action::new(
        move |(code, title_val, desc_val, email): &(String, String, Option<String>, String)| {
            let code = code.clone();
            let title_val = title_val.clone();
            let desc_val = desc_val.clone();
            let email = email.clone();
            async move { create_module_fn(code, title_val, desc_val, email).await }
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

    let enroll_tutor_action = Action::new(move |request: &EnrollTutorRequest| {
        let request = request.clone();
        async move { enroll_tutor(request).await }
    });

    let unenroll_tutor_action = Action::new(move |request: &EnrollTutorRequest| {
        let request = request.clone();
        async move { unenroll_tutor(request).await }
    });

    // Handle module creation
    let on_submit = move |_| {
        message.set(String::new());
        success.set(false);

        let code = module_code.get().trim().to_string();
        if code.is_empty() {
            message.set("Please enter a module code".to_string());
            success.set(false);
            return;
        }

        if title.get().trim().is_empty() {
            message.set("Please enter a module title".to_string());
            success.set(false);
            return;
        }

        let email = match current_user.get() {
            Some(user) => {
                web_sys::console::log_1(
                    &format!("Creating module for email: {}", user.email_address).into(),
                );
                user.email_address
            }
            None => {
                message.set("You must be logged in to create a module".to_string());
                success.set(false);
                return;
            }
        };

        let desc_val = if desc.get().trim().is_empty() {
            None
        } else {
            Some(desc.get())
        };

        created_module_code.set(code.clone());
        create_action.dispatch((code, title.get(), desc_val, email));
    };

    // Handle module creation response
    Effect::new({
        move |_| {
            if let Some(result) = create_action.value().get() {
                match result {
                    Ok(response) => {
                        message.set(response.message.clone());
                        success.set(response.success);

                        // Keep form open to allow adding students
                        // User will click "Done" button when finished
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

        let module = created_module_code.get();
        if module.is_empty() {
            student_message.set("Please create the module first".to_string());
            return;
        }

        enroll_action.dispatch(EnrollStudentRequest {
            student_email: email,
            module_code: module,
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
                    // Always close modal on response
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

        let module = created_module_code.get();
        if module.is_empty() {
            student_message.set("Please create the module first".to_string());
            return;
        }

        // Parse CSV - expect email addresses, one per line or comma-separated
        let emails: Vec<String> = content
            .lines()
            .flat_map(|line| line.split(','))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s.contains('@'))
            .collect();

        if emails.is_empty() {
            student_message.set("No valid email addresses found in CSV".to_string());
            return;
        }

        bulk_enroll_action.dispatch((module, emails));
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

                        // Reload students list
                        let module = created_module_code.get();
                        if !module.is_empty() {
                            leptos::task::spawn_local(async move {
                                if let Ok(response) = get_module_students(module).await {
                                    if response.success {
                                        students.set(response.students);
                                    }
                                }
                            });
                        }
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
        student_to_remove.set(email);
        student_name_to_remove.set(name);
        show_remove_student_modal.set(true);
    };

    // Confirm student removal
    let on_confirm_remove_student = move |_| {
        let email = student_to_remove.get();
        let module = created_module_code.get();
        unenroll_action.dispatch((module, email));
        show_remove_student_modal.set(false);
    };

    let on_cancel_remove_student = move |_| {
        show_remove_student_modal.set(false);
    };

    // Handle unenroll response - FIXED to reload list properly
    Effect::new(move |_| {
        if let Some(result) = unenroll_action.value().get() {
            match result {
                Ok(response) => {
                    student_message.set(response.message.clone());

                    if response.success {
                        // Reload the entire student list from server
                        let module = created_module_code.get();
                        leptos::task::spawn_local(async move {
                            if let Ok(response) = get_module_students(module).await {
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

    // Tutor management handlers
    let handle_add_tutor = move |_: leptos::ev::MouseEvent| {
        tutor_message.set(String::new());
        tutor_message_success.set(true);
        new_tutor_email.set(String::new());
        show_add_tutor_modal.set(true);
    };

    let on_add_tutor = move |_: leptos::ev::MouseEvent| {
        let email = new_tutor_email.get().trim().to_lowercase();
        if email.is_empty() {
            tutor_message.set("Please enter a tutor email".to_string());
            tutor_message_success.set(false);
            return;
        }

        let module = created_module_code.get();
        if module.is_empty() {
            tutor_message.set("Please create the module first".to_string());
            tutor_message_success.set(false);
            return;
        }

        enroll_tutor_action.dispatch(EnrollTutorRequest {
            tutor_email: email,
            module_code: module,
        });
    };

    Effect::new(move |_| {
        if let Some(result) = enroll_tutor_action.value().get() {
            match result {
                Ok(response) => {
                    tutor_message.set(response.message.clone());
                    tutor_message_success.set(response.success);

                    if response.success {
                        new_tutor_email.set(String::new());
                        show_add_tutor_modal.set(false);

                        let module = created_module_code.get();
                        if !module.is_empty() {
                            leptos::task::spawn_local(async move {
                                if let Ok(list) = get_module_tutors(module).await {
                                    tutors.set(list);
                                }
                            });
                        }
                    }
                }
                Err(e) => {
                    tutor_message.set(format!("Error: {}", e));
                    tutor_message_success.set(false);
                }
            }
        }
    });

    let handle_remove_tutor = move |email: String, name: String| {
        tutor_to_remove.set(email);
        tutor_name_to_remove.set(name);
        show_remove_tutor_modal.set(true);
    };

    let on_confirm_remove_tutor = move |_: leptos::ev::MouseEvent| {
        let module = created_module_code.get();
        if module.is_empty() {
            tutor_message.set("Please create the module first".to_string());
            tutor_message_success.set(false);
            show_remove_tutor_modal.set(false);
            return;
        }

        let request = EnrollTutorRequest {
            module_code: module,
            tutor_email: tutor_to_remove.get(),
        };
        unenroll_tutor_action.dispatch(request);
        show_remove_tutor_modal.set(false);
    };

    let on_cancel_remove_tutor = move |_| {
        show_remove_tutor_modal.set(false);
    };

    Effect::new(move |_| {
        if let Some(result) = unenroll_tutor_action.value().get() {
            match result {
                Ok(response) => {
                    tutor_message.set(response.message.clone());
                    tutor_message_success.set(response.success);

                    if response.success {
                        let module = created_module_code.get();
                        if !module.is_empty() {
                            leptos::task::spawn_local(async move {
                                if let Ok(list) = get_module_tutors(module).await {
                                    tutors.set(list);
                                }
                            });
                        }
                    }
                }
                Err(e) => {
                    tutor_message.set(format!("Error: {}", e));
                    tutor_message_success.set(false);
                }
            }
        }
    });

    view! {
        <section class="new-module">
            <div class="page-header" style="display:flex;align-items:center;gap:8px;">
                <A href="/home" attr:class="link">"←"</A>
                <h1 class="page-title">"New Module"</h1>
                <p class="page-subtitle" style="margin-left:8px;">"Create a new module and manage student enrollment"</p>
            </div>

            <div class="form-card">
                <h3 class="heading">"Module Information"</h3>

                <label class="label" style="margin-top:6px;">"Module Code "<span style="color:#ef4444;">"*"</span></label>
                <input
                    class="input"
                    type="text"
                    placeholder="e.g., CS112 or MATH201"
                    bind:value=module_code
                    disabled=move || !created_module_code.get().is_empty()
                />

                <label class="label" style="margin-top:10px;">"Module Title "<span style="color:#ef4444;">"*"</span></label>
                <input
                    class="input"
                    placeholder="e.g., Introduction to Programming"
                    bind:value=title
                    disabled=move || !created_module_code.get().is_empty()
                />

                <label class="label" style="margin-top:10px;">"Description"</label>
                <textarea
                    class="textarea"
                    placeholder="Enter module description..."
                    bind:value=desc
                    disabled=move || !created_module_code.get().is_empty()
                ></textarea>

                <Show when=move || !message.get().is_empty()>
                    <p class=move || if success.get() { "success center" } else { "error center" } style="margin-top:12px;">
                        {message}
                    </p>
                </Show>

                <Show when=move || created_module_code.get().is_empty()>
                    <div class="actions-row">
                        <button
                            class="btn btn-accent"
                            on:click=on_submit
                            disabled=move || create_action.pending().get()
                        >
                            {move || if create_action.pending().get() {
                                "Creating Module...".into_view()
                            } else {
                                "Create Module".into_view()
                            }}
                        </button>
                        <A href="/home" attr:class="btn btn-outline">"Cancel"</A>
                    </div>
                </Show>

                <Show when=move || !created_module_code.get().is_empty()>
                    <>
                        <div class="divider"></div>

                        // Tutor Management Section
                        <div class="heading" style="display:flex; align-items:center; justify-content:space-between;">
                            <span>"Tutor Management"</span>
                            <div style="display:flex; gap:8px;">
                                <button class="btn btn-accent" on:click=handle_add_tutor>"+ Add Tutor"</button>
                            </div>
                        </div>

                        <Show when=move || !tutor_message.get().is_empty()>
                            <p
                                class=move || if tutor_message_success.get() { "success center" } else { "error center" }
                                style="margin-top:8px;"
                            >
                                {tutor_message}
                            </p>
                        </Show>

                        <div class="card" style="padding:0; margin-top:10px;">
                            <Show
                                when=move || !tutors.get().is_empty()
                                fallback=|| view! {
                                    <p style="padding:20px; text-align:center; color:#6b7280;">
                                        "No tutors assigned yet. Add tutors to collaborate on this module."
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
                                        {move || tutors.get().into_iter().map(|tutor| {
                                            let email = tutor.email_address.clone();
                                            let full_name = format!("{} {}", tutor.name, tutor.surname);
                                            view! {
                                                <tr>
                                                    <td>{full_name.clone()}</td>
                                                    <td>{email.clone()}</td>
                                                    <td>
                                                        <button
                                                            class="btn btn-outline btn-small"
                                                            style="color:#ef4444; border-color:#fecaca;"
                                                            on:click=move |_| {
                                                                handle_remove_tutor(email.clone(), full_name.clone());
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

                        <div class="actions-row">
                            <A href="/home" attr:class="btn btn-accent">"✓ Done"</A>
                        </div>
                    </>
                </Show>
            </div>

            // Add Student Modal
            <Show when=move || show_add_modal.get()>
                <div class="modal-overlay" on:click=move |_| show_add_modal.set(false)>
                    <div class="modal-content" on:click=|e| e.stop_propagation()>
                        <h2 class="modal-title">"Add Student"</h2>
                        <p class="modal-text">"Enter the student's email address to enroll them in this module."</p>

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
                            placeholder="student1@university.ac.za\nstudent2@university.ac.za\nstudent3@university.ac.za"
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
            // Add Tutor Modal
            <Show when=move || show_add_tutor_modal.get()>
                <div class="modal-overlay" on:click=move |_| show_add_tutor_modal.set(false)>
                    <div class="modal-content" on:click=|e| e.stop_propagation()>
                        <h2 class="modal-title">"Add Tutor"</h2>
                        <p class="modal-text">"Enter the tutor's email address to assign them to this module."</p>

                        <input
                            class="input"
                            type="email"
                            placeholder="tutor@university.ac.za"
                            bind:value=new_tutor_email
                            style="margin-bottom:16px;"
                        />

                        <div class="modal-actions">
                            <button class="btn btn-outline" on:click=move |_| show_add_tutor_modal.set(false)>"Cancel"</button>
                            <button
                                class="btn btn-accent"
                                on:click=on_add_tutor
                                disabled=move || enroll_tutor_action.pending().get()
                            >
                                {move || if enroll_tutor_action.pending().get() {
                                    "Adding...".into_view()
                                } else {
                                    "Add Tutor".into_view()
                                }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            // Remove Tutor Confirmation Modal
            <Show when=move || show_remove_tutor_modal.get()>
                <div class="modal-overlay" on:click=move |_| show_remove_tutor_modal.set(false)>
                    <div class="modal-content" on:click=|e| e.stop_propagation()>
                        <h2 class="modal-title">"Remove Tutor?"</h2>
                        <p class="modal-text">
                            "Are you sure you want to remove "
                            <strong>{move || tutor_name_to_remove.get()}</strong>
                            " from this module?"
                        </p>

                        <div class="modal-actions">
                            <button class="btn btn-outline" on:click=on_cancel_remove_tutor>"Cancel"</button>
                            <button
                                class="btn btn-danger"
                                on:click=on_confirm_remove_tutor
                                disabled=move || unenroll_tutor_action.pending().get()
                            >
                                {move || if unenroll_tutor_action.pending().get() {
                                    "Removing...".into_view()
                                } else {
                                    "Remove Tutor".into_view()
                                }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

        </section>
    }
}
