use crate::user_context::get_current_user;
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn TopBar() -> impl IntoView {
    let current_user = get_current_user();

    let user_name = move || match current_user.get() {
        Some(user) => format!("{} {}", user.name, user.surname),
        None => "User".to_string(),
    };

    let profile_url = move || match current_user.get() {
        Some(user) => match user.role.as_str() {
            "lecturer" => "/lecturer/profile".to_string(),
            "tutor" => "/tutor/profile".to_string(),
            _ => "/lecturer/profile".to_string(), // fallback
        },
        None => "/lecturer/profile".to_string(),
    };

    let user_avatar = move || match current_user.get() {
        Some(user) => match user.role.as_str() {
            "lecturer" => "👩🏻‍🏫",
            "tutor" => "👨🏻‍🏫",
            _ => "👩🏻‍🏫", // fallback
        },
        None => "👩🏻‍🏫",
    };

    view! {
        <header class="topbar" role="banner">
            <div class="topbar-left">
                <div class="brand"><A href="/home"><img src="/logo.png" alt="Logo"/></A></div>
            </div>
            <div class="topbar-right">
                <A href=profile_url attr:class="user-chip">
                    <span class="avatar" aria-hidden="true">{user_avatar}</span>
                    <span class="name">{user_name}</span>
                </A>
            </div>
        </header>
    }
}
