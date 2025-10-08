
pub mod auth_functions;
pub mod class_functions;
pub mod class_qr;
pub mod classes;
pub mod edit_class;
pub mod edit_modules;
pub mod error;
pub mod helpers;
pub mod home;
pub mod login;
pub mod module_functions;
pub mod new_class;
pub mod new_module;
pub mod profile;
pub mod profile_functions;
pub mod register;
pub mod statistics;
pub mod stats_functions;
pub mod student_functions;
pub mod timetable;


pub use class_qr::{ClassQrFullscreenPage, ClassQrPage};
pub use classes::ClassesPage;
pub use edit_class::EditClass;
pub use edit_modules::EditModule;
pub use error::Error;
pub use home::HomePage;
pub use login::Login;
pub use new_class::NewClass;
pub use new_module::NewModule;
pub use profile::Profile;
pub use register::Register;
pub use statistics::Statistics;
pub use timetable::Timetable;
