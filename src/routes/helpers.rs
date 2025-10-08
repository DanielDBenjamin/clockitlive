pub fn build_return_path(origin: Option<String>, module_code: &str) -> String {
    match origin.as_deref() {
        Some("timetable") => "/timetable".to_string(),
        Some("home") => "/home".to_string(),
        _ => {
            if module_code.is_empty() {
                "/classes".to_string()
            } else {
                format!("/classes?module={}", module_code)
            }
        }
    }
}
