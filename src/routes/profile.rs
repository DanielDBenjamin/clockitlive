use crate::routes::auth_functions::ResetPassword;
use crate::routes::profile_functions::{update_profile, UpdateProfileRequest};
use crate::user_context::{get_current_user, set_current_user};
use leptos::prelude::*;
use leptos_router::components::A;
use urlencoding::encode;

#[component]
pub fn Profile() -> impl IntoView {
    let current_user = get_current_user();

    // Form fields
    let name = RwSignal::new(String::new());
    let surname = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let university = RwSignal::new(String::new());
    let message = RwSignal::new(String::new());
    let success = RwSignal::new(false);

    let show_reset = RwSignal::new(false);
    let reset_new_password = RwSignal::new(String::new());
    let reset_confirm_password = RwSignal::new(String::new());
    let reset_message = RwSignal::new(String::new());
    let reset_success = RwSignal::new(false);
    // Password visibility state for reset inputs
    let show_new_password = RwSignal::new(false);
    let show_confirm_password = RwSignal::new(false);

    // Load current user data into form
    Effect::new(move |_| {
        if let Some(user) = current_user.get() {
            name.set(user.name.clone());
            surname.set(user.surname.clone());
            email.set(user.email_address.clone());
            university.set(user.university.clone());
        }
    });

    let update_action = Action::new(move |request: &UpdateProfileRequest| {
        let request = request.clone();
        async move { update_profile(request).await }
    });

    let reset_action = ServerAction::<ResetPassword>::new();
    let reset_pending = reset_action.pending();

    let on_submit = move |_| {
        message.set(String::new());
        success.set(false);

        let user_id = match current_user.get() {
            Some(user) => user.user_id,
            None => {
                message.set("You must be logged in".to_string());
                return;
            }
        };

        let request = UpdateProfileRequest {
            user_id,
            name: name.get(),
            surname: surname.get(),
            email_address: email.get(),
            university: university.get(),
        };

        update_action.dispatch(request);
    };

    let on_reset_submit = move |_| {
        reset_message.set(String::new());
        reset_success.set(false);
        reset_action.dispatch(ResetPassword {
            email: email.get(),
            new_password: reset_new_password.get(),
            confirm_password: reset_confirm_password.get(),
        });
    };

    // Handle response
    Effect::new(move |_| {
        if let Some(result) = update_action.value().get() {
            match result {
                Ok(response) => {
                    message.set(response.message.clone());
                    success.set(response.success);

                    if response.success {
                        // Update the user context with new data
                        if let Some(updated_user) = response.user {
                            set_current_user(updated_user);
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
                    if response.success {
                        reset_new_password.set(String::new());
                        reset_confirm_password.set(String::new());
                        show_reset.set(false);
                    }
                }
                Err(e) => {
                    reset_message.set(format!("Error: {}", e));
                    reset_success.set(false);
                }
            }
        }
    });

    let toggle_reset = move |_| {
        let next = !show_reset.get();
        show_reset.set(next);
        if !next {
            reset_new_password.set(String::new());
            reset_confirm_password.set(String::new());
        }
        reset_message.set(String::new());
        reset_success.set(false);
    };

    let user_display = move || {
        current_user
            .get()
            .map(|u| format!("{} {}", u.name, u.surname))
            .unwrap_or_else(|| "User".to_string())
    };

    let user_role = move || {
        current_user
            .get()
            .map(|u| match u.role.as_str() {
                "lecturer" => "Lecturer",
                "tutor" => "Tutor",
                "student" => "Student",
                _ => "User",
            })
            .unwrap_or("User")
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
        <section class="profile-page">
            <header class="page-header">
                <div class="page-title-row" style="display:flex;align-items:center;gap:8px;">
                    <A href="/home" attr:class="link">"←"</A>
                    <h1 class="page-title">"Profile & Account Settings"</h1>
                </div>
                <p class="page-subtitle">"Manage your personal information and account preferences"</p>
            </header>

            <section class="profile-card" aria-labelledby="profile-summary">
                <div class="profile-avatar">
                    <img
                        prop:src=move || avatar_url().unwrap_or_else(|| "https://ui-avatars.com/api/?name=User&background=14b8a6&color=ffffff&format=svg".to_string())
                        alt="Profile picture"
                    />
                </div>
                <div class="profile-summary">
                    <h2 id="profile-summary" class="profile-name">{user_display}</h2>
                    <p class="profile-role">{user_role}</p>
                </div>
            </section>

            <section class="profile-section" aria-labelledby="personal-information">
                <div class="profile-section-header">
                    <span class="profile-section-icon" aria-hidden="true">
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                            <circle cx="12" cy="7" r="4"/>
                            <path d="M5.5 21a6.5 6.5 0 0 1 13 0"/>
                        </svg>
                    </span>
                    <h2 id="personal-information" class="profile-section-title">"Personal Information"</h2>
                </div>
                <div class="profile-form">
                    <div class="profile-field">
                        <label class="profile-label" for="profile-name">"First Name"</label>
                        <input id="profile-name" class="input" type="text" bind:value=name autocomplete="given-name"/>
                    </div>
                    <div class="profile-field">
                        <label class="profile-label" for="profile-surname">"Last Name"</label>
                        <input id="profile-surname" class="input" type="text" bind:value=surname autocomplete="family-name"/>
                    </div>
                    <div class="profile-field">
                        <label class="profile-label" for="profile-email">"Email Address"</label>
                        <input id="profile-email" class="input" type="email" bind:value=email autocomplete="email"/>
                    </div>
                    <div class="profile-field">
                        <label class="profile-label" for="profile-university">"University"</label>
                        <input id="profile-university" class="input" type="text" bind:value=university autocomplete="organization"/>
                    </div>
                </div>
            </section>

            <section class="profile-section" aria-labelledby="account-settings">
                <div class="profile-section-header">
                    <span class="profile-section-icon profile-section-icon-gear" aria-hidden="true">
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                            <circle cx="12" cy="12" r="3"/>
                            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82h.09A1.65 1.65 0 0 0 9 4.09V4a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51h.09a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v.09a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
                        </svg>
                    </span>
                    <h2 id="account-settings" class="profile-section-title">"Account Settings"</h2>
                </div>
                <div class="profile-reset-row">
                    <div>
                        <h3 class="profile-reset-title">"Reset Password"</h3>
                        <p class="profile-reset-subtitle">"Change your account password"</p>
                    </div>
                    <button class="btn profile-reset-btn" type="button" on:click=toggle_reset>{move || if show_reset.get() { "Close" } else { "Reset" }}</button>
                </div>
                <Show when=move || show_reset.get()>
                    <div class="reset-inline">
                        <p class="muted">{move || format!("Reset for {}", email.get())}</p>
                        <label class="profile-label">"New Password"</label>
                        <div class="input-group">
                            <input class="input" type=move || if show_new_password.get() { "text" } else { "password" } bind:value=reset_new_password placeholder="Enter new password" />
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

                        <label class="profile-label">"Confirm Password"</label>
                        <div class="input-group">
                            <input class="input" type=move || if show_confirm_password.get() { "text" } else { "password" } bind:value=reset_confirm_password placeholder="Confirm new password" />
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
                        <button class="btn btn-outline" type="button" on:click=on_reset_submit disabled=move || reset_pending.get()>{move || if reset_pending.get() { "Updating..." } else { "Update Password" }}</button>
                        <Show when=move || !reset_message.get().is_empty()>
                            <p class=move || if reset_success.get() { "success" } else { "error" }>{reset_message}</p>
                        </Show>
                    </div>
                </Show>
            </section>

            <Show when=move || !message.get().is_empty()>
                <p class=move || if success.get() { "success center" } else { "error center" } style="margin-top:16px;">
                    {message}
                </p>
            </Show>

            <div class="profile-actions">
                <button
                    class="btn profile-save"
                    type="button"
                    on:click=on_submit
                    disabled=move || update_action.pending().get()
                >
                    {move || if update_action.pending().get() {
                        "Saving...".into_view()
                    } else {
                        "Save Changes".into_view()
                    }}
                </button>
                <A href="/home" attr:class="btn profile-cancel">"Cancel"</A>
            </div>
        </section>
    }
}
