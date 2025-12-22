//! 表定义
//!
//! 定义数据库表结构，包含表 ID、名称、列定义、主键和根页信息

use crate::column::Column;
use crate::ids::{ColumnId, PageId, TableId};

/// 表实体
///
/// 定义数据库表，包含表 ID、名称、列定义、主键和根页
///
/// 不变量:
/// - name 非空
/// - columns 非空
/// - primary_key 如果存在，必须引用 columns 中的列
/// - root_page 必须有效
///
/// 生命周期: 'static
/// 线程安全: Send + Sync
#[derive(Debug, Clone)]
pub struct Table {
  pub id: TableId,
  pub name: String,
  pub columns: Vec<Column>,
  pub primary_key: Option<ColumnId>,
  /// B+Tree 根页 ID
  pub root_page: PageId,
}

impl Table {
  /// 创建新表
  ///
  /// 使用给定的表 ID、名称、列和根页创建表。
  /// 主键可以从列的约束中自动检测，或者通过 `primary_key` 参数显式指定。
  ///
  /// # Examples
  ///
  /// use rdb_domain::{Table, TableId, PageId, Column, ColumnId, DataType};
  ///
  /// let columns = vec![
  ///   Column::new(ColumnId::new(1), "id".to_string(), DataType::Integer),
  /// ];
  ///
  /// let table = Table::new(
  ///   TableId::new(1),
  ///   "users".to_string(),
  ///   columns,
  ///   None,
  ///   PageId::new(1),
  /// );
  ///
  pub fn new(
    id: TableId,
    name: String,
    columns: Vec<Column>,
    primary_key: Option<ColumnId>,
    root_page: PageId,
  ) -> Self {
    Self { id, name, columns, primary_key, root_page }
  }

  /// 查找列（按名称）
  ///
  /// 返回订一个匹配名称的列的引用
  ///
  /// # Examples
  ///
  /// use rdb_domain::{Table, TableId, PageId, Column, ColumnId, DataType};
  ///
  /// let columns = vec![
  ///   Column::new(ColumnId::new(1), "id".to_string(), DataType::Integer)
  /// ];
  ///
  /// let table = Table::new(
  ///   TableId::new(1),
  ///   "users".to_string(),
  ///   columns,
  ///   None,
  ///   PageId::new(1),
  /// );
  ///
  /// assert!(table.get_column("id").is_some());
  /// assert!(table.get_column("nonexistent").is_none());
  pub fn get_column(&self, name: &str) -> Option<&Column> {
    self.columns.iter().find(|col| col.name == name)
  }

  /// 获取主键列
  ///
  /// 如果表有主键，返回主键的引用
  ///
  /// # Examples
  ///
  /// use rdb_domain::{Table, TableId, PageId, Column, ColumnId, DataType};
  ///
  /// let columns = vec![
  ///   Column::new(ColumnId::new(1), "id".to_string(), DataType::Integer),
  /// ];
  ///
  /// let table = Table::new(
  ///   TableId::new(1),
  ///   "users".to_string(),
  ///   columns,
  ///   Some(ColumnId::new(1)),
  ///   PageId::new(1),
  /// );
  ///
  /// assert!(table.primary_key_column().is_some());
  pub fn primary_key_column(&self) -> Option<&Column> {
    self
      .primary_key
      .and_then(|pk_id| self.columns.iter().find(|col| col.id == pk_id))
  }
}

// 取保 Table 是 Send + Sync
unsafe impl Send for Table {}
unsafe impl Sync for Table {}
