use leptos::prelude::*;

#[component]
pub fn TermsAndConditions() -> impl IntoView {
    let handle_accept = move |_| {
        // Set localStorage flag that terms were accepted
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item("terms_accepted", "true");
            }
        }

        // Navigate back to register page
        let navigate = leptos_router::hooks::use_navigate();
        navigate("/register", Default::default());
    };

    view! {
        <div class="auth-layout">
            <div class="auth-card">
                <div class="auth-header">
                    <div class="logo-container">
                        <img src="/logo.png" srcset="/logo.png 1x" alt="Clock It" class="brand-logo-img" width="160" height="60" />
                    </div>
                    <p class="tagline">"Track your time, manage your life"</p>
                </div>
//T&C page 
               <div class="form" style="max-height: calc(70vh - 120px); overflow-y: auto;">
    <h2 style="text-align: center; margin-bottom: 20px;">"Terms of Service"</h2>

    <div style="padding: 20px; line-height: 1.8;">
        <ol style="list-style-type: decimal; padding-left: 20px;">
            <li style="margin-bottom: 16px;">
                <strong>"Privacy"</strong>
                <p style="margin: 8px 0 0 0;">
                    "We collect your name, email, university affiliation, and schedule data to provide the Service. We implement security measures to protect your information, but cannot guarantee 100% security."
                </p>
            </li>
            
            <li style="margin-bottom: 16px;">
                <strong>"Service Availability"</strong>
                <ul style="margin: 8px 0 0 0; padding-left: 20px; list-style-type: disc;">
                    <li>"We strive for continuous service but cannot guarantee uninterrupted access"</li>
                    <li>"The Service may be unavailable due to maintenance or technical issues"</li>
                    <li>"We may modify or discontinue the Service at any time"</li>
                </ul>
            </li>
            
            <li style="margin-bottom: 16px;">
                <strong>"Academic Disclaimer"</strong>
                <p style="margin: 8px 0 0 0;">
                    "ClockIt is a scheduling tool only. Always verify schedule information with official university sources. We are not responsible for missed classes or schedule conflicts."
                </p>
            </li>
            
            <li style="margin-bottom: 16px;">
                <strong>"Contact"</strong>
                <p style="margin: 8px 0 0 0;">
                    "For questions, contact us at: "
                    <a href="mailto:support@clockit.com" style="color: #14b8a6; text-decoration: none;">
                        "support@clockit.com"
                    </a>
                </p>
            </li>
        </ol>
    </div>
</div>

                    <button
                        class="btn btn-accent btn-block"
                        on:click=handle_accept
                        style="margin-top: 20px;"
                    >
                        "Accept"
                    </button>
                </div>
            </div>
    }
}