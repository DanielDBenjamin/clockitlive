use crate::user_context::get_current_user;
use leptos::prelude::*;
use leptos_router::components::A;
use crate::components::ThemeSwitcher;
use urlencoding::encode;

#[component]
pub fn TopBar() -> impl IntoView {
    let current_user = get_current_user();

    let user_name = move || match current_user.get() {
        Some(user) => format!("{} {}", user.name, user.surname),
        None => "User".to_string(),
    };

    let avatar_url = Signal::derive(move || {
        current_user.get().map(|u| {
            let full_name = format!("{} {}", u.name, u.surname);
            let encoded = encode(&full_name);
            format!(
                "https://ui-avatars.com/api/?name={}&background=14b8a6&color=ffffff&format=svg",
                encoded
            )
        })
    });

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
                    <img
                        class="avatar"
                        alt=user_name
                        prop:src=move || {
                            avatar_url
                                .get()
                                .unwrap_or_else(|| "/logo.png".to_string())
                        }
                    />
                    <span class="name">{user_name}</span>
                </A>
            </div>
        </header>
    }
}