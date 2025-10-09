use leptos::prelude::*;
use leptos_router::components::A;
use crate::routes::auth_functions::{send_password_reset_otp, verify_otp, reset_password};

#[component]
pub fn ForgotPassword() -> impl IntoView {
    let email = RwSignal::new(String::new());
    let otp_code = RwSignal::new(String::new());
    let new_password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());
    let message = RwSignal::new(String::new());
    let success = RwSignal::new(false);
    
    // State management
    let step = RwSignal::new(1); // 1: Enter email, 2: Enter OTP, 3: Enter new password
    let show_password = RwSignal::new(false);
    let show_confirm_password = RwSignal::new(false);

    // Actions
    let send_otp_action = Action::new(|email: &String| {
        let email = email.clone();
        async move { send_password_reset_otp(email).await }
    });
    
    let verify_otp_action = Action::new(|(email, otp): &(String, String)| {
        let email = email.clone();
        let otp = otp.clone();
        async move { verify_otp(email, otp).await }
    });
    
    let reset_password_action = Action::new(|(email, password, confirm): &(String, String, String)| {
        let email = email.clone();
        let password = password.clone();
        let confirm = confirm.clone();
        async move { reset_password(email, password, confirm).await }
    });

    // Step 1: Send OTP
    let on_send_otp = move |_| {
        message.set(String::new());
        success.set(false);
        
        if email.get().trim().is_empty() {
            message.set("Please enter your email address".to_string());
            return;
        }
        
        send_otp_action.dispatch(email.get());
    };
    
    // Step 2: Verify OTP
    let on_verify_otp = move |_| {
        message.set(String::new());
        success.set(false);
        
        if otp_code.get().len() != 6 {
            message.set("Please enter the 6-digit code".to_string());
            return;
        }
        
        verify_otp_action.dispatch((email.get(), otp_code.get()));
    };
    
    // Step 3: Reset password
    let on_reset_password = move |_| {
        message.set(String::new());
        success.set(false);
        
        if new_password.get().len() < 6 {
            message.set("Password must be at least 6 characters".to_string());
            return;
        }
        
        if new_password.get() != confirm_password.get() {
            message.set("Passwords do not match".to_string());
            return;
        }
        
        reset_password_action.dispatch((email.get(), new_password.get(), confirm_password.get()));
    };

    // Handle send OTP response
    Effect::new(move |_| {
        if let Some(result) = send_otp_action.value().get() {
            match result {
                Ok(response) => {
                    if response.success {
                        step.set(2);
                        message.set(response.message);
                        success.set(true);
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
    });
    
    // Handle verify OTP response
    Effect::new(move |_| {
        if let Some(result) = verify_otp_action.value().get() {
            match result {
                Ok(response) => {
                    if response.success {
                        step.set(3);
                        message.set("OTP verified! Enter your new password.".to_string());
                        success.set(true);
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
    });
    
    // Handle reset password response
    Effect::new(move |_| {
        if let Some(result) = reset_password_action.value().get() {
            match result {
                Ok(response) => {
                    message.set(response.message);
                    success.set(response.success);
                    
                    if response.success {
                        // Clear form and show success
                        email.set(String::new());
                        otp_code.set(String::new());
                        new_password.set(String::new());
                        confirm_password.set(String::new());
                        step.set(4); // Success step
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
                    <h2 class="auth-title">
                        {move || match step.get() {
                            1 => "Reset Password",
                            2 => "Verify Email",
                            3 => "New Password",
                            4 => "Password Reset",
                            _ => "Reset Password"
                        }}
                    </h2>
                    <p class="auth-subtitle">
                        {move || match step.get() {
                            1 => "Enter your email to receive a reset code",
                            2 => "Check your email for the verification code",
                            3 => "Enter your new password",
                            4 => "Your password has been updated",
                            _ => ""
                        }}
                    </p>
                </div>

                <div class="form">
                    // Step 1: Enter Email
                    <Show when=move || step.get() == 1>
                        <label class="label">"Email Address"</label>
                        <div class="input-group">
                            <input 
                                class="input" 
                                type="email" 
                                placeholder="Enter your email address" 
                                bind:value=email 
                            />
                            <span class="input-icon" aria-hidden="true">
                                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <path d="M4 8l8 6 8-6"/>
                                    <rect x="4" y="4" width="16" height="16" rx="2"/>
                                </svg>
                            </span>
                        </div>

                        <div style="display: flex; justify-content: center; margin-top: 20px;">
                            <button
                                class="btn btn-accent"
                                on:click=on_send_otp
                                disabled=move || send_otp_action.pending().get()
                                style="min-width: 200px; justify-content: center; text-align: center;"
                            >
                                {move || if send_otp_action.pending().get() {
                                    "Sending Code..."
                                } else {
                                    "Send Reset Code"
                                }}
                            </button>
                        </div>
                    </Show>

                    // Step 2: Enter OTP
                    <Show when=move || step.get() == 2>
                        <label class="label">"Verification Code"</label>
                        <div class="input-group">
                            <input 
                                class="input" 
                                type="text" 
                                placeholder="Enter 6-digit code" 
                                maxlength="6"
                                bind:value=otp_code
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    // Only allow digits
                                    let digits: String = value.chars().filter(|c| c.is_ascii_digit()).take(6).collect();
                                    otp_code.set(digits);
                                }
                            />
                        </div>

                        <div style="display: flex; justify-content: center; gap: 12px; margin-top: 20px;">
                            <button
                                class="btn btn-secondary"
                                on:click=move |_| step.set(1)
                                style="min-width: 120px; text-align: center; display: flex; align-items: center; justify-content: center;"
                            >
                                "Back"
                            </button>
                            <button
                                class="btn btn-accent"
                                on:click=on_verify_otp
                                disabled=move || verify_otp_action.pending().get() || otp_code.get().len() != 6
                                style="min-width: 120px; text-align: center; display: flex; align-items: center; justify-content: center;"
                            >
                                {move || if verify_otp_action.pending().get() {
                                    "Verifying..."
                                } else {
                                    "Verify Code"
                                }}
                            </button>
                        </div>

                        <div style="text-align: center; margin-top: 16px;">
                            <button 
                                class="text-link"
                                on:click=move |_| {
                                    send_otp_action.dispatch(email.get());
                                }
                                disabled=move || send_otp_action.pending().get()
                                style="border: none; text-align: center;"
                            >
                                {move || if send_otp_action.pending().get() {
                                    "Resending..."
                                } else {
                                    "Resend Code"
                                }}
                            </button>
                        </div>
                    </Show>

                    // Step 3: Enter New Password
                    <Show when=move || step.get() == 3>
                        <label class="label">"New Password"</label>
                        <div class="input-group">
                            <input 
                                class="input" 
                                type=move || if show_password.get() { "text" } else { "password" }
                                placeholder="Enter new password" 
                                bind:value=new_password 
                            />
                            <span 
                                class="input-icon password-toggle" 
                                on:click=move |_| show_password.set(!show_password.get())
                                role="button"
                                tabindex="0"
                            >
                                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <path d="M1 12s4-7 11-7 11 7 11 7-4 7-11 7-11-7-11-7z"/>
                                    <circle cx="12" cy="12" r="3"/>
                                </svg>
                            </span>
                        </div>

                        <label class="label">"Confirm New Password"</label>
                        <div class="input-group">
                            <input 
                                class="input" 
                                type=move || if show_confirm_password.get() { "text" } else { "password" }
                                placeholder="Confirm new password" 
                                bind:value=confirm_password 
                            />
                            <span 
                                class="input-icon password-toggle" 
                                on:click=move |_| show_confirm_password.set(!show_confirm_password.get())
                                role="button"
                                tabindex="0"
                            >
                                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <path d="M1 12s4-7 11-7 11 7 11 7-4 7-11 7-11-7-11-7z"/>
                                    <circle cx="12" cy="12" r="3"/>
                                </svg>
                            </span>
                        </div>

                        <div style="display: flex; justify-content: center; gap: 12px; margin-top: 20px;">
                            <button
                                class="btn btn-secondary"
                                on:click=move |_| step.set(2)
                                style="min-width: 120px; text-align: center; display: flex; align-items: center; justify-content: center;"
                            >
                                "Back"
                            </button>
                            <button
                                class="btn btn-accent"
                                on:click=on_reset_password
                                disabled=move || reset_password_action.pending().get()
                                style="min-width: 120px; text-align: center; display: flex; align-items: center; justify-content: center;"
                            >
                                {move || if reset_password_action.pending().get() {
                                    "Updating..."
                                } else {
                                    "Update Password"
                                }}
                            </button>
                        </div>
                    </Show>

                    // Step 4: Success
                    <Show when=move || step.get() == 4>
                        <div style="text-align: center; padding: 20px;">
                            <div style="color: #10b981; font-size: 48px; margin-bottom: 16px;">
                                "✓"
                            </div>
                            <p style="color: #10b981; font-weight: 600; margin-bottom: 20px;">
                                "Password Updated Successfully!"
                            </p>
                            <A href="/" attr:class="btn btn-accent" attr:style="min-width: 200px; text-align: center; display: flex; align-items: center; justify-content: center;">
                                "Sign In Now"
                            </A>
                        </div>
                    </Show>

                    // Show messages
                    <Show when=move || !message.get().is_empty() && step.get() != 4>
                        <p class=move || if success.get() { "success center" } else { "error center" }>
                            {message}
                        </p>
                    </Show>

                    // Back to login link (only show on steps 1-3)
                    <Show when=move || step.get() <= 3>
                        <p class="muted center" style="margin-top: 20px;">
                            "Remember your password? "
                            <A href="/" attr:class="text-link accent">"Sign in"</A>
                        </p>
                    </Show>
                </div>
            </div>
        </div>
    }
}