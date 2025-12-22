//! 列定义
//!
//! 定义数据库表的列结构，包含列名、数据类型和约束

use crate::data_type::DataType;
use crate::ids::ColumnId;
use crate::value::Value;
use crate::DomainError;

/// 列约束
///
/// 定义列的约束条件，包括 NOT NULL、UNIQUE、PRIMARY KEY 和 AUTOINCREMENT。
/// 注意：AUTOINCREMENT 仅适用于 INTEGER PRIMARY KEY。
///
/// 生命周期: 'static
/// 线程安全: Send + Sync
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnConstraints {
  /// NOT NULL 约束
  pub not_null: bool,
  /// UNIQUE 约束
  pub unique: bool,
  /// PRIMARY KEY 约束
  pub primary_key: bool,
  /// AUTOINCREMENT (仅 INTEGER PRIMARY KEY)
  pub autoincrement: bool,
}

impl Default for ColumnConstraints {
  fn default() -> Self {
    Self { not_null: false, unique: false, primary_key: false, autoincrement: false }
  }
}

/// 列实体
///
/// 定义数据库表中的列，包含列 ID、名称、数据类型和约束。
///
/// 不变量:
/// - name 非空
/// - data_type 有效
/// - constraints 有效
///
/// 声明周期: 'static
/// 线程安全: Send + Sync
#[derive(Debug, Clone)]
pub struct Column {
  pub id: ColumnId,
  pub name: String,
  pub data_type: DataType,
  pub constraints: ColumnConstraints,
  pub default_value: Option<Value<'static>>,
}

impl Column {
  /// 创建新列
  ///
  /// 使用默认约束创建列。
  ///
  /// # Examples
  ///
  /// use rdb_domain::{Column, ColumnId, DataType};
  ///
  /// let column = Column::new(ColumnId::new(1), "id".to_string(), DataType::Integer);
  /// assert_eq!(column.name, "id");
  /// assert_eq!(column.data_type, DataType::Integer);
  ///
  pub fn new(id: ColumnId, name: String, data_type: DataType) -> Self {
    Self { id, name, data_type, constraints: ColumnConstraints::default(), default_value: None }
  }

  /// 创建带约束的列
  ///
  /// # Examples
  ///
  /// use rdb_domain::{Column, ColumnId, ColumnConstraints, DataType};
  ///
  /// let constraints = ColumnConstraints { not_null: true, primary_key: true, ..Default::default() };
  ///
  /// let column = Column::with_constraints(ColumnId::new(1), "id".to_string(), DataType::Integer, constraints);
  ///
  pub fn with_constraints(
    id: ColumnId,
    name: String,
    data_type: DataType,
    constraints: ColumnConstraints,
  ) -> Self {
    Self { id, name, data_type, constraints, default_value: None }
  }

  /// 验证值是否符合列定义
  ///
  /// 检查值是否:
  /// 1. 类型匹配列的数据类型
  /// 2. 满足 NOT NULL 约束（如果设置）
  ///
  /// # Examples
  ///
  /// use rdb_domain::{Column, ColumnId, ColumnConstraints, DataType, Value};
  /// use std::borrow::Cow;
  ///
  /// let mut column = Column::new(ColumnId::new(1), "id".to_string(), DataType::Integer);
  /// column.constraints.not_null = true;
  ///
  /// assert!(column.validate_value(&Value::Integer(123)).is_ok());
  /// assert!(column.validate_value(&Value::Null).is_err());
  ///
  pub fn validate_value(&self, value: &Value) -> Result<(), DomainError> {
    // 检查 NOT NULL 约束
    if self.constraints.not_null && matches!(value, Value::Null) {
      return Err(DomainError::NotNullViolation { name: self.name.clone() });
    }

    // 检查类型匹配
    let value_type = value.data_type();
    if value_type != self.data_type && !matches!(value, Value::Null) {
      return Err(DomainError::TypeMismatch {
        name: self.name.clone(),
        expected: self.data_type.to_sql_type().to_string(),
        got: value_type.to_sql_type().to_string(),
      });
    }

    Ok(())
  }
}

// 取保 Column 是 Send + Sync
unsafe impl Send for Column {}
unsafe impl Sync for Column {}
