//! 值对象
//!
//! 定义数据库中的值对象，支持四种基本类型:
//! - `Null`: NULL 值
//! - `Integer`: 64-bit 整数
//! - `Real`: 64-bit 浮点数
//! - `Text`: UTF-8 字符串(使用 Cow 避免拷贝)
//! - `Blob`: 二进制数据(使用 Cow 避免拷贝)

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cmp::Ordering;

use crate::data_type::DataType;

/// 值对象：数据库值
///
/// 表示数据库中的单个值，支持四种基础类型。
/// 使用 `Cow` 来避免不必要的拷贝，可以持有借用数据或拥有数据。
///
/// 声明周期: 'v (可能引用外部数据，避免拷贝)
/// 线程安全: Send + Sync
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value<'v> {
  /// NULL 值
  Null,
  /// 64-bit 整数
  Integer(i64),
  /// 64-bit 浮点数
  Real(f64),
  /// UTF-8 字符串(使用 Cow 避免拷贝)
  Text(#[serde(borrow)] Cow<'v, str>),
  /// 二进制数据(使用 Cow 避免拷贝)
  Blob(#[serde(borrow)] Cow<'v, [u8]>),
}

// 确保 Value 是 Send + Sync
unsafe impl<'v> Send for Value<'v> {}
unsafe impl<'v> Sync for Value<'v> {}

impl<'v> Value<'v> {
  /// 转换为所有权的值
  ///
  /// 将借用数据克隆为拥有数据，返回 `Value<'static>`。
  ///
  /// # Examples
  ///
  /// use rdb_domain::Value;
  /// use std::borrow::Cow;
  ///
  /// let value = Value::Text(Cow::Borrowed("hello"));
  /// let owned = value.into_owned();
  ///
  pub fn into_owned(self) -> Value<'static> {
    match self {
      Value::Null => Value::Null,
      Value::Integer(i) => Value::Integer(i),
      Value::Real(r) => Value::Real(r),
      Value::Text(cow) => Value::Text(Cow::Owned(cow.into_owned())),
      Value::Blob(cow) => Value::Blob(Cow::Owned(cow.into_owned())),
    }
  }

  /// 获取值的类型
  ///
  /// 返回对应的 `DataType`。
  ///
  /// # Examples
  ///
  /// use rdb_domain::{DataType, Value};
  ///
  /// assert_eq!(Value::Null.data_type(), DataType::Integer);
  /// assert_eq!(Value::Integer(123).data_type(), DataType::Integer);
  /// assert_eq!(Value::Real(3.14).data_type(), DataType::Real);
  ///
  pub fn data_type(&self) -> DataType {
    match self {
      Value::Null => DataType::Integer, // NULL 在 SQLite 中通常关联到 Integer
      Value::Integer(_) => DataType::Integer,
      Value::Real(_) => DataType::Real,
      Value::Text(_) => DataType::Text,
      Value::Blob(_) => DataType::Blob,
    }
  }

  /// 尝试转换为 i64
  ///
  /// 如果值是 `Integer`, 返回 `Some(i64)`, 否则返回 `None`。
  ///
  /// # Examples
  ///
  /// use rdb_domain::Value;
  ///
  /// assert_eq!(Value::Integer(123).as_integer(), Some(123));
  /// assert_eq!(Value::Real(3.14).as_integer(), None);
  ///
  pub fn as_integer(&self) -> Option<i64> {
    match self {
      Value::Integer(i) => Some(*i),
      _ => None,
    }
  }

  /// 尝试转换为 f64
  ///
  /// 如果值是 `Real`, 返回 `Some(f64)`, 否则返回 `None`。
  ///
  /// # Examples
  ///
  /// use rdb_domain::Value;
  ///
  /// assert_eq!(Value::Real(3.14).as_real(), Some(3.14));
  /// assert_eq!(Value::Integer(123).as_real(), None);
  ///
  pub fn as_real(&self) -> Option<f64> {
    match self {
      Value::Real(r) => Some(*r),
      _ => None,
    }
  }

  /// 尝试转换为 &str
  ///
  /// 如果值是 `Text`, 返回 `Some(&str)`, 否则返回 `None`。
  ///
  /// # Examples
  ///
  /// use rdb_domain::Value;
  /// use std::borrow::Cow;
  ///
  /// let value = Value::Text(Cow::Borrowed("hello"));
  /// assert_eq!(value.as_text(), Some("hello"));
  /// assert_eq!(Value::Real(3.14).as_text(), None);
  ///
  pub fn as_text(&self) -> Option<&str> {
    match self {
      Value::Text(s) => Some(s.as_ref()),
      _ => None,
    }
  }

  /// 尝试转换为 &[u8]
  ///
  /// 如果值是 `Blob`, 返回 `Some(&[u8])`, 否则返回 `None`。
  ///
  /// # Examples
  ///
  /// use rdb_domain::Value;
  /// use std::borrow::Cow;
  ///
  /// let value = Value::Blob(Cow::Borrowed(b"hello"));
  /// assert_eq!(value.as_blob(), Some(b"hello" as &[u8]));
  ///
  pub fn as_blob(&self) -> Option<&[u8]> {
    match self {
      Value::Blob(cow) => Some(cow.as_ref()),
      _ => None,
    }
  }

  /// SQL 语义比较（NULL != NULL)
  ///
  /// 按照 SQL 的语义进行比较
  /// - 如果任一值为 NULL，返回 `None` (NULL 与任何值比较都返回 NULL)
  /// - 否则返回 `Some(Ordering`
  ///
  /// # Examples
  ///
  /// use rdb_domain::Value;
  /// use std::cmp::Ordering;
  ///
  /// assert_eq!(Value::Integer(1).sql_compare(&Value::Integer(2)), Some(Ordering::Less));
  /// assert_eq!(Value::Null.sql_compare(&Value::Integer(1)), None);
  /// assert_eq!(Value::Null.sql_compare(&Value::Null), None);
  ///
  pub fn sql_compare(&self, other: &Self) -> Option<Ordering> {
    match (self, other) {
      // NULL 与任何值比较都返回 NULL
      (Value::Null, _) | (_, Value::Null) => None,
      // 同类型比较
      (Value::Integer(a), Value::Integer(b)) => Some(a.cmp(b)),
      (Value::Real(a), Value::Real(b)) => a.partial_cmp(b),
      (Value::Text(a), Value::Text(b)) => Some(a.cmp(b)),
      (Value::Blob(a), Value::Blob(b)) => Some(a.cmp(b)),
      // 其他类型无法比较
      _ => None,
    }
  }
}
