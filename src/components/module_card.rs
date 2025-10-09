use crate::utils::module_visuals::{module_visual, ModuleVisual};
use leptos::ev::{KeyboardEvent, MouseEvent};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn ModuleCard(
    code: String,
    name: String,
    desc: String,
    students: i32,
    class_count: i32,
    module_code: String,
) -> impl IntoView {
    let navigate = use_navigate();

    let ModuleVisual { variant, label } = module_visual(&module_code);
    let icon_classes = format!("module-icon {}", variant);
    let href = format!("/classes?module={}", module_code.clone());

    let go_card = {
        let href = href.clone();
        let navigate = navigate.clone();
        move |_: MouseEvent| {
            navigate(&href, Default::default());
        }
    };

    let go_card_key = {
        let href = href.clone();
        let navigate = navigate.clone();
        move |e: KeyboardEvent| {
            let k = e.key();
            if k == "Enter" || k == " " {
                e.prevent_default();
                navigate(&href, Default::default());
            }
        }
    };

    let go_new_class = {
        let module_code = module_code.clone();
        move |e: MouseEvent| {
            e.stop_propagation();
            e.prevent_default();
            navigate(
                &format!("/classes/new?module={}", module_code),
                Default::default(),
            );
        }
    };

    view! {
        <div
            class="module-card-link"
            role="link"
            tabindex="0"
            on:click=go_card
            on:keydown=go_card_key
        >
            <div class="card module-card">
                <div class=icon_classes aria-hidden="true">{label.clone()}</div>
                <div class="module-body">
                    <div class="module-code">{code}</div>
                    <div class="module-name">{name}</div>
                    <p class="module-desc">
                        {if desc.is_empty() {
                            "No description available".to_string()
                        } else {
                            desc
                        }}
                    </p>
                    <div class="module-meta">
                        <span class="meta-left">
                            <span aria-hidden="true">"👥"</span>
                            <span class="muted">{students} " students"</span>
                        </span>
                        <button
                            class="btn btn-primary btn-small"
                            on:click=go_new_class
                        >
                            "+ Add Class"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
