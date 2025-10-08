use crate::user_context::clear_current_user;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_location, use_navigate};

#[component]
pub fn NavBar() -> impl IntoView {
    let navigate = use_navigate();

    let handle_signout = move |_| {
        clear_current_user();
        navigate("/", Default::default());
    };
    view! {
        <aside class="sidebar">
            <nav class="nav">
                <NavLink href="/home" label="Home" icon="🏠"/>
                <NavLink href="/timetable" label="Timetable" icon="📅"/>
                <NavLink href="/statistics" label="Statistics" icon="📊"/>
            </nav>
            <div class="sidebar-footer">
                <button class="signout" on:click=handle_signout>"Sign Out"</button>
            </div>
        </aside>
    }
}

#[component]
fn NavLink(href: &'static str, label: &'static str, icon: &'static str) -> impl IntoView {
    let location = use_location();
    let is_active = Signal::derive(move || {
        let path = location.pathname.get();
        if href == "/" {
            path == "/" || path.is_empty()
        } else {
            path.starts_with(href)
        }
    });

    let classes = move || {
        if is_active.get() {
            "nav-link active".to_string()
        } else {
            "nav-link".to_string()
        }
    };

    view! {
        // Order matters for typed builder: href before class
        <A href=href attr:class=classes>
            <span class="icon" aria-hidden="true">{icon}</span>
            <span class="label">{label}</span>
        </A>
    }
}
