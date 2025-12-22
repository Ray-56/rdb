//! 领域错误类型
//!
//! 定义领域层可能出现的所有错误类型

use crate::ids::{ColumnId, IndexId, TableId};
use thiserror::Error;

/// 领域错误类型
///
/// 表示领域层操作中可能出现的错误，包括不变量违反、约束违反等
///
/// 线程安全: Send + Sync
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
  /// 表已存在
  #[error("Table '{name}' already exists")]
  TableAlreadyExists { name: String },

  /// 表不存在
  #[error("Table with ID {table_id:?} does not exist")]
  TableNotFound { table_id: TableId },

  /// 表名不存在
  #[error("Table '{name} does not exist")]
  TableNameNotFound { name: String },

  /// 表必须至少有一列
  #[error("Table must have at least one column")]
  TableMusthHaveColumns,

  /// 列不存在
  #[error("Column '{name}' does not exist in table")]
  ColumnNotFound { name: String },

  /// 列 ID 不存在
  #[error("Column with ID {column_id:?} does not exist in table")]
  ColumnIdNotFound { column_id: ColumnId },

  /// 约束违反：NOT NULL
  #[error("Column '{name}' does not allow NULL values")]
  NotNullViolation { name: String },

  /// 约束违反：类型不匹配
  #[error("Value type does not match column '{name}' type (expected: {expected:?}, got: {got:?})")]
  TypeMismatch { name: String, expected: String, got: String },

  /// 主键引用无效
  #[error("Primary key column {column_id:?} does not exist in table")]
  InvalidPrimaryKeyReference { column_id: ColumnId },

  /// 索引相关错误：表不存在
  #[error("Table with ID {table_id:?} does not exist for index")]
  IndexTableNotFound { table_id: TableId },

  /// 索引相关错误：列不存在
  #[error("Column with ID {column_id:?} does not exist in table for index")]
  IndexColumnNotFound { column_id: ColumnId },

  /// 索引相关错误：索引已存在
  #[error("Index with ID {index_id:?} already exists")]
  IndexAlreadyExists { index_id: IndexId },

  /// 索引相关错误：索引列不存在
  #[error("Index with ID {index_id:?} does not exist")]
  IndexNotFound { index_id: IndexId },

  /// 索引相关错误：索引名不存在
  #[error("Index '{name}' does not exist in table")]
  IndexNameNotFound { name: String },

  /// 索引名已存在（在同一表中）
  #[error("Index '{name}' already exists in table")]
  IndexNameAlreadyExists { name: String },

  /// 系统表不能被删除
  #[error("System table '{name}' cannot be dropped")]
  CannotDropSystemTable { name: String },

  /// 不变量违反（通用）
  #[error("Invariant violation: {message}")]
  InvariantViolation { message: String },
}
