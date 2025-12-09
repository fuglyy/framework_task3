pub mod models;
pub mod errors;
pub mod contracts;

// Публичный ре-экспорт для удобства
pub use models::AppState;
pub use errors::AppError;
pub use contracts::{NasaClientContract, OsdrRepoContract, SpaceServiceContract};