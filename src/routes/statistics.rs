use crate::routes::stats_functions::*;
use crate::user_context::get_current_user;
use chrono::Utc;
use leptos::prelude::*;



#[component]
pub fn Statistics() -> impl IntoView {
    // Minimal, unstyled baseline to rebuild from scratch
    let current_user = get_current_user();

    let (selected_module, set_selected_module) = signal(None::<String>);
    let (timeframe, set_timeframe) = signal("Monthly".to_string());
    let now_ym = Utc::now().format("%Y-%m").to_string();
    let (selected_month, set_selected_month) = signal(now_ym.clone());
    let next_disabled = {
        let now = now_ym.clone();
        Signal::derive(move || selected_month.get() >= now)
    };

    // Data: modules
    let module_options = Resource::new(
        move || current_user.get().map(|u| u.email_address),
        |email| async move {
            match email {
                Some(email) => get_module_options(email).await,
                None => Err(ServerFnError::new("Not logged in".to_string())),
            }
        },
    );

    // Overview numbers
    let overview = Resource::new(
        move || {
            (
                current_user.get().map(|u| u.email_address),
                selected_module.get(),
            )
        },
        |(email, module)| async move {
            match email {
                Some(email) => get_overall_stats(email, module, None).await,
                None => Err(ServerFnError::new("Not logged in".to_string())),
            }
        },
    );

    // Trend points (weekly within month, or monthly YTD)
    let trend = Resource::new(
        move || {
            (
                current_user.get().map(|u| u.email_address),
                selected_module.get(),
                timeframe.get(),
                selected_month.get(),
            )
        },
        |(email, module, tf, month)| async move {
            match email {
                Some(email) => get_weekly_trends(email, module, Some(tf), Some(month)).await,
                None => Err(ServerFnError::new("Not logged in".to_string())),
            }
        },
    );

    // Monthly trend resource (for delta comparisons even when viewing Weekly)
    let monthly_trend = Resource::new(
        move || {
            (
                current_user.get().map(|u| u.email_address),
                selected_module.get(),
            )
        },
        |(email, module)| async move {
            match email {
                Some(email) => {
                    get_weekly_trends(email, module, Some("Monthly".to_string()), None).await
                }
                None => Err(ServerFnError::new("Not logged in".to_string())),
            }
        },
    );

    // Most missed modules (for all-modules view)
    let missed = Resource::new(
        move || {
            (
                current_user.get().map(|u| u.email_address),
                selected_module.get(),
            )
        },
        |(email, module)| async move {
            match email {
                Some(email) => get_most_missed_modules(email, module).await,
                None => Err(ServerFnError::new("Not logged in".to_string())),
            }
        },
    );

    // Student attendance for selected module (overall)
    let students_attendance = Resource::new(
        move || {
            (
                current_user.get().map(|u| u.email_address),
                selected_module.get(),
            )
        },
        |(email, module)| async move {
            match (email, module) {
                (Some(email), Some(code)) => get_module_student_attendance(email, code, None).await,
                _ => Err(ServerFnError::new("No context".to_string())),
            }
        },
    );

    let search_students = RwSignal::new(String::new());
    let (selected_student, set_selected_student) = signal(None::<(i64, String, String, f64)>);
    let student_detail = Resource::new(
        move || {
            (
                current_user.get().map(|u| u.email_address),
                selected_module.get(),
                selected_student.get().map(|t| t.0),
            )
        },
        |(email, module, student_id)| async move {
            match (email, module, student_id) {
                (Some(email), Some(code), Some(sid)) => {
                    get_student_module_attendance_detail(email, code, sid).await
                }
                _ => Err(ServerFnError::new("No student selected".to_string())),
            }
        },
    );

    view! {
        <section class="stats-v2">
            <div class="header">
                <h1>{move || {
                    if let Some(code) = selected_module.get() {
                        if let Some(Ok(list)) = module_options.get() {
                            if let Some(m) = list.into_iter().find(|m| m.module_code == code) {
                                return format!("Attendance Statistics - {}", m.module_title);
                            }
                        }
                        "Attendance Statistics".to_string()
                    } else {
                        "Attendance Statistics - All Modules".to_string()
                    }
                }}</h1>
                <p class="subtitle">"Monitor student attendance"</p>
            </div>

            <div class="filters">
                <label>"Module: "
                    <select on:change=move |ev| {
                        let v = event_target_value(&ev);
                        if v.is_empty() || v == "all" { set_selected_module.set(None); }
                        else { set_selected_module.set(Some(v)); }
                    }>
                        <option value="all">"All Modules"</option>
                        <Suspense fallback=|| view! { <option>"Loading..."</option> }>
                            {move || module_options.get().map(|opts| match opts {
                                Ok(list) => list
                                    .into_iter()
                                    .map(|m| view! { <option value={m.module_code.clone()}>{m.module_title}</option> })
                                    .collect_view(),
                                Err(_) => vec![view! { <option value={String::new()}>{"Error loading".to_string()}</option> }],
                            })}
                        </Suspense>
                    </select>
                </label>

                <label style="margin-left:12px;">"Timeframe: "
                    <select on:change=move |ev| set_timeframe.set(event_target_value(&ev))>
                        <option value="Monthly">"Monthly"</option>
                        <option value="Weekly">"Weekly"</option>
                    </select>
                </label>

                <button class="btn btn-outline" style="margin-left:12px;" on:click=move |_| {
                    #[cfg(feature = "hydrate")]
                    {
                        use leptos::task::spawn_local;
                        use wasm_bindgen::JsCast;
                        
                        spawn_local(async move {
                            if let Some(email) = current_user.get().map(|u| u.email_address) {
                                match export_attendance_data(
                                    email,
                                    selected_module.get(),
                                    Some(timeframe.get()),
                                    if timeframe.get() == "Weekly" { Some(selected_month.get()) } else { None }
                                ).await {
                                    Ok(csv_data) => {
                                        // Create and download CSV file
                                        let blob = web_sys::Blob::new_with_str_sequence_and_options(
                                            &js_sys::Array::of1(&wasm_bindgen::JsValue::from_str(&csv_data)),
                                            &{
                                                let options = web_sys::BlobPropertyBag::new();
                                                options.set_type("text/csv");
                                                options
                                            }
                                        ).unwrap();
                                        
                                        let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
                                        let window = web_sys::window().unwrap();
                                        let document = window.document().unwrap();
                                        let a = document.create_element("a").unwrap();
                                        a.set_attribute("href", &url).unwrap();
                                        
                                        let filename = if let Some(module) = selected_module.get() {
                                            format!("attendance_{}_{}.csv", module, chrono::Utc::now().format("%Y%m%d"))
                                        } else {
                                            format!("attendance_all_modules_{}.csv", chrono::Utc::now().format("%Y%m%d"))
                                        };
                                        a.set_attribute("download", &filename).unwrap();
                                        
                                        let html_element = a.dyn_into::<web_sys::HtmlElement>().unwrap();
                                        html_element.click();
                                        
                                        web_sys::Url::revoke_object_url(&url).unwrap();
                                    }
                                    Err(e) => {
                                        web_sys::console::error_1(&format!("Export failed: {}", e).into());
                                    }
                                }
                            }
                        });
                    }
                    #[cfg(not(feature = "hydrate"))]
                    {
                        // Server-side: do nothing or show message
                        leptos::logging::log!("Export only available in browser");
                    }
                }>
                    "📊 Export CSV"
                </button>

                <Show when=move || timeframe.get() == "Weekly">
                    <span style="margin-left:12px;">
                        <button class="btn btn-outline" on:click=move |_| {
                            let cur = selected_month.get();
                            let parts: Vec<i32> = cur.split('-').filter_map(|s| s.parse::<i32>().ok()).collect();
                            if parts.len() == 2 {
                                let mut y = parts[0];
                                let mut m = parts[1];
                                m -= 1; if m == 0 { m = 12; y -= 1; }
                                set_selected_month.set(format!("{:04}-{:02}", y, m));
                            }
                        }>{"Prev"}</button>
                        <span style="padding:0 8px;">{selected_month}</span>
                        <button class="btn btn-outline" disabled=move || next_disabled.get() on:click=move |_| {
                            let cur = selected_month.get();
                            let parts: Vec<i32> = cur.split('-').filter_map(|s| s.parse::<i32>().ok()).collect();
                            if parts.len() == 2 {
                                let mut y = parts[0];
                                let mut m = parts[1];
                                m += 1; if m == 13 { m = 1; y += 1; }
                                let next = format!("{:04}-{:02}", y, m);
                                let now = Utc::now().format("%Y-%m").to_string();
                                if next <= now { set_selected_month.set(next); }
                            }
                        }>{"Next"}</button>
                        <button class="btn btn-outline btn-small" style="margin-left:8px;" on:click=move |_| {
                            set_selected_module.set(None);
                            set_timeframe.set("Monthly".to_string());
                            let now = Utc::now().format("%Y-%m").to_string();
                            set_selected_month.set(now);
                        }>{"Reset"}</button>
                    </span>
                </Show>
            </div>

            <h2>"Key Metrics"</h2>
            <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                {move || overview.get().map(|res| match res {
                    Ok(s) => view! {
                        <div class="kpi-grid">
                            <div class="card">
                                <div class="kpi-title">"Overall Attendance" <span class="kpi-ico">"%"</span></div>
                                <div class="kpi-value">{format!("{:.1}%", s.attendance_rate)}</div>
                                <div class="kpi-meta" style=move || {
                                    if timeframe.get()=="Weekly" {
                                        if let Some(Ok(mpoints)) = monthly_trend.get() {
                                            // compare current month vs previous month
                                            let curr = selected_month.get();
                                            let parts: Vec<i32> = curr.split('-').filter_map(|s| s.parse::<i32>().ok()).collect();
                                            if parts.len()==2 {
                                                let (mut y, mut m) = (parts[0], parts[1]);
                                                let curr_label = format!("{:04}-{:02}", y, m);
                                                m -= 1; if m==0 { m=12; y-=1; }
                                                let prev_label = format!("{:04}-{:02}", y, m);
                                                let curr_val = mpoints.iter().find(|p| p.week==curr_label).map(|p| p.attendance_rate);
                                                let prev_val = mpoints.iter().find(|p| p.week==prev_label).map(|p| p.attendance_rate);
                                                if let (Some(c), Some(p)) = (curr_val, prev_val) {
                                                    if c - p >= 0.0 { "color:#10b981".to_string() } else { "color:#ef4444".to_string() }
                                                } else { String::new() }
                                            } else { String::new() }
                                        } else { String::new() }
                                    } else {
                                        if let Some(Ok(list)) = trend.get() {
                                            if list.len() >= 2 {
                                                let last = list[list.len()-1].attendance_rate;
                                                let prev = list[list.len()-2].attendance_rate;
                                                let diff = last - prev;
                                                if diff >= 0.0 { "color:#10b981".to_string() } else { "color:#ef4444".to_string() }
                                            } else { String::new() }
                                        } else { String::new() }
                                    }
                                }>{move || {
                                    if timeframe.get()=="Weekly" {
                                        if let Some(Ok(mpoints)) = monthly_trend.get() {
                                            let curr = selected_month.get();
                                            let parts: Vec<i32> = curr.split('-').filter_map(|s| s.parse::<i32>().ok()).collect();
                                            if parts.len()==2 {
                                                let (mut y, mut m) = (parts[0], parts[1]);
                                                let curr_label = format!("{:04}-{:02}", y, m);
                                                m -= 1; if m==0 { m=12; y-=1; }
                                                let prev_label = format!("{:04}-{:02}", y, m);
                                                let curr_val = mpoints.iter().find(|p| p.week==curr_label).map(|p| p.attendance_rate);
                                                let prev_val = mpoints.iter().find(|p| p.week==prev_label).map(|p| p.attendance_rate);
                                                if let (Some(c), Some(p)) = (curr_val, prev_val) {
                                                    let diff = c - p;
                                                    if diff >= 0.0 { format!("↑ +{:.1}% from last month", diff) } else { format!("↓ {:.1}% from last month", diff.abs()) }
                                                } else { String::new() }
                                            } else { String::new() }
                                        } else { String::new() }
                                    } else {
                                        if let Some(Ok(list)) = trend.get() {
                                            if list.len() >= 2 {
                                                let last = list[list.len()-1].attendance_rate;
                                                let prev = list[list.len()-2].attendance_rate;
                                                let diff = last - prev;
                                                if diff >= 0.0 { format!("↑ +{:.1}% from last month", diff) } else { format!("↓ {:.1}% from last month", diff.abs()) }
                                            } else { String::new() }
                                        } else { String::new() }
                                    }
                                }}</div>
                            </div>
                            <div class="card">
                                <div class="kpi-title">"Total Students" <span class="kpi-ico">"👥"</span></div>
                                <div class="kpi-value">{s.total_students}</div>
                                <div class="kpi-meta">{move || {
                                    if selected_module.get().is_none() {
                                        if let Some(Ok(mods)) = module_options.get() { format!("Across {} modules", mods.len()) } else { String::new() }
                                    } else { String::new() }
                                }}</div>
                            </div>
                            <div class="card">
                                <div class="kpi-title">"Avg Class Size" <span class="kpi-ico">"🏫"</span></div>
                                <div class="kpi-value">{format!("{:.0}", s.avg_class_size)}</div>
                                <div class="kpi-meta">"Students per class"</div>
                            </div>
                            <div class="card">
                                <div class="kpi-title">"Absent Today" <span class="kpi-ico">"⚠"</span></div>
                                <div class="kpi-value">{s.absent_today}</div>
                                <div class="kpi-meta">{move || if s.total_students>0 { format!("{:.1}% of total", (s.absent_today as f64)*100.0/(s.total_students as f64)) } else { String::new() }}</div>
                            </div>
                        </div>
                    }.into_any(),
                    Err(_) => view! { <div class="card">"Error loading overview"</div> }.into_any(),
                })}
            </Suspense>

            <div class="charts-grid" style="margin-top:16px;">
                <div class="chart-box">
                    <div style="display:flex; align-items:center; justify-content:space-between;">
                        <h3 style="margin:0;">{move || if timeframe.get()=="Weekly" { "Weekly Attendance".to_string() } else { "Monthly Attendance (YTD)".to_string() }}</h3>
                        <div style="display:flex; align-items:center; gap:6px; font-size:12px; color:#6b7280;">
                            <span style="display:inline-block; width:10px; height:10px; border-radius:9999px; background:#14b8a6;"></span>
                            <span>"Attendance %"</span>
                        </div>
                    </div>
                    <div class="box-body">
                    <Suspense fallback=|| view! { <div>"Loading trend..."</div> }>
                        {move || trend.get().map(|res| match res {
                            Ok(list) if !list.is_empty() => {
                                let width = 680.0_f64;
                                let height = 260.0_f64;
                                let padding = 40.0_f64;
                                let chart_w = width - 2.0*padding;
                                let chart_h = height - 2.0*padding;
                                let x_step = chart_w / (list.len() as f64 - 1.0).max(1.0);

                                let min_rate: f64 = list.iter().map(|t| t.attendance_rate).fold(100.0_f64, |a,b| a.min(b));
                                let max_rate: f64 = list.iter().map(|t| t.attendance_rate).fold(0.0_f64, |a,b| a.max(b));
                                let mut y_min = (min_rate - 5.0).floor().max(0.0);
                                let mut y_max = (max_rate + 5.0).ceil().min(100.0);
                                if (y_max - y_min) < 10.0 { y_max = (y_min + 10.0).min(100.0); }
                                let to_y = |v: f64| padding + chart_h * (1.0 - ((v - y_min)/(y_max - y_min)).max(0.0).min(1.0));

                                let points: Vec<(f64,f64)> = list.iter().enumerate().map(|(i, p)| {
                                    (padding + (i as f64)*x_step, to_y(p.attendance_rate))
                                }).collect();
                                let poly = points.iter().map(|(x,y)| format!("{},{}", x,y)).collect::<Vec<_>>().join(" ");

                                view! { <svg viewBox={format!("0 0 {} {}", width, height)} style="width:100%; height:auto; max-height:260px;">
                                    <line x1={padding.to_string()} y1={padding.to_string()} x2={padding.to_string()} y2={(height-padding).to_string()} stroke="#e5e7eb" stroke-width="2"/>
                                    <line x1={padding.to_string()} y1={(height-padding).to_string()} x2={(width-padding).to_string()} y2={(height-padding).to_string()} stroke="#e5e7eb" stroke-width="2"/>
                                    {(0..=4).map(|i|{
                                        let v = y_min + (i as f64)*(y_max-y_min)/4.0;
                                        let y = to_y(v);
                                        view! { <g>
                                            <line x1={padding.to_string()} y1={y.to_string()} x2={(width-padding).to_string()} y2={y.to_string()} stroke="#f3f4f6" stroke-width="1"/>
                                            <text x={(padding-16.0).to_string()} y={y.to_string()} text-anchor="end" alignment-baseline="middle" font-size="11" fill="#6b7280">{format!("{:.0}%", v)}</text>
                                        </g> }
                                    }).collect_view()}
                                    <polyline points={poly} fill="none" stroke="#14b8a6" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>
                                    {(0..list.len()).map(|i|{
                                        let x = padding + (i as f64)*x_step;
                                        let y = to_y(list[i].attendance_rate);
                                        let raw = list[i].week.clone();
                                        let lbl: String = if timeframe.get()=="Monthly" && raw.contains('-') {
                                            let parts: Vec<&str> = raw.split('-').collect();
                                            if parts.len()==2 { match parts[1] {"01"=>"Jan","02"=>"Feb","03"=>"Mar","04"=>"Apr","05"=>"May","06"=>"Jun","07"=>"Jul","08"=>"Aug","09"=>"Sep","10"=>"Oct","11"=>"Nov","12"=>"Dec", _=>raw.as_str()} .to_string() } else { raw.clone() }
                                        } else if timeframe.get()=="Weekly" { raw.replace("Week ", "W") } else { raw.clone() };
                                        let val = format!("{:.0}%", list[i].attendance_rate);
                                        let cls = list[i].class_count;
                                        view! { <g>
                                            <circle cx={x.to_string()} cy={y.to_string()} r="4" fill="#14b8a6"><title>{format!("{} — {}", lbl, val)}</title></circle>
                                            <text x={x.to_string()} y={(height - padding + 16.0).to_string()} text-anchor="middle" font-size="10" fill="#6b7280">{lbl.clone()}</text>
                                            {move || if timeframe.get()=="Weekly" {
                                                view! { <text x={x.to_string()} y={(height - padding + 28.0).to_string()} text-anchor="middle" font-size="9" fill="#9ca3af">{format!("{} cls", cls)}</text> }.into_any()
                                            } else { view! { <></> }.into_any() }}
                                            <text x={x.to_string()} y={(y - 10.0).to_string()} text-anchor="middle" font-size="10" fill="#374151">{val}</text>
                                        </g> }
                                    }).collect_view()}
                                </svg> }.into_any()
                            }
                            Ok(_) => view! { <div>"No trend data"</div> }.into_any(),
                            Err(_) => view! { <div>"Error loading trend"</div> }.into_any(),
                        })}
                    </Suspense>
                    </div>
                </div>
                <div class="chart-box">
                    {move || if selected_module.get().is_some() {
                        // Module-specific side panel: search, class list, and students
                        view! {
                            <h3>"Students in Module"</h3>
                            <div class="box-body">
                                <input class="input" placeholder="Search name or email..." bind:value=search_students style="margin-bottom:8px; width:100%; padding:8px 10px; border:1px solid var(--sidebar-border); border-radius:8px;"/>
                            <div class="muted" style="margin-bottom:6px;">"Students"</div>
                            <Suspense fallback=|| view! { <div>"Loading students..."</div> }>
                                {move || students_attendance.get().map(|res| match res {
                                    Ok(students) => {
                                        let q = search_students.get().to_lowercase();
                                        let mut filtered = students.into_iter().filter(|s| {
                                            if q.is_empty() { return true; }
                                            s.name.to_lowercase().contains(&q) || s.surname.to_lowercase().contains(&q) || s.email_address.to_lowercase().contains(&q)
                                        }).collect::<Vec<_>>();
                                        filtered.sort_by(|a,b| a.surname.cmp(&b.surname).then(a.name.cmp(&b.name)));
                                        if filtered.is_empty() {
                                            view! { <div class="muted">"No students match your search"</div> }.into_any()
                                        } else {
                                            view! { <div>
                                                {filtered.into_iter().map(|s| {
                                                    let rate = format!("{:.0}%", s.attendance_rate);
                                                    let sid = s.user_id;
                                                    let name = s.name.clone();
                                                    let surname = s.surname.clone();
                                                    let rate_val = s.attendance_rate;
                                                    view! { <>
                                                        <div class=move || if selected_student.get().map(|t| t.0)==Some(sid) { "student-row active" } else { "student-row" }
                                                            on:click=move |_| {
                                                                if selected_student.get().map(|t| t.0)==Some(sid) {
                                                                    set_selected_student.set(None);
                                                                } else {
                                                                    set_selected_student.set(Some((sid, name.clone(), surname.clone(), rate_val)));
                                                                }
                                                            }
                                                        >
                                                            <div>
                                                                <div>{format!("{} {}", s.name, s.surname)}</div>
                                                                <div class="muted">{s.email_address}</div>
                                                            </div>
                                                            <div>{rate}</div>
                                                        </div>
                                                        <Show when=move || selected_student.get().map(|t| t.0)==Some(sid)>
                                                            <Suspense fallback=|| view! { <div class="muted">"Loading student detail..."</div> }>
                                                                {move || student_detail.get().map(|res| match res {
                                                                    Ok(rows) if !rows.is_empty() => view! {
                                                                        <div class="list">
                                                                            {rows.into_iter().map(|row| {
                                                                                let badge = match row.status.as_str() {
                                                                                    "present" => "background:#d1fae5; color:#065f46;",
                                                                                    "late" => "background:#fef3c7; color:#92400e;",
                                                                                    _ => "background:#fee2e2; color:#991b1b;",
                                                                                };
                                                                                view! { <div class="list-item" style="display:flex; justify-content:space-between; align-items:center;">
                                                                                    <div>
                                                                                        <div>{row.title.clone()}</div>
                                                                                        <div class="muted">{format!("{} {}", row.date, row.time)}</div>
                                                                                    </div>
                                                                                    <span style=badge>{row.status}</span>
                                                                                </div> }
                                                                            }).collect_view()}
                                                                        </div>
                                                                    }.into_any(),
                                                                    Ok(_) => view! { <div class="muted">"No classes yet"</div> }.into_any(),
                                                                    Err(_) => view! { <div>"Error loading detail"</div> }.into_any(),
                                                                })}
                                                            </Suspense>
                                                        </Show>
                                                    </> }
                                                }).collect_view()}
                                            </div> }.into_any()
                                        }
                                    },
                                    Err(_) => view! { <div>"Error loading students"</div> }.into_any(),
                                })}
                            </Suspense>
                            </div>
                        }.into_any()
                    } else {
                        // All-modules view: keep Most Missed Modules
                        view! {
                            <h3>"Most Missed Modules"</h3>
                            <Suspense fallback=|| view! { <div>"Loading..."</div> }>
                                {move || missed.get().map(|res| match res {
                                    Ok(list) if !list.is_empty() => view! {
                                        <div class="bar-list">
                                            {list.into_iter().map(|m| {
                                                let w = format!("{}%", m.absence_rate.min(100.0));
                                                view! { <div class="bar-row">
                                                    <div class="bar-head"><span>{m.module_title}</span><span class="bar-value">{format!("{:.1}%", m.absence_rate)}</span></div>
                                                    <div class="bar-track"><div class="bar-fill" style:width=w></div></div>
                                                </div> }
                                            }).collect_view()}
                                        </div>
                                    }.into_any(),
                                    Ok(_) => view! { <div>"No data"</div> }.into_any(),
                                    Err(_) => view! { <div>"Error loading"</div> }.into_any(),
                                })}
                            </Suspense>
                        }.into_any()
                    }}
                </div>
            </div>
        </section>
    }
}
