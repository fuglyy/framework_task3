mod config;

// Объявляем модули с помощью crate::domain::models
// Публичный импорт AppConfig напрямую из domain::models
pub use crate::domain::models::AppConfig;
use crate::domain::errors::AppError;

// Публичный ре-экспорт нужных элементов
pub use self::config::load_config;
