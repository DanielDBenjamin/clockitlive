use crate::routes::auth_functions::{LoginUser, ResetPassword};
use crate::user_context::set_current_user;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

#[component]
pub fn Login() -> impl IntoView {
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let _role = RwSignal::new("Lecturer".to_string());
    let message = RwSignal::new(String::new());
    let success = RwSignal::new(false);

    let show_reset = RwSignal::new(false);
    let reset_email = RwSignal::new(String::new());
    let reset_new_password = RwSignal::new(String::new());
    let reset_confirm_password = RwSignal::new(String::new());
    let reset_message = RwSignal::new(String::new());
    let reset_success = RwSignal::new(false);

    // Password visibility state
    let show_password = RwSignal::new(false);
    let show_new_password = RwSignal::new(false);
    let show_confirm_password = RwSignal::new(false);

    let navigate = use_navigate();

    let login_action = ServerAction::<LoginUser>::new();
    let reset_action = ServerAction::<ResetPassword>::new();

    // Client-side validation for password reset
    let reset_password_valid = Signal::derive(move || {
        let new_pass = reset_new_password.get();
        let confirm_pass = reset_confirm_password.get();
        
        if new_pass.is_empty() || confirm_pass.is_empty() {
            return (false, String::new());
        }
        
        if new_pass.len() < 6 {
            return (false, "Password must be at least 6 characters".to_string());
        }
        
        if new_pass != confirm_pass {
            return (false, "Passwords do not match".to_string());
        }
        
        (true, String::new())
    });

    Effect::new({
        let message = message.clone();
        let success = success.clone();
        move |_| {
            if login_action.pending().get() {
                message.set(String::new());
                success.set(false);
            }
        }
    });

    Effect::new({
        let reset_message = reset_message.clone();
        let reset_success = reset_success.clone();
        move |_| {
            if reset_action.pending().get() {
                reset_message.set(String::new());
                reset_success.set(false);
            }
        }
    });

    // Handle login response
    Effect::new(move |_| {
        if let Some(result) = login_action.value().get() {
            match result {
                Ok(auth_response) => {
                    message.set(auth_response.message);
                    success.set(auth_response.success);

                    if auth_response.success {
                        // Store the logged-in user
                        if let Some(user) = auth_response.user {
                            // Redirect based on role
                            let redirect_path = match user.role.as_str() {
                                "student" => "/student/home",
                                "lecturer" | "tutor" => "/home",
                                _ => "/home", // Default fallback
                            };

                            set_current_user(user);
                            navigate(redirect_path, Default::default());
                        }
                    }
                }
                Err(e) => {
                    message.set(format!("Error: {}", e));
                    success.set(false);
                }
            }
        }
    });

    Effect::new(move |_| {
        if let Some(result) = reset_action.value().get() {
            match result {
                Ok(response) => {
                    reset_message.set(response.message.clone());
                    reset_success.set(response.success);
                }
                Err(e) => {
                    reset_message.set(format!("Error: {}", e));
                    reset_success.set(false);
                }
            }
        }
    });

    view! {
        <div class="auth-layout">
            <div class="auth-card">
                <div class="auth-header">
                    <div class="logo-container">
                        <img src="/logo.png" srcset="/logo.png 1x" alt="Clock It" class="brand-logo-img" width="160" height="60" />
                    </div>
                    <p class="tagline">"Track your time, manage your life"</p>
                </div>

                <ActionForm action=login_action>
                    <div class="form">
                    <label class="label">"Email"</label>
                    <div class="input-group">
                        <input
                            class="input"
                            type="email"
                            name="email"
                            placeholder="jane.gerber@university.edu"
                            bind:value=email
                            required
                            autocomplete="email"
                        />
                        <span class="input-icon" aria-hidden="true">
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4h16v16H4z" opacity="0"></path><path d="M4 8l8 6 8-6"/><rect x="4" y="4" width="16" height="16" rx="2"/></svg>
                        </span>
                    </div>

                    <label class="label">"Password"</label>
                    <div class="input-group">
                        <input
                            class="input"
                            type=move || if show_password.get() { "text" } else { "password" }
                            name="password"
                            placeholder="••••••••"
                            bind:value=password
                            required
                            autocomplete="current-password"
                        />
                        <span 
                            class="input-icon password-toggle" 
                            on:click=move |_| show_password.set(!show_password.get())
                            on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                if ev.key() == "Enter" || ev.key() == " " {
                                    ev.prevent_default();
                                    show_password.set(!show_password.get());
                                }
                            }
                            role="button"
                            tabindex="0"
                            aria-label=move || if show_password.get() { "Hide password" } else { "Show password" }
                        >
                            // Eye closed (hidden password)
                            <svg 
                                width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" 
                                stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                                style=move || if show_password.get() { "opacity: 0; position: absolute;" } else { "opacity: 1;" }
                            >
                                <path d="M1 12s4-7 11-7 11 7 11 7-4 7-11 7-11-7-11-7z"/>
                                <circle cx="12" cy="12" r="3"/>
                            </svg>
                            // Eye open with slash (visible password)
                            <svg 
                                width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" 
                                stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                                style=move || if show_password.get() { "opacity: 1;" } else { "opacity: 0; position: absolute;" }
                            >
                                <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20C5 20 1 12 1 12a18.45 18.45 0 0 1 2.06-2.94L17.94 17.94Z"/>
                                <path d="M9.9 4.24A9.12 9.12 0 0 1 12 4C19 4 23 12 23 12a18.5 18.5 0 0 1-2.16 3.19L9.9 4.24Z"/>
                                <line x1="1" y1="1" x2="23" y2="23"/>
                            </svg>
                        </span>
                    </div>

                    <div style="display: flex; justify-content: center;">
                        <button
                            class="btn btn-accent"
                            type="submit"
                            disabled=move || {
                                login_action.pending().get() || 
                                email.get().trim().is_empty() || 
                                password.get().trim().is_empty()
                            }
                            style="min-width: 200px; justify-content: center;"
                        >
                            <span style="opacity: 1;">
                                {move || if login_action.pending().get() {
                                    "Signing in..."
                                } else {
                                    "Sign In"
                                }}
                            </span>
                        </button>
                    </div>

                    <p class="center" style="margin:10px 0 0;">
                        <button class="link-button accent" on:click=move |ev: leptos::ev::MouseEvent| {
                            ev.prevent_default();
                            let next = !show_reset.get();
                            show_reset.set(next);
                            if next { reset_email.set(email.get()); }
                            reset_message.set(String::new());
                            reset_success.set(false);
                        }>"Forgot password?"</button>
                    </p>
                    <Show when=move || show_reset.get()>
                        <ActionForm action=reset_action>
                            <div class="reset-inline">
                            <label class="label">"Email"</label>
                            <input
                                class="input"
                                type="email"
                                name="email"
                                placeholder="jane.gerber@university.edu"
                                bind:value=reset_email
                            />

                            <label class="label">"New Password"</label>
                            <div class="input-group">
                                <input
                                    class="input"
                                    type=move || if show_new_password.get() { "text" } else { "password" }
                                    name="new_password"
                                    placeholder="••••••••"
                                    bind:value=reset_new_password
                                />
                                <span 
                                    class="input-icon password-toggle" 
                                    on:click=move |_| show_new_password.set(!show_new_password.get())
                                    on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                        if ev.key() == "Enter" || ev.key() == " " {
                                            ev.prevent_default();
                                            show_new_password.set(!show_new_password.get());
                                        }
                                    }
                                    role="button"
                                    tabindex="0"
                                    aria-label=move || if show_new_password.get() { "Hide password" } else { "Show password" }
                                >
                                    // Eye closed (hidden password)
                                    <svg 
                                        width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" 
                                        stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                                        style=move || if show_new_password.get() { "opacity: 0; position: absolute;" } else { "opacity: 1;" }
                                    >
                                        <path d="M1 12s4-7 11-7 11 7 11 7-4 7-11 7-11-7-11-7z"/>
                                        <circle cx="12" cy="12" r="3"/>
                                    </svg>
                                    // Eye open with slash (visible password)
                                    <svg 
                                        width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" 
                                        stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                                        style=move || if show_new_password.get() { "opacity: 1;" } else { "opacity: 0; position: absolute;" }
                                    >
                                        <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20C5 20 1 12 1 12a18.45 18.45 0 0 1 2.06-2.94L17.94 17.94Z"/>
                                        <path d="M9.9 4.24A9.12 9.12 0 0 1 12 4C19 4 23 12 23 12a18.5 18.5 0 0 1-2.16 3.19L9.9 4.24Z"/>
                                        <line x1="1" y1="1" x2="23" y2="23"/>
                                    </svg>
                                </span>
                            </div>

                            <label class="label">"Confirm Password"</label>
                            <div class="input-group">
                                <input
                                    class="input"
                                    type=move || if show_confirm_password.get() { "text" } else { "password" }
                                    name="confirm_password"
                                    placeholder="••••••••"
                                    bind:value=reset_confirm_password
                                />
                                <span 
                                    class="input-icon password-toggle" 
                                    on:click=move |_| show_confirm_password.set(!show_confirm_password.get())
                                    on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                        if ev.key() == "Enter" || ev.key() == " " {
                                            ev.prevent_default();
                                            show_confirm_password.set(!show_confirm_password.get());
                                        }
                                    }
                                    role="button"
                                    tabindex="0"
                                    aria-label=move || if show_confirm_password.get() { "Hide password" } else { "Show password" }
                                >
                                    // Eye closed (hidden password)
                                    <svg 
                                        width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" 
                                        stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                                        style=move || if show_confirm_password.get() { "opacity: 0; position: absolute;" } else { "opacity: 1;" }
                                    >
                                        <path d="M1 12s4-7 11-7 11 7 11 7-4 7-11 7-11-7-11-7z"/>
                                        <circle cx="12" cy="12" r="3"/>
                                    </svg>
                                    // Eye open with slash (visible password)
                                    <svg 
                                        width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" 
                                        stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                                        style=move || if show_confirm_password.get() { "opacity: 1;" } else { "opacity: 0; position: absolute;" }
                                    >
                                        <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20C5 20 1 12 1 12a18.45 18.45 0 0 1 2.06-2.94L17.94 17.94Z"/>
                                        <path d="M9.9 4.24A9.12 9.12 0 0 1 12 4C19 4 23 12 23 12a18.5 18.5 0 0 1-2.16 3.19L9.9 4.24Z"/>
                                        <line x1="1" y1="1" x2="23" y2="23"/>
                                    </svg>
                                </span>
                            </div>

                            // Client-side validation feedback
                            <Show when=move || {
                                let (valid, msg) = reset_password_valid.get();
                                !msg.is_empty() && !valid
                            }>
                                <p class="error center" style="margin: 8px 0;">
                                    {move || reset_password_valid.get().1}
                                </p>
                            </Show>

                            <button 
                                class="btn btn-outline btn-block" 
                                type="submit" 
                                disabled=move || {
                                    reset_action.pending().get() || 
                                    reset_email.get().trim().is_empty() ||
                                    !reset_password_valid.get().0
                                }
                            >
                                {move || if reset_action.pending().get() { "Updating..." } else { "Reset Password" }}
                            </button>

                            <Show when=move || !reset_message.get().is_empty()>
                                <p class=move || if reset_success.get() { "success center" } else { "error center" }>
                                    {reset_message}
                                </p>
                            </Show>
                            </div>
                        </ActionForm>
                    </Show>

                    <p class="muted center" style="margin:6px 0 0;">
                        "Don't have an account? "
                        <A href="/register" attr:class="text-link accent">"Create account"</A>
                    </p>

                    // Show messages
                    <Show when=move || !message.get().is_empty()>
                        <p class=move || if success.get() { "success center" } else { "error center" }>
                            {message}
                        </p>
                    </Show>
                    </div>
                </ActionForm>
            </div>
        </div>
    }
}
