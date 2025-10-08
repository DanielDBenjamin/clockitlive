use crate::routes::auth_functions::LoginUser;
use crate::user_context::set_current_user;
use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn StudentLoginPage() -> impl IntoView {
    let navigate = use_navigate();

    // Create signals for form inputs
    let (student_id, set_student_id) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (show_password, set_show_password) = signal(false);
    let feedback = RwSignal::new(None::<(bool, String)>);

    let login_action = ServerAction::<LoginUser>::new();

    Effect::new({
        let feedback = feedback.clone();
        move |_| {
            if login_action.pending().get() {
                feedback.set(None);
            }
        }
    });

    Effect::new({
        let navigate = navigate.clone();
        let feedback = feedback.clone();
        move |_| {
            if let Some(result) = login_action.value().get() {
                match result {
                    Ok(response) => {
                        if response.success {
                            if let Some(user) = response.user {
                                set_current_user(user.clone());
                                navigate("/student/home", Default::default());
                            }
                        } else {
                            feedback.set(Some((false, response.message.clone())));
                        }
                    }
                    Err(err) => {
                        feedback.set(Some((false, err.to_string())));
                    }
                }
            }
        }
    });

    // Handle form submission
    let handle_submit = move |ev: SubmitEvent| {
        ev.prevent_default();

        let student_id_value = student_id.get();
        let password_value = password.get();

        login_action.dispatch(LoginUser {
            email: student_id_value,
            password: password_value,
        });
    };

    // Toggle password visibility
    let toggle_password = move |_| {
        set_show_password.update(|show| *show = !*show);
    };

    view! {
        <div class="student-mobile-container">
            {/* Header with logo and tagline */}
            <div class="student-header-section">
                <div class="student-logo-container">
                    <img src="/logo.png" srcset="/logo.png 1x" alt="Clock It" class="student-brand-logo-img" width="160" height="60" />
                </div>
                <p class="student-tagline">"Track your time, manage your life"</p>
            </div>

            {/* Login form card */}
            <div class="student-login-card">
                <div class="student-login-header">
                    <h2 class="student-login-title">"Welcome back"</h2>
                    <p class="student-login-subtitle">"Sign in to your Clock It account"</p>
                </div>

                <form class="student-login-form" on:submit=handle_submit>
                    <div class="student-form-group">
                        <label class="student-form-label" for="student-id">"Student ID"</label>
                        <div class="student-input-container">
                            <input
                                type="text"
                                id="student-id"
                                class="student-form-input"
                                placeholder="Enter your student ID or email"
                                prop:value=student_id
                                on:input=move |ev| {
                                    set_student_id.set(event_target_value(&ev));
                                }
                            />
                            <div class="student-input-icon">
                                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"></path>
                                    <polyline points="22,6 12,13 2,6"></polyline>
                                </svg>
                            </div>
                        </div>
                    </div>

                    <div class="student-form-group">
                        <label class="student-form-label" for="password">"Password"</label>
                        <div class="student-input-container">
                            <input
                                type=move || if show_password.get() { "text" } else { "password" }
                                id="password"
                                class="student-form-input"
                                placeholder="Enter your password"
                                prop:value=password
                                on:input=move |ev| {
                                    set_password.set(event_target_value(&ev));
                                }
                            />
                            <button
                                type="button"
                                class="student-password-toggle"
                                on:click=toggle_password
                            >
                                {move || if show_password.get() {
                                    view! {
                                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                            <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path>
                                            <line x1="1" y1="1" x2="23" y2="23"></line>
                                        </svg>
                                    }
                                } else {
                                    view! {
                                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                            <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
                                            <circle cx="12" cy="12" r="3"></circle>
                                        </svg>
                                    }
                                }}
                            </button>
                        </div>
                    </div>

                    <button
                        type="submit"
                        class="student-login-button"
                        disabled=move || login_action.pending().get()
                    >
                        {move || if login_action.pending().get() {
                            "Signing in…"
                        } else {
                            "Sign In"
                        }}
                    </button>
                </form>

                {move || {
                    feedback
                        .get()
                        .map(|(_, message)| {
                            view! { <p class="student-login-feedback">{message}</p> }.into_any()
                        })
                        .unwrap_or_else(|| view! { <></> }.into_any())
                }}

                <div class="student-login-footer">
                    <a href="#" class="student-forgot-link">"Forgot password?"</a>
                    <a href="/register" class="student-create-link">"Create account"</a>
                </div>

                <div class="student-terms-section">
                    <p class="student-terms-text">
                        "By signing in, you agree to our "
                        <a href="#" class="student-terms-link">"Terms of Service"</a>
                        " and "
                        <a href="#" class="student-terms-link">"Privacy Policy"</a>
                    </p>
                </div>
            </div>
        </div>
    }
}
