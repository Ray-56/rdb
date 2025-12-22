//! 行数据
//!
//! 定义数据库表中的行数据，包含行 ID 和列值

use crate::ids::RowId;
use crate::table::Table;
use crate::value::Value;

/// 行实体
///
/// 表示数据库表中的一行数据，包含行 ID 和列值。
///
/// 不变量：
/// - values 长度必须等于表的列数（验证在 Table 层进行）
/// - 主键列不能为 NULL （如果有，验证在 Table 层进行）
///
/// 生命周期: 'r (可能引用外部数据)
/// 线程安全: Send + Sync
#[derive(Debug, Clone)]
pub struct Row<'r> {
  /// 行 ID （等同于 INTEGER PRIMARY KEY)
  pub row_id: RowId,
  /// 列值
  pub values: Vec<Value<'r>>,
}

impl<'r> Row<'r> {
  /// 创建新行
  ///
  /// 使用给定的行 ID 和值创建行。
  ///
  /// # Examples
  ///
  /// use rdb_domain::{Row, RowId, Value};
  /// use std::borrow::Cow;
  ///
  /// let values = vec![Value::Integer(1), Value::Text(Cow::Borrowed("Alice"))];
  ///
  /// let row = Row::new(RowId::new(1), values);
  /// assert_eq!(row.row_id, RowId::new(1));
  /// assert_eq!(row.values.len(), 2);
  pub fn new(row_id: RowId, values: Vec<Value<'r>>) -> Self {
    Self { row_id, values }
  }

  /// 获取列值（按索引）
  ///
  /// 返回指定索引位置的值的引用，如果索引超出范围则返回 `None`。
  ///
  /// # Examples
  ///
  /// use rdb_domain::{Row, Value};
  /// use std::borrow::Cow;
  ///
  /// let values = vec![Value::Integer(1), Value::Text(Cow::Borrowed("Alice"))];
  ///
  /// let row = Row::new(RowId::new(1), values);
  /// assert_eq!(row.get(0), Some(&Value::Integer(1)));
  /// assert_eq!(row.get(1), Some(&Value::Text(Cow::Borrowed("Alice"))));
  /// assert_eq!(row.get(2), None);
  pub fn get(&self, index: usize) -> Option<&Value<'r>> {
    self.values.get(index)
  }

  /// 获取列值（按列名）
  ///
  /// 根据列名从表中查找对应的列，然后返回改列的值。
  /// 如果列不存在或者索引超出范围，则返回 `None`。
  ///
  /// # Examples
  ///
  /// use rdb_domain::{Row, RowId, Value, Table, TableId, PageId, Column, ColumnId, DataType};
  /// use std::borrow::Cow;
  ///
  /// let columns = vec![
  ///   Column::new(ColumnId::new(1), "id".to_string(), DataType::Integer),
  ///   Column::new(ColumnId::new(2), "name".to_string(), DataType::Text),
  /// ];
  ///
  /// let table = Table::new(TableId::new(1), "users".to_string(), columns, None, PageId::new(1));
  ///
  /// let values = vec![Value::Integer(1), Value::Text(Cow::Borrowed("Alice"))];
  ///
  /// let row = Row::new(RowId::new(1), values);
  /// assert_eq!(row.get_by_name("id", &table), Some(&Value::Integer(1)));
  /// assert_eq!(row.get_by_name("name", &table), Some(&Value::Text(Cow::Borrowed("Alice"))));
  /// assert_eq!(row.get_by_name("nonexistent", &table), None);
  pub fn get_by_name(&self, column_name: &str, table: &Table) -> Option<&Value<'r>> {
    table
      .columns
      .iter()
      .position(|col| col.name == column_name)
      .and_then(|index| self.get(index))
  }

  /// 转换为所有权的行
  ///
  /// 将借用数据克隆为拥有数据，返回 `Row<'static>`。
  ///
  /// # Examples
  ///
  /// use rdb_domain::{Row, RowId, Value};
  /// use std::borrow::Cow;
  ///
  /// let values = vec![Value::Text(Cow::Borrowed("Hello"))];
  ///
  /// let row = Row::new(RowId::new(1), values);
  /// let owned = row.into_owned();
  ///
  pub fn into_owned(self) -> Row<'static> {
    Row { row_id: self.row_id, values: self.values.into_iter().map(|v| v.into_owned()).collect() }
  }
}

// 保证 Row 是 Send + Sync
unsafe impl<'r> Send for Row<'r> {}
unsafe impl<'r> Sync for Row<'r> {}
