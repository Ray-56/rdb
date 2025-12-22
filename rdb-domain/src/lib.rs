//! rbd 领域模型
//!
//! 本模块包含 rdb 数据库的核心领域模型，严格遵循 DDD 原则

pub mod column;
pub mod data_type;
pub mod database;
pub mod error;
pub mod ids;
pub mod row;
pub mod table;
pub mod value;

pub use column::{Column, ColumnConstraints};
pub use data_type::DataType;
pub use database::Database;
pub use error::DomainError;
pub use ids::{ColumnId, IndexId, LockId, PageId, RowId, TableId, TransactionId};
pub use row::Row;
pub use table::Table;
pub use value::Value;
