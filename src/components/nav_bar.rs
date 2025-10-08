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
                <NavLink href="/home" label="Home" icon_type="home"/>
                <NavLink href="/timetable" label="Timetable" icon_type="calendar"/>
                <NavLink href="/statistics" label="Statistics" icon_type="chart"/>
            </nav>
            <div class="sidebar-footer">
                <button class="signout" on:click=handle_signout>"Sign Out"</button>
            </div>
        </aside>
    }
}

#[component]
fn NavLink(href: &'static str, label: &'static str, icon_type: &'static str) -> impl IntoView {
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
            "nav-link active"
        } else {
            "nav-link"
        }
    };

    view! {
        <A href=href attr:class=classes>
            <span class="icon" aria-hidden="true">
                {move || match icon_type {
                    "home" => view! {
                        <svg class="nav-icon" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path>
                            <polyline points="9 22 9 12 15 12 15 22"></polyline>
                        </svg>
                    }.into_any(),
                    "calendar" => view! {
                        <svg class="nav-icon" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
                            <line x1="16" y1="2" x2="16" y2="6"></line>
                            <line x1="8" y1="2" x2="8" y2="6"></line>
                            <line x1="3" y1="10" x2="21" y2="10"></line>
                        </svg>
                    }.into_any(),
                    "chart" => view! {
                        <svg class="nav-icon" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <line x1="18" y1="20" x2="18" y2="10"></line>
                            <line x1="12" y1="20" x2="12" y2="4"></line>
                            <line x1="6" y1="20" x2="6" y2="14"></line>
                        </svg>
                    }.into_any(),
                    _ => view! { <span></span> }.into_any(),
                }}
            </span>
            <span class="label">{label}</span>
        </A>
    }
}