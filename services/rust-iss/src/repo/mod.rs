pub mod pg_pool; // Для инициализации пула
pub mod iss_repo;
pub mod osdr_repo;
// pub mod cache_repo; // Если вы его создали

// Публичный ре-экспорт (для service layer)
pub use iss_repo::*;
pub use osdr_repo::*;