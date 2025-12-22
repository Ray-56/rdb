//! 数据库聚合根
//!
//! 定义数据库实例，管理表、索引和全局状态

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::ids::{IndexId, TableId};
use crate::table::Table;
use crate::DomainError;

/// 数据库聚合根
///
/// 管理数据库的表、索引和全局状态。
///
/// 不变量：
/// - tables 和 indexes 必须保持一致
/// - 索引必须引用存在表
/// - schema_version 单调递增
/// - 表名在数据库中唯一
///
/// 生命周期: 'static (拥有所有数据)
/// 线程安全: 需要通过 Arc<Mutex<Database>> 共享
#[derive(Debug)]
pub struct Database {
  /// 数据库文件路径
  pub path: PathBuf,
  /// 表集合（表 ID -> 表定义）
  pub tables: HashMap<TableId, Table>,

  /// 索引集合（索引 ID -> 表定义）
  /// 注意: Index 类型尚未实现，暂时使用占位符
  /// TODO: 实现 Index 类型后替换为 HashMap<IndexId, Index>
  #[allow(dead_code)]
  pub indexes: HashMap<IndexId, ()>,
  /// 模式版本号（每次 DDL 操作递增）
  pub schema_version: u32,
}

impl Database {
  /// 创建新数据库实例
  ///
  /// 使用给定的路径创建新的空数据库
  ///
  /// # Arguments
  ///
  /// * `path` - 数据库文件路径
  ///
  /// # Examples
  ///
  /// use rdb_domain::Database;
  /// use std::path::Path;
  ///
  /// let db = Database::new(Path::new("/tmp/test.db"));
  ///
  pub fn new(path: impl AsRef<Path>) -> Self {
    Self {
      path: path.as_ref().to_path_buf(),
      tables: HashMap::new(),
      indexes: HashMap::new(),
      schema_version: 0,
    }
  }

  /// 添加表（DDL 操作）
  ///
  /// 将表添加到数据库中，并递增 schema_version。
  ///
  /// 不变量检查：
  /// - 表名必须唯一
  /// - 表必须至少有一列
  ///
  /// # Arguments
  ///
  /// * `table` - 要添加的表
  ///
  /// # Returns
  ///
  /// 返回表的 ID，如果表名已存在则返回错误
  ///
  /// # Examples
  ///
  /// use rdb_domain::{Database, Table, TableId, PageId, Column, ColumnId, DataType};
  /// use std::path::Path;
  ///
  /// let mut db = Database::new(Path::new("/tmp/test.db"));
  ///
  /// let columns = vec![Column::new(ColumnId::new(1), "id".to_string(), DataType::Integer)];
  ///
  /// let table = Table::new(Table::new(1), "users".to_string(), columns, None, PageId::new(1));
  ///
  /// let table_id = db.add_table(table)?;
  pub fn add_table(&mut self, table: Table) -> Result<TableId, DomainError> {
    // 检查表明是否唯一
    if self.tables.values().any(|t| t.name == table.name) {
      return Err(DomainError::TableAlreadyExists { name: table.name });
    }

    // 检查表是否有列
    if table.columns.is_empty() {
      return Err(DomainError::TableMusthHaveColumns);
    }

    let table_id = table.id;
    self.tables.insert(table_id, table);
    self.schema_version += 1;

    Ok(table_id)
  }

  /// 删除表（级联删除关联索引）
  ///
  /// 从数据库中删除表，并删除所有关联的索引
  ///
  /// # Arguments
  ///
  /// * `table_id` - 要删除的表 ID
  ///
  /// # Returns
  ///
  /// 如果表不存在则返回错误
  ///
  /// # Examples
  ///
  /// use rdb_domain::{Database, TableId};
  /// use std::path::Path;
  ///
  /// let mut db = Database::new(Path::new("/tmp/test.db"));
  ///
  /// db.drop_table(TableId::new(1)).unwrap();
  pub fn drop_table(&mut self, table_id: TableId) -> Result<(), DomainError> {
    if !self.tables.contains_key(&table_id) {
      return Err(DomainError::TableNotFound { table_id });
    }

    // 删除表
    self.tables.remove(&table_id);

    // TODO: 级联删除关联的索引
    // 当 Index 类型实现后，需要删除所有引用此表的索引
    // self.indexes.retain(|_, index| index.table_id != table_id);

    self.schema_version += 1;

    Ok(())
  }

  /// 获取表定义（不可变引用）
  ///
  /// # Arguments
  ///
  /// * `table_id` - 表 ID
  ///
  /// # Returns
  ///
  /// 如果表存在则返回表的引用，否则返回 None
  ///
  /// # Examples
  ///
  /// use rdb_domain::{Database, TableId};
  ///
  /// let table = db.get_table(TableId::new(1));
  pub fn get_table(&self, table_id: TableId) -> Option<&Table> {
    self.tables.get(&table_id)
  }

  /// 根据表名查找表
  ///
  /// # Arguments
  ///
  /// * `name` - 表名
  ///
  /// # Returns
  ///
  /// 如果找到则返回表的引用，否则返回 None
  ///
  /// # Examples
  ///
  /// use rdb_domain::Database;
  /// use std::path::Path;
  ///
  /// let db = Database::new(Path::new("/tmp/test.db"));
  ///
  /// let table = db.get_table_by_name("users");
  ///
  pub fn get_table_by_name(&self, name: &str) -> Option<&Table> {
    self.tables.values().find(|t| t.name == name)
  }

  /// 添加索引（DDL 操作）
  ///
  /// 不变量检查：
  /// - 引用的表必须存在
  /// - 索引在表中必须存在
  ///
  /// # Arguments
  ///
  /// * `index_id` - 索引 ID
  /// * `table_id` - 索引所属的表 ID
  ///
  /// # Returns
  ///
  /// 返回索引的 ID，如果检查失败则返回错误
  ///
  /// # Note
  ///
  /// 此方法在 Index 类型实现后需要完善
  #[allow(dead_code)]
  pub fn add_index(
    &mut self,
    index_id: IndexId,
    table_id: TableId,
  ) -> Result<IndexId, DomainError> {
    // 检查表是否存在
    if !self.tables.contains_key(&table_id) {
      return Err(DomainError::TableNotFound { table_id });
    }

    // TODO: 实现 Index 类型后完善此方法
    // 检查索引列是否在表中存在
    // 添加索引并递增 schema_version
    self.indexes.insert(index_id, ());
    self.schema_version += 1;

    Ok(index_id)
  }

  /// 索取所有表 ID
  ///
  /// # Returns
  ///
  /// 返回所有表 ID 的迭代器
  pub fn table_ids(&self) -> impl Iterator<Item = &TableId> {
    self.tables.keys()
  }

  /// 获取表的数量
  ///
  /// # Returns
  ///
  /// 返回数据库中表的数量
  pub fn table_count(&self) -> usize {
    self.tables.len()
  }
}

// 保证 Database 是 Send + Sync
// 注意：实际使用时需要通过 Arc<Mutex<Database>> 来保证线程安全
unsafe impl Send for Database {}
unsafe impl Sync for Database {}
