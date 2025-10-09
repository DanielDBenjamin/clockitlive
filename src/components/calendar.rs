use crate::utils::module_visuals::module_visual;
use chrono::{Datelike, Duration, Local, NaiveDate};
use leptos::prelude::*;

#[component]
pub fn Calendar(
    #[prop(into)] classes: Signal<Vec<crate::database::classes::Class>>,
    #[prop(into)] on_date_select: Callback<String>,
) -> impl IntoView {
    let today = Local::now().naive_local().date();
    let current_month = RwSignal::new(today);
    let selected_date = RwSignal::new(today.format("%Y-%m-%d").to_string());

    // Fix: Use move closures to capture the signal properly
    let prev_month = move |_| {
        current_month.update(|date| {
            let new_date = *date - Duration::days(1);
            *date = NaiveDate::from_ymd_opt(new_date.year(), new_date.month(), 1).unwrap();
        });
    };

    let next_month = move |_| {
        current_month.update(|date| {
            let next = if date.month() == 12 {
                NaiveDate::from_ymd_opt(date.year() + 1, 1, 1).unwrap()
            } else {
                NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1).unwrap()
            };
            *date = next;
        });
    };

    let days_in_month = move || {
        let month = current_month.get();
        let first_day = NaiveDate::from_ymd_opt(month.year(), month.month(), 1).unwrap();
        let first_weekday = first_day.weekday().num_days_from_sunday() as usize;

        let days_in_current = if month.month() == 12 {
            NaiveDate::from_ymd_opt(month.year() + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(month.year(), month.month() + 1, 1)
        }
        .unwrap()
        .signed_duration_since(first_day)
        .num_days() as usize;

        (first_weekday, days_in_current)
    };

    let month_name = move || {
        let month = current_month.get();
        format!(
            "{} {}",
            match month.month() {
                1 => "January",
                2 => "February",
                3 => "March",
                4 => "April",
                5 => "May",
                6 => "June",
                7 => "July",
                8 => "August",
                9 => "September",
                10 => "October",
                11 => "November",
                12 => "December",
                _ => "Unknown",
            },
            month.year()
        )
    };

    let class_variant_for_day = move |day: u32| -> Option<&'static str> {
        let month = current_month.get();
        let date_str = format!("{:04}-{:02}-{:02}", month.year(), month.month(), day);
        classes
            .get()
            .iter()
            .find(|c| c.date == date_str)
            .map(|class| module_visual(&class.module_code).variant)
    };

    let select_date = move |day: u32| {
        let month = current_month.get();
        let date_str = format!("{:04}-{:02}-{:02}", month.year(), month.month(), day);
        selected_date.set(date_str.clone());
        on_date_select.run(date_str);
    };

    view! {
        <div class="calendar">
            <div class="calendar-header">
                <button class="cal-nav" aria-label="Previous Month" on:click=prev_month>"‹"</button>
                <div class="month">{month_name}</div>
                <button class="cal-nav" aria-label="Next Month" on:click=next_month>"›"</button>
            </div>
            <div class="calendar-grid">
                <div class="dow">"S"</div>
                <div class="dow">"M"</div>
                <div class="dow">"T"</div>
                <div class="dow">"W"</div>
                <div class="dow">"T"</div>
                <div class="dow">"F"</div>
                <div class="dow">"S"</div>

                {move || {
                    let (first_weekday, days_in_current) = days_in_month();
                    let month = current_month.get();
                    let today_str = today.format("%Y-%m-%d").to_string();

                    (0..42).map(|i| {
                        if i < first_weekday || i >= first_weekday + days_in_current {
                            view! { <div class="day day-empty"></div> }.into_any()
                        } else {
                            let day = (i - first_weekday + 1) as u32;
                            let date_str = format!("{:04}-{:02}-{:02}", month.year(), month.month(), day);
                            let is_today = date_str == today_str;
                            let is_selected = date_str == selected_date.get();
                            let dot_variant = class_variant_for_day(day).map(|v| v.to_string());

                            let class_names = if is_selected {
                                "day day-selected"
                            } else if is_today {
                                "day day-today"
                            } else {
                                "day"
                            };

                            view! {
                                <div
                                    class=class_names
                                    on:click=move |_| select_date(day)
                                >
                                    <span class="day-number">{day}</span>
                                    {match dot_variant {
                                        Some(variant) => view! { <span class=format!("day-dot {}", variant)></span> }.into_any(),
                                        None => view! { <></> }.into_any(),
                                    }}
                                </div>
                            }.into_any()
                        }
                    }).collect_view()
                }}
            </div>
        </div>
    }
}
