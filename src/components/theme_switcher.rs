use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

#[derive(Clone, Copy)]
pub struct ThemeContext {
    pub theme: RwSignal<String>,
}

pub fn provide_theme_context() {
    let theme = RwSignal::new("light".to_string());

    #[cfg(not(feature = "ssr"))]
    {
        use web_sys::window;

        if let Some(window) = window() {
            if let Ok(storage) = window.local_storage() {
                if let Some(storage) = storage {
                    if let Ok(Some(saved_theme)) = storage.get_item("theme") {
                        theme.set(saved_theme);
                    }
                }
            }
        }

        // Sync theme to <html> and localStorage
        Effect::new(move |_| {
            if let Some(window) = window() {
                if let Some(document) = window.document() {
                    if let Some(html) = document.document_element() {
                        let _ = html.set_attribute("data-theme", &theme.get());
                    }
                }
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item("theme", &theme.get());
                }
            }
        });
    }

    provide_context(ThemeContext { theme });
}

#[component]
pub fn ThemeSwitcher() -> impl IntoView {
    let ctx = use_context::<ThemeContext>().expect("ThemeContext not provided");

    let toggle_theme = move |_| {
        let current = ctx.theme.get();
        ctx.theme.set(if current == "light" {
            "dark".to_string()
        } else {
            "light".to_string()
        });
    };

    view! {
        <div class="settings-controls">
            <button
                class="theme-toggle"
                on:click=toggle_theme
                title=move || if ctx.theme.get() == "light" { "Switch to Dark Mode" } else { "Switch to Light Mode" }
            >
                {move || {
                    if ctx.theme.get() == "light" {
                        view! {
                            <span>
                                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
                                </svg>
                            </span>
                        }.into_view()
                    } else {
                        view! {
                            <span>
                                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <circle cx="12" cy="12" r="5"></circle>
                                    <line x1="12" y1="1" x2="12" y2="3"></line>
                                    <line x1="12" y1="21" x2="12" y2="23"></line>
                                    <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
                                    <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
                                    <line x1="1" y1="12" x2="3" y2="12"></line>
                                    <line x1="21" y1="12" x2="23" y2="12"></line>
                                    <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
                                    <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
                                </svg>
                            </span>
                        }.into_view()
                    }
                }}
            </button>
        </div>
    }
}