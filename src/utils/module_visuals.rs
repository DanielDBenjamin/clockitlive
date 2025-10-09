use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub struct ModuleVisual {
    pub variant: &'static str,
    pub label: String,
}

/// Derive a consistent color variant and label for module icons across the app.
const VARIANTS: [&str; 8] = [
    "mod-purp",
    "mod-blue",
    "mod-orange",
    "mod-green",
    "mod-rose",
    "mod-teal",
    "mod-amber",
    "mod-slate",
];

struct ModuleColorState {
    assignments: HashMap<String, usize>,
    next_index: usize,
}

impl ModuleColorState {
    fn get_or_assign(&mut self, key: &str) -> usize {
        if let Some(existing) = self.assignments.get(key) {
            *existing
        } else {
            let index = self.next_index;
            self.assignments.insert(key.to_string(), index);
            self.next_index = (self.next_index + 1) % VARIANTS.len();
            index
        }
    }
}

static MODULE_COLOR_STATE: OnceLock<Mutex<ModuleColorState>> = OnceLock::new();

pub fn module_visual(module_code: &str) -> ModuleVisual {
    let normalized = module_code.trim().to_uppercase();
    let state = MODULE_COLOR_STATE.get_or_init(|| {
        Mutex::new(ModuleColorState {
            assignments: HashMap::new(),
            next_index: 0,
        })
    });

    let mut guard = state.lock().expect("Module color state poisoned");
    let variant_index = guard.get_or_assign(&normalized);
    let variant = VARIANTS[variant_index];

    let alphanumeric: String = normalized
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(3)
        .collect();

    let label = if alphanumeric.is_empty() {
        "MOD".to_string()
    } else {
        alphanumeric.to_uppercase()
    };

    ModuleVisual { variant, label }
}
