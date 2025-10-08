use crate::routes::class_functions::create_class_fn;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_query_map};

#[component]
pub fn NewClass() -> impl IntoView {
    let navigate = use_navigate();
    let query = use_query_map();

    let title = RwSignal::new(String::new());
    let venue = RwSignal::new(String::new());
    let desc = RwSignal::new(String::new());
    let recurring = RwSignal::new("No repeat".to_string());
    let recurrence_count = RwSignal::new("8".to_string()); // Store as String for input binding
    let date = RwSignal::new(String::new());
    // Set default time to 09:00 (common class start time)
    let hour = RwSignal::new("09".to_string());
    let minute = RwSignal::new("00".to_string());
    
    // Helper function to format time with leading zeros and bounds checking
    let format_time_component = |value: &str, default: u32, max: u32| -> String {
        let num = value.parse::<u32>().unwrap_or(default).min(max);
        format!("{:02}", num)
    };
    let duration = RwSignal::new("90".to_string());
    let message = RwSignal::new(String::new());
    let success = RwSignal::new(false);

    // Get module code from URL query params
    let module_code = move || query.with(|q| q.get("module").unwrap_or_default());

    let create_action = Action::new(
        move |(
            module,
            title_val,
            venue_val,
            desc_val,
            recurring_val,
            date_val,
            time_val,
            duration_val,
            count,
        ): &(
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            String,
            String,
            i32,
            Option<i32>,
        )| {
            let module = module.clone();
            let title_val = title_val.clone();
            let venue_val = venue_val.clone();
            let desc_val = desc_val.clone();
            let recurring_val = recurring_val.clone();
            let date_val = date_val.clone();
            let time_val = time_val.clone();
            let duration_val = *duration_val;
            let count = *count;
            async move {
                create_class_fn(
                    module,
                    title_val,
                    venue_val,
                    desc_val,
                    recurring_val,
                    date_val,
                    time_val,
                    duration_val,
                    count,
                )
                .await
            }
        },
    );

    let on_submit = move |_| {
        message.set(String::new());
        success.set(false);

        let current_module = module_code();

        if current_module.is_empty() {
            message.set("No module selected. Please go back and select a module.".to_string());
            return;
        }

        if title.get().trim().is_empty() {
            message.set("Please enter a class title".to_string());
            return;
        }

        if date.get().trim().is_empty() {
            message.set("Please select a date".to_string());
            return;
        }

        let formatted_hour = format_time_component(&hour.get(), 9, 23);
        let formatted_minute = format_time_component(&minute.get(), 0, 59);
        let time_str = format!("{}:{}", formatted_hour, formatted_minute);

        let venue_val = if venue.get().trim().is_empty() {
            None
        } else {
            Some(venue.get())
        };
        let desc_val = if desc.get().trim().is_empty() {
            None
        } else {
            Some(desc.get())
        };
        let recurring_val = if recurring.get() == "No repeat" {
            None
        } else {
            Some(recurring.get())
        };

        // Only pass recurrence_count if recurring is enabled
        let count_val = if recurring.get() != "No repeat" {
            // Parse string to i32, default to 8 if invalid
            recurrence_count.get().parse::<i32>().ok()
        } else {
            None
        };

        let duration_val = duration.get().parse::<i32>().unwrap_or(90).max(15);

        create_action.dispatch((
            current_module,
            title.get(),
            venue_val,
            desc_val,
            recurring_val,
            date.get(),
            time_str,
            duration_val,
            count_val,
        ));
    };

    Effect::new({
        let navigate = navigate.clone();
        move |_| {
            if let Some(result) = create_action.value().get() {
                match result {
                    Ok(response) => {
                        message.set(response.message.clone());
                        success.set(response.success);

                        if response.success {
                            let nav = navigate.clone();
                            let mod_code = module_code();
                            set_timeout(
                                move || {
                                    nav(
                                        &format!("/classes?module={}", mod_code),
                                        Default::default(),
                                    );
                                },
                                std::time::Duration::from_millis(1500),
                            );
                        }
                    }
                    Err(e) => {
                        message.set(format!("Error: {}", e));
                        success.set(false);
                    }
                }
            }
        }
    });

    view! {
        <section class="new-class">
            <div class="page-header" style="display:flex;align-items:center;gap:8px;">
                <A href=move || format!("/classes?module={}", module_code()) attr:class="link">"←"</A>
                <h1 class="page-title">"New Class"</h1>
                <Show when=move || !module_code().is_empty()>
                    <p class="page-subtitle" style="margin-left:8px;">
                        {move || format!("for {}", module_code())}
                    </p>
                </Show>
            </div>

            <div class="form-card">
                <div class="form-grid">
                    <div class="form-col">
                        <label class="label">"Title"</label>
                        <input class="input" placeholder="Linked List Fundamentals – Lecture 4" bind:value=title />

                        <label class="label" style="margin-top:10px;">"Venue"</label>
                        <input class="input" placeholder="Room A101" bind:value=venue />

                        <label class="label" style="margin-top:10px;">"Description"</label>
                        <textarea class="textarea" placeholder="Enter a class description" bind:value=desc></textarea>

                        <label class="label" style="margin-top:10px;">"Recurring"</label>
                        <select class="input" bind:value=recurring>
                            <option selected>"No repeat"</option>
                            <option>"Daily"</option>
                            <option>"Weekly"</option>
                            <option>"Monthly"</option>
                        </select>

                        // Show recurrence count input if recurring is enabled
                        <Show when=move || recurring.get() != "No repeat">
                            <div style="margin-top:10px;">
                                <label class="label">"Number of Occurrences"</label>
                                <div style="display:flex; align-items:center; gap:12px;">
                                    <input
                                        type="number"
                                        class="input"
                                        min="2"
                                        max="52"
                                        bind:value=recurrence_count
                                        style="max-width:120px;"
                                    />
                                    <span class="muted" style="font-size:13px;">
                                        {move || match recurring.get().as_str() {
                                            "Daily" => "days",
                                            "Weekly" => "weeks",
                                            "Monthly" => "months",
                                            _ => "instances"
                                        }}
                                    </span>
                                </div>
                                <p class="muted" style="font-size:12px; margin-top:6px;">
                                    {move || {
                                        let count = recurrence_count.get().parse::<i32>().unwrap_or(8);
                                        let pattern = recurring.get();
                                        format!("This will create {} {} class instances", count,
                                            match pattern.as_str() {
                                                "Daily" => "daily",
                                                "Weekly" => "weekly",
                                                "Monthly" => "monthly",
                                                _ => ""
                                            }
                                        )
                                    }}
                                </p>
                            </div>
                        </Show>
                    </div>

                    <aside class="form-side">
                        <label class="label">"Date"</label>
                        <input class="input" type="date" bind:value=date />

                        <label class="label" style="margin-top:10px;">"Time"</label>
                        <div class="time-picker">
                            <div class="time-box">
                                <input 
                                    type="number" 
                                    min="0" 
                                    max="23" 
                                    bind:value=hour
                                    on:blur=move |_| {
                                        let formatted = format_time_component(&hour.get(), 9, 23);
                                        hour.set(formatted);
                                    }
                                />
                                <div class="t-label">"Hour"</div>
                            </div>
                            <div class="colon">":"</div>
                            <div class="time-box">
                                <input 
                                    type="number" 
                                    min="0" 
                                    max="59" 
                                    bind:value=minute
                                    on:blur=move |_| {
                                        let formatted = format_time_component(&minute.get(), 0, 59);
                                        minute.set(formatted);
                                    }
                                />
                                <div class="t-label">"Minute"</div>
                            </div>
                        </div>

                        <label class="label" style="margin-top:16px;">"Duration"</label>
                        <select class="input" bind:value=duration>
                            <option value="45">"45 minutes"</option>
                            <option value="60">"1 hour"</option>
                            <option value="75">"1 hour 15 min"</option>
                            <option value="90">"1 hour 30 min"</option>
                            <option value="105">"1 hour 45 min"</option>
                            <option value="120">"2 hours"</option>
                        </select>
                    </aside>
                </div>

                <Show when=move || !message.get().is_empty()>
                    <p class=move || if success.get() { "success center" } else { "error center" } style="margin-top:12px;">
                        {message}
                    </p>
                </Show>

                <div class="actions-row">
                    <button
                        class="btn btn-accent"
                        on:click=on_submit
                        disabled=move || create_action.pending().get()
                    >
                        {move || if create_action.pending().get() {
                            "Creating Class...".into_view()
                        } else {
                            "✓ Save Class".into_view()
                        }}
                    </button>
                    <A href="/home" attr:class="btn btn-outline">"× Cancel"</A>
                </div>
            </div>
        </section>
    }
}
