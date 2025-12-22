//! 数据类型定义
//!
//! 定义数据库支持的基础数据类型，遵循 SQLite 的类型系统

use crate::Value;

/// 数据类型值对象
///
/// 定义列的数据类型，支持四种基础类型:
/// - `Integer`: 64-bit 整数
/// - `Real`: 64-bit 浮点数
/// - `Text`: UTF-8 字符串
/// - `Blob`: 二进制数据
///
/// 生命周期: 'static
/// 线程安全: Send + Sync
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataType {
  /// INTEGER 类型（64-bit）
  Integer,
  /// REAL 类型（64-bit 浮点）
  Real,
  /// TEXT 类型（UTF-8 字符串）
  Text,
  /// BLOB 类型（二进制数据）
  Blob,
}

impl DataType {
  /// 从 SQL 类型名解析
  ///
  /// 支持 SQLite 兼容的类型名:
  /// - `INTEGER`, `INT` -> `Integer`
  /// - `REAL`, `FLOAT`, `DOUBLE`, `DOUBLE PRECISION` -> `Real`
  /// - `TEXT`, `VARCHAR`, `CHAR`, `STRING` -> `Text`
  /// - `BLOB`, `BINARY` -> `Blob`
  ///
  /// # Examples
  ///
  /// use rdb_domain::DataType;
  ///
  /// assert_eq!(DataType::from_sql_type("INTEGER"), Some(DataType::Integer));
  /// assert_eq!(DataType::from_sql_type("REAL"), Some(DataType::Real));
  /// assert_eq!(DataType::from_sql_type("TEXT"), Some(DataType::Text));
  /// assert_eq!(DataType::from_sql_type("BLOB"), Some(DataType::Blob));
  ///
  pub fn from_sql_type(sql_type: &str) -> Option<Self> {
    match sql_type.to_uppercase().trim() {
      "INTEGER" | "INT" => Some(Self::Integer),
      "REAL" | "FLOAT" | "DOUBLE" | "DOUBLE PRECISION" => Some(Self::Real),
      "TEXT" | "VARCHAR" | "CHAR" | "STRING" => Some(Self::Text),
      "BLOB" | "BINARY" => Some(Self::Blob),
      _ => None,
    }
  }

  /// 转换为 SQL 类型名
  ///
  /// 返回标准的 SQL 类型名（大写）。
  ///
  /// # Examples
  ///
  /// use rdb_domain::DataType;
  ///
  /// assert_eq!(DataType::Integer.to_sql_type(), "INTEGER");
  /// assert_eq!(DataType::Real.to_sql_type(), "REAL");
  /// assert_eq!(DataType::Text.to_sql_type(), "TEXT");
  /// assert_eq!(DataType::Blob.to_sql_type(), "BLOB");
  ///
  pub fn to_sql_type(&self) -> &'static str {
    match self {
      DataType::Integer => "INTEGER",
      DataType::Real => "REAL",
      DataType::Text => "TEXT",
      DataType::Blob => "BLOB",
    }
  }

  /// 检查值是否匹配此类型
  ///
  /// 注意：此方法需要 `Value` 类型已实现。当前为占位。
  /// 等 `Value` 类型实现后（T017），需要根据 `Value` 的实际类型进行匹配。
  ///
  /// # Examples
  ///
  /// use rdb_domain::{DataType, Value};
  ///
  /// let value = Value::Integer(123);
  /// assert!(DataType::Integer.matches(&value));
  ///
  pub fn matches<'v>(&self, value: &Value<'v>) -> bool {
    match value {
      Value::Null => true,
      Value::Integer(_) => matches!(self, DataType::Integer),
      Value::Real(_) => matches!(self, DataType::Real),
      Value::Text(_) => matches!(self, DataType::Text),
      Value::Blob(_) => matches!(self, DataType::Blob),
    }
  }
}
