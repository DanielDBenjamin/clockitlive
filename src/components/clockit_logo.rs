use leptos::prelude::*;

#[component]
pub fn ClockItLogo() -> impl IntoView {
    view! {
        <img
            src="public/logo.svg"
            alt="ClockIt Logo"
            class="clockit-logo"
            style="width:120px;height:32px;"
        />
    }
}