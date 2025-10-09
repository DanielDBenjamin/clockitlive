use crate::database::modules::{CreateModuleRequest, Module, ModuleWithStats, UpdateModuleRequest};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::database::{
    init_db_pool,
    modules::{
        create_module, delete_module, get_lecturer_modules_with_stats, get_tutor_modules_with_stats, get_module, update_module,
    },
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ModuleResponse {
    pub success: bool,
    pub message: String,
    pub module: Option<Module>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ModulesListResponse {
    pub success: bool,
    pub message: String,
    pub modules: Vec<ModuleWithStats>,
}

/// Create a new module
#[server(CreateModule, "/api")]
pub async fn create_module_fn(
    module_code: String,
    module_title: String,
    description: Option<String>,
    lecturer_email: String,
) -> Result<ModuleResponse, ServerFnError> {
    // Validate input
    if module_code.trim().is_empty() {
        return Ok(ModuleResponse {
            success: false,
            message: "Module code is required".to_string(),
            module: None,
        });
    }

    if module_title.trim().is_empty() {
        return Ok(ModuleResponse {
            success: false,
            message: "Module title is required".to_string(),
            module: None,
        });
    }

    // Initialize database connection
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let request = CreateModuleRequest {
        module_code: module_code.trim().to_string(),
        module_title: module_title.trim().to_string(),
        description: description.filter(|s| !s.trim().is_empty()),
    };

    match create_module(&pool, &lecturer_email, request).await {
        Ok(module) => Ok(ModuleResponse {
            success: true,
            message: "Module created successfully!".to_string(),
            module: Some(module),
        }),
        Err(e) => Ok(ModuleResponse {
            success: false,
            message: e,
            module: None,
        }),
    }
}

/// Get all modules for the current lecturer
#[server(GetLecturerModules, "/api")]
pub async fn get_lecturer_modules_fn(
    lecturer_email: String,
) -> Result<ModulesListResponse, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    match get_lecturer_modules_with_stats(&pool, &lecturer_email).await {
        Ok(modules) => Ok(ModulesListResponse {
            success: true,
            message: "Modules fetched successfully".to_string(),
            modules,
        }),
        Err(e) => Ok(ModulesListResponse {
            success: false,
            message: e,
            modules: vec![],
        }),
    }
}

/// Get all modules for the current tutor
#[server(GetTutorModules, "/api")]
pub async fn get_tutor_modules_fn(
    tutor_email: String,
) -> Result<ModulesListResponse, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    match get_tutor_modules_with_stats(&pool, &tutor_email).await {
        Ok(modules) => Ok(ModulesListResponse {
            success: true,
            message: "Modules fetched successfully".to_string(),
            modules,
        }),
        Err(e) => Ok(ModulesListResponse {
            success: false,
            message: e,
            modules: vec![],
        }),
    }
}

/// Get a single module by code
#[server(GetModule, "/api")]
pub async fn get_module_fn(module_code: String) -> Result<ModuleResponse, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    match get_module(&pool, &module_code).await {
        Ok(Some(module)) => Ok(ModuleResponse {
            success: true,
            message: "Module found".to_string(),
            module: Some(module),
        }),
        Ok(None) => Ok(ModuleResponse {
            success: false,
            message: "Module not found".to_string(),
            module: None,
        }),
        Err(e) => Ok(ModuleResponse {
            success: false,
            message: e,
            module: None,
        }),
    }
}

/// Update a module
#[server(UpdateModule, "/api")]
pub async fn update_module_fn(
    module_code: String,
    module_title: String,
    description: Option<String>,
) -> Result<ModuleResponse, ServerFnError> {
    if module_title.trim().is_empty() {
        return Ok(ModuleResponse {
            success: false,
            message: "Module title is required".to_string(),
            module: None,
        });
    }

    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    let request = UpdateModuleRequest {
        module_code,
        module_title: module_title.trim().to_string(),
        description: description.filter(|s| !s.trim().is_empty()),
    };

    match update_module(&pool, request).await {
        Ok(module) => Ok(ModuleResponse {
            success: true,
            message: "Module updated successfully!".to_string(),
            module: Some(module),
        }),
        Err(e) => Ok(ModuleResponse {
            success: false,
            message: e,
            module: None,
        }),
    }
}

/// Delete a module
#[server(DeleteModule, "/api")]
pub async fn delete_module_fn(module_code: String) -> Result<ModuleResponse, ServerFnError> {
    let pool = init_db_pool()
        .await
        .map_err(|e| ServerFnError::new(format!("Database connection failed: {}", e)))?;

    match delete_module(&pool, &module_code).await {
        Ok(_) => Ok(ModuleResponse {
            success: true,
            message: "Module deleted successfully!".to_string(),
            module: None,
        }),
        Err(e) => Ok(ModuleResponse {
            success: false,
            message: e,
            module: None,
        }),
    }
}
