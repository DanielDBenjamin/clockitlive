use crate::database::classes::Class;
use crate::utils::module_visuals::{module_visual, ModuleVisual};
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn ClassList(#[prop(into)] classes: Signal<Vec<Class>>) -> impl IntoView {
    view! {
        <div class="class-list-container">
            <Show
                when=move || !classes.get().is_empty()
                fallback=|| view! {
                    <p class="no-classes">"No classes scheduled for this day"</p>
                }
            >
                <ul class="class-list">
                    {move || classes.get().into_iter().map(|class| {
                        let ModuleVisual { variant, .. } = module_visual(&class.module_code);
                        let class_id = class.class_id;
                        let module_code = class.module_code.clone();
                        let item_class = format!("class-item {}", variant);
                        let dot_class = format!("dot {}", variant);

                        view! {
                            <li class=item_class>
                                <A
                                    href=format!("/classes/edit?id={}&origin=home", class_id)
                                    attr:class="class-item-link"
                                >
                                    <span class=dot_class aria-hidden="true"></span>
                                    <div class="class-info">
                                        <div class="class-module-code">{module_code}</div>
                                        <div class="class-title">{class.title.clone()}</div>
                                        <div class="class-sub">
                                            {class.time.clone()}
                                            {class.venue.as_ref().map(|v| format!(" – {}", v)).unwrap_or_default()}
                                        </div>
                                    </div>
                                </A>
                            </li>
                        }
                    }).collect_view()}
                </ul>
            </Show>
        </div>
    }
}
