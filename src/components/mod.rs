pub mod nav_bar;

pub use nav_bar::NavBar;

pub mod calendar;
pub mod class_list;
pub mod header;
pub mod module_card;
pub mod module_card_tailwind;
pub mod qr_scanner;
pub mod stat_tile;
pub mod top_bar;
pub mod clockit_logo;
pub mod theme_switcher;

pub use calendar::Calendar;
pub use class_list::ClassList;
pub use header::Header;
pub use module_card::ModuleCard as moduleCardOld;
pub use module_card_tailwind::ModuleCard;
pub use qr_scanner::QrScanner;
pub use stat_tile::StatTile;
pub use top_bar::TopBar;
pub use clockit_logo::*;
pub use theme_switcher::ThemeSwitcher;

