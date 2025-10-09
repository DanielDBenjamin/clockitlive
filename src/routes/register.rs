use leptos::prelude::*;
use leptos_router::components::A;

// Import the server functions and types
use crate::routes::auth_functions::register_user;
use crate::types::RegisterData;

#[component]
pub fn Register() -> impl IntoView {
    let name = RwSignal::new(String::new());
    let surname = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let confirm = RwSignal::new(String::new());
    let role = RwSignal::new("Lecturer".to_string());
    let message = RwSignal::new(String::new());
    let success = RwSignal::new(false);
    let terms_accepted = RwSignal::new(false);
    let show_terms_error = RwSignal::new(false);

    // Password visibility state
    let show_password = RwSignal::new(false);
    let show_confirm_password = RwSignal::new(false);

    let register_action = Action::new(|data: &RegisterData| {
        let data = data.clone();
        async move { register_user(data).await }
    });

    let on_submit = move |_: leptos::ev::MouseEvent| {
        message.set(String::new());
        success.set(false);

        let data = RegisterData {
            name: name.get(),
            surname: surname.get(),
            email: email.get(),
            password: password.get(),
            confirm_password: confirm.get(),
            role: role.get().to_lowercase(),
        };

        register_action.dispatch(data);
    };
    let on_submit = move |_| {
        message.set(String::new());
        success.set(false);
        show_terms_error.set(false);

        // Check if terms are accepted
        if !terms_accepted.get() {
            show_terms_error.set(true);
            message.set("You must accept the Terms of Service to create an account".to_string());
            return;
        }

        let data = RegisterData {
            name: name.get(),
            surname: surname.get(),
            email: email.get(),
            password: password.get(),
            confirm_password: confirm.get(),
            role: role.get().to_lowercase(),
        };

        register_action.dispatch(data);
    };
    
    // Handle response
    Effect::new(move |_| {
        if let Some(result) = register_action.value().get() {
            match result {
                Ok(auth_response) => {
                    message.set(auth_response.message);
                    success.set(auth_response.success);

                    if auth_response.success {
                        // Clear form on success
                        name.set(String::new());
                        surname.set(String::new());
                        email.set(String::new());
                        password.set(String::new());
                        confirm.set(String::new());
                    }
                }
                Err(e) => {
                    message.set(format!("Error: {}", e));
                    success.set(false);
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
                <div class="segmented">
                    <button
                        class=move || if role.get() == "Lecturer" { "seg-btn active" } else { "seg-btn" }
                        on:click=move |_| role.set("Lecturer".to_string())
                    >"Lecturer"</button>
                    <button
                        class=move || if role.get() == "Tutor" { "seg-btn active" } else { "seg-btn" }
                        on:click=move |_| role.set("Tutor".to_string())
                    >"Tutor"</button>
                    <button
                        class=move || if role.get() == "Student" { "seg-btn active" } else { "seg-btn" }
                        on:click=move |_| role.set("Student".to_string())
                    >"Student"</button>
                </div>

                <div class="form">
                    <div class="row-2">
                        <div>
                            <label class="label">"Name"</label>
                            <input class="input" type="text" placeholder="Name" bind:value=name />
                        </div>
                        <div>
                            <label class="label">"Surname"</label>
                            <input class="input" type="text" placeholder="Surname" bind:value=surname />
                        </div>
                    </div>

                    <label class="label">"Email"</label>
                    <div class="input-group">
                        <input class="input" type="email" placeholder="Enter your email" bind:value=email />
                        <span class="input-icon" aria-hidden="true">
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 8l8 6 8-6"/><rect x="4" y="4" width="16" height="16" rx="2"/></svg>
                        </span>
                    </div>

                    <label class="label">"Password"</label>
                    <div class="input-group">
                        <input 
                            class="input" 
                            type=move || if show_password.get() { "text" } else { "password" }
                            placeholder="Enter your password" 
                            bind:value=password 
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

                    <label class="label">"Confirm password"</label>
                    <div class="input-group">
                        <input 
                            class="input" 
                            type=move || if show_confirm_password.get() { "text" } else { "password" }
                            placeholder="Re-enter your password" 
                            bind:value=confirm 
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

                    <div style="display: flex; justify-content: center;">
                        <button
                            class="btn btn-accent"
                            on:click=on_submit
                            disabled=move || register_action.pending().get()
                            style="min-width: 200px; justify-content: center;"
                        >
                            <span style="opacity: 1;">
                                {move || if register_action.pending().get() {
                                    "Creating Account..."
                                } else {
                                    "Create Account"
                                }}
                            </span>
                        </button>
                    </div>

                    <div style="margin-top: 12px; margin-bottom: 12px;">
                        <label style="display: flex; align-items: center; gap: 8px; cursor: pointer;">
                            <input
                                type="checkbox"
                                checked=move || terms_accepted.get()
                                on:change=move |_| {
                                    terms_accepted.update(|v| *v = !*v);
                                    show_terms_error.set(false);
                                }
                                style="cursor: pointer;"
                            />
                            <span class="small" style=move || if show_terms_error.get() { "color: #dc2626;" } else { "" }>
                                "I accept the "
                                <a
                                    href="/terms"
                                    class="text-link accent"
                                    on:click=move |ev: leptos::ev::MouseEvent| {
                                        ev.prevent_default();
                                        // Save form data to localStorage before navigating
                                        if let Some(window) = web_sys::window() {
                                            if let Ok(Some(storage)) = window.local_storage() {
                                                let _ = storage.set_item("register_name", &name.get());
                                                let _ = storage.set_item("register_surname", &surname.get());
                                                let _ = storage.set_item("register_email", &email.get());
                                                let _ = storage.set_item("register_password", &password.get());
                                                let _ = storage.set_item("register_confirm", &confirm.get());
                                                let _ = storage.set_item("register_role", &role.get());
                                            }
                                        }
                                        let navigate = leptos_router::hooks::use_navigate();
                                        navigate("/terms", Default::default());
                                    }
                                >"Terms of Service"</a>
                            </span>
                        </label>
                    </div>

                    // Show messages
                    <Show when=move || !message.get().is_empty()>
                        <p class=move || if success.get() { "success center" } else { "error center" }>
                            {message}
                        </p>
                    </Show>

                    <p class="muted center" style="margin-top:8px;">
                        "Already have an account? "
                        <A href="/" attr:class="text-link accent">"Sign in"</A>
                    </p>
                </div>
            </div>
        </div>
    }
}
