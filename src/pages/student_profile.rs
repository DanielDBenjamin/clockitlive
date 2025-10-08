use crate::user_context::{clear_current_user, get_current_user};
use crate::routes::auth_functions::ResetPassword;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use urlencoding::encode;
use leptos_router::components::A;

fn format_role(role: &str) -> String {
    match role {
        "student" => "Student".to_string(),
        "lecturer" => "Lecturer".to_string(),
        "tutor" => "Tutor".to_string(),
        other => {
            let mut chars = other.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => "Student".to_string(),
            }
        }
    }
}

#[component]
pub fn StudentProfilePage() -> impl IntoView {
    let navigate = use_navigate();
    let current_user = get_current_user();

    // Reset password state
    let show_reset_modal = RwSignal::new(false);
    let new_password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());
    let reset_message = RwSignal::new(String::new());
    let reset_success = RwSignal::new(false);
    let show_new_password = RwSignal::new(false);
    let show_confirm_password = RwSignal::new(false);

    let navigate_back = navigate.clone();
    let go_back = move |_| {
        navigate_back("/student/home", Default::default());
    };
    let value = navigate.clone();
    let go_to_edit = move |_| {
        value("/student/profile/edit", Default::default());
    };

    // Handle sign out
    let navigate_logout = navigate.clone();
    let handle_sign_out = move |_| {
        // Clear the current user from context and localStorage
        clear_current_user();
        // Redirect to login page
        navigate_logout("/", Default::default());
    };

    // Reset password action
    let reset_action = ServerAction::<ResetPassword>::new();
    let reset_pending = reset_action.pending();

    let open_reset_modal = move |_| {
        show_reset_modal.set(true);
        new_password.set(String::new());
        confirm_password.set(String::new());
        reset_message.set(String::new());
        reset_success.set(false);
        show_new_password.set(false);
        show_confirm_password.set(false);
    };

    let close_reset_modal = move |_| {
        show_reset_modal.set(false);
        new_password.set(String::new());
        confirm_password.set(String::new());
        reset_message.set(String::new());
        reset_success.set(false);
        show_new_password.set(false);
        show_confirm_password.set(false);
    };

    let toggle_new_password = move |_| {
        show_new_password.set(!show_new_password.get());
    };

    let toggle_confirm_password = move |_| {
        show_confirm_password.set(!show_confirm_password.get());
    };

    let handle_reset_submit = move |_| {
        reset_message.set(String::new());
        reset_success.set(false);

        let email = match current_user.get() {
            Some(user) => user.email_address,
            None => {
                reset_message.set("User not found".to_string());
                return;
            }
        };

        reset_action.dispatch(ResetPassword {
            email,
            new_password: new_password.get(),
            confirm_password: confirm_password.get(),
        });
    };

    // Handle reset password response
    let navigate_redirect = navigate.clone();
    Effect::new(move |_| {
        if let Some(result) = reset_action.value().get() {
            match result {
                Ok(response) => {
                    reset_message.set(response.message.clone());
                    reset_success.set(response.success);
                    if response.success {
                        new_password.set(String::new());
                        confirm_password.set(String::new());
                        // Redirect to profile page after 1.5 seconds
                        let nav = navigate_redirect.clone();
                        set_timeout(
                            move || {
                                show_reset_modal.set(false);
                                nav("/student/profile", Default::default());
                            },
                            std::time::Duration::from_millis(1500),
                        );
                    }
                }
                Err(e) => {
                    reset_message.set(format!("Error: {}", e));
                    reset_success.set(false);
                }
            }
        }
    });

    // Get user info from context
    let user_name = move || {
        current_user
            .get()
            .map(|u| format!("{} {}", u.name, u.surname))
            .unwrap_or_else(|| "Student".to_string())
    };

    let user_email = move || {
        current_user
            .get()
            .map(|u| u.email_address.clone())
            .unwrap_or_else(|| "student@example.com".to_string())
    };

    let user_id = move || {
        current_user
            .get()
            .map(|u| format!("STU-{:06}", u.user_id))
            .unwrap_or_else(|| "STU-000000".to_string())
    };

    let user_role = move || {
        current_user
            .get()
            .map(|u| format_role(&u.role))
            .unwrap_or_else(|| "Student".to_string())
    };

    let avatar_url = move || {
        current_user.get().map(|u| {
            let full_name = format!("{} {}", u.name, u.surname);
            let encoded = encode(&full_name);
            format!(
                "https://ui-avatars.com/api/?name={}&background=14b8a6&color=ffffff&format=svg",
                encoded
            )
        })
    };

    view! {
        <div class="student-profile-container">
            {/* Header */}
            <header class="student-profile-header">
                <button class="student-back-btn" on:click=go_back>
                    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M19 12H5M12 19l-7-7 7-7"/>
                    </svg>
                </button>
                <h1 class="student-profile-title">"Profile"</h1>
                <button class="student-settings-btn" on:click=go_to_edit>
                    <svg
                        width="24"
                        height="24"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                    >
                        <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                        <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                    </svg>
                </button>
            </header>

            {/* Scrollable Content */}
            <div class="student-profile-content">
                {/* Profile Avatar Section */}
                <div class="student-profile-avatar-section">
                    <div class="student-profile-avatar-container">
                        <img
                            prop:src=move || avatar_url().unwrap_or_else(|| "/logo.png".to_string())
                            alt=user_name
                            class="student-profile-avatar-img"
                        />
                    </div>
                    <h2 class="student-profile-name">{user_name}</h2>
                    <p class="student-profile-subtitle">{user_role}</p>
                </div>

                {/* Personal Information */}
                <section class="student-profile-section">
                    <h3 class="student-section-title">
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
                            <circle cx="12" cy="7" r="4"></circle>
                        </svg>
                        "Personal Information"
                    </h3>

                    <div class="student-info-card">
                        <div class="student-info-icon student-info-icon-blue">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <rect x="3" y="4" width="18" height="16" rx="2"></rect>
                                <path d="M7 8h10M7 12h6"></path>
                            </svg>
                        </div>
                        <div class="student-info-text">
                            <div class="student-info-label">"Student ID"</div>
                            <div class="student-info-value">{user_id}</div>
                        </div>
                    </div>

                    <div class="student-info-card">
                        <div class="student-info-icon student-info-icon-teal">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"></path>
                                <polyline points="22,6 12,13 2,6"></polyline>
                            </svg>
                        </div>
                        <div class="student-info-text">
                            <div class="student-info-label">"Email Address"</div>
                            <div class="student-info-value">{user_email}</div>
                        </div>
                    </div>

                    <div class="student-info-card">
                        <div class="student-info-icon student-info-icon-cyan">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <path d="M22 10v6M2 10l10-5 10 5-10 5z"></path>
                                <path d="M6 12v5c3 3 9 3 12 0v-5"></path>
                            </svg>
                        </div>
                        <div class="student-info-text">
                            <div class="student-info-label">"Account Type"</div>
                            <div class="student-info-value">{user_role}</div>
                        </div>
                    </div>
                </section>

                {/* Account Settings */}
                <section class="student-profile-section">
                    <h3 class="student-section-title">
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <circle cx="12" cy="12" r="3"></circle>
                            <path d="M12 1v6m0 6v6"></path>
                        </svg>
                        "Account Settings"
                    </h3>

                    <button class="student-settings-item" on:click=open_reset_modal>
                        <div class="student-settings-icon student-settings-icon-red">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <path d="M12 1v6"></path>
                                <path d="M12 17v6"></path>
                                <circle cx="12" cy="12" r="3"></circle>
                            </svg>
                        </div>
                        <div class="student-settings-text">
                            <div class="student-settings-label">"Reset Password"</div>
                            <div class="student-settings-desc">"Change your account password"</div>
                        </div>
                        <svg class="student-settings-arrow" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M9 18l6-6-6-6"></path>
                        </svg>
                    </button>

                    
                </section>

                

                {/* Other Options */}
                <section class="student-profile-section">
                    <A href="/student/support" attr:class="student-settings-item" attr:style="text-decoration: none;">
                        <div class="student-settings-icon student-settings-icon-gray">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <circle cx="12" cy="12" r="10"></circle>
                                <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path>
                                <line x1="12" y1="17" x2="12.01" y2="17"></line>
                            </svg>
                        </div>
                        <div class="student-settings-text">
                            <div class="student-settings-label">"Support"</div>
                        </div>
                        <svg class="student-settings-arrow" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M9 18l6-6-6-6"></path>
                        </svg>
                    </A>

                    <button class="student-settings-item student-settings-item-danger" on:click=handle_sign_out>
                        <div class="student-settings-icon student-settings-icon-red">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"></path>
                                <polyline points="16 17 21 12 16 7"></polyline>
                                <line x1="21" y1="12" x2="9" y2="12"></line>
                            </svg>
                        </div>
                        <div class="student-settings-text">
                            <div class="student-settings-label">"Sign Out"</div>
                        </div>
                        <svg class="student-settings-arrow" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M9 18l6-6-6-6"></path>
                        </svg>
                    </button>
                </section>
            </div>

            {/* Reset Password Modal */}
            <Show when=move || show_reset_modal.get()>
                <div class="modal-overlay" on:click=close_reset_modal>
                    <div class="modal-content" on:click=move |e| e.stop_propagation()>
                        <div class="modal-header">
                            <h2 class="modal-title">"Reset Password"</h2>
                            <button class="modal-close-btn" on:click=close_reset_modal>
                                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <path d="M18 6L6 18M6 6l12 12"></path>
                                </svg>
                            </button>
                        </div>

                        <div class="modal-body">
                            <p class="modal-description">"Enter your new password below"</p>

                            <div class="modal-input-group">
                                <label class="modal-label">"New Password"</label>
                                <div class="modal-input-wrapper">
                                    <input
                                        type=move || if show_new_password.get() { "text" } else { "password" }
                                        class="modal-input"
                                        bind:value=new_password
                                        placeholder="Enter new password"
                                    />
                                    <button
                                        type="button"
                                        class="modal-eye-btn"
                                        on:click=toggle_new_password
                                    >
                                        <Show when=move || show_new_password.get() fallback=|| view! {
                                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                                                <circle cx="12" cy="12" r="3"/>
                                            </svg>
                                        }>
                                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/>
                                                <line x1="1" y1="1" x2="23" y2="23"/>
                                            </svg>
                                        </Show>
                                    </button>
                                </div>
                            </div>

                            <div class="modal-input-group">
                                <label class="modal-label">"Confirm Password"</label>
                                <div class="modal-input-wrapper">
                                    <input
                                        type=move || if show_confirm_password.get() { "text" } else { "password" }
                                        class="modal-input"
                                        bind:value=confirm_password
                                        placeholder="Confirm new password"
                                    />
                                    <button
                                        type="button"
                                        class="modal-eye-btn"
                                        on:click=toggle_confirm_password
                                    >
                                        <Show when=move || show_confirm_password.get() fallback=|| view! {
                                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                                                <circle cx="12" cy="12" r="3"/>
                                            </svg>
                                        }>
                                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/>
                                                <line x1="1" y1="1" x2="23" y2="23"/>
                                            </svg>
                                        </Show>
                                    </button>
                                </div>
                            </div>

                            <Show when=move || !reset_message.get().is_empty()>
                                <div class=move || if reset_success.get() { "modal-message modal-message-success" } else { "modal-message modal-message-error" }>
                                    {move || reset_message.get()}
                                </div>
                            </Show>

                            <div class="modal-actions">
                                <button
                                    class="modal-btn modal-btn-cancel"
                                    on:click=close_reset_modal
                                    disabled=move || reset_pending.get()
                                >
                                    "Cancel"
                                </button>
                                <button
                                    class="modal-btn modal-btn-confirm"
                                    on:click=handle_reset_submit
                                    disabled=move || reset_pending.get()
                                >
                                    {move || if reset_pending.get() { "Updating..." } else { "Update Password" }}
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}