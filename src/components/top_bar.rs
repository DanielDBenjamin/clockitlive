use crate::user_context::get_current_user;
use leptos::prelude::*;
use leptos_router::components::A;
use crate::components::ThemeSwitcher;

#[component]
pub fn TopBar() -> impl IntoView {
    let current_user = get_current_user();

    let user_name = move || match current_user.get() {
        Some(user) => format!("{} {}", user.name, user.surname),
        None => "User".to_string(),
    };

    view! {
        <header class="topbar" role="banner">
            <div class="topbar-left">
                <div class="brand">
                    <A href="/home">
                        <img
                            src="logo.png"
                            alt="Logo"
                            class="clockit-logo"
                            style="width:120px;height:32px;"
                        />
                    </A>
                </div>
            </div>
            <div class="topbar-right">
            <ThemeSwitcher/>
                <A href="/lecturer/profile" attr:class="user-chip">
                    <span class="avatar" aria-hidden="true">"👩🏻‍🏫"</span>
                    <span class="name">{user_name}</span>
                </A>
            </div>
        </header>
    }
}