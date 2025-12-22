//! ID 类型定义（newtype 模式）
//!
//! 使用 newtype 模式提供类型安全的 ID，防止不同类型的 ID 混淆。

/// 表 ID（newtype 模式）
///
/// 用于唯一标识数据库中的表。
/// 底层类型：`u32`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TableId(u32);

impl TableId {
  #[inline]
  pub fn new(id: u32) -> Self {
    Self(id)
  }

  #[inline]
  pub fn into_inner(self) -> u32 {
    self.0
  }
}

impl From<u32> for TableId {
  fn from(id: u32) -> Self {
    Self::new(id)
  }
}

impl From<TableId> for u32 {
  #[inline]
  fn from(id: TableId) -> Self {
    id.into_inner()
  }
}

/// 列 ID
///
/// 用于唯一标识表中的列
/// 底层类型：`u32`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColumnId(u32);

impl ColumnId {
  #[inline]
  pub fn new(id: u32) -> Self {
    Self(id)
  }

  #[inline]
  pub fn into_inner(self) -> u32 {
    self.0
  }
}

impl From<u32> for ColumnId {
  #[inline]
  fn from(id: u32) -> Self {
    Self::new(id)
  }
}

impl From<ColumnId> for u32 {
  #[inline]
  fn from(id: ColumnId) -> Self {
    id.0
  }
}

/// 索引 ID
///
/// 用于唯一标识数据库中的索引
/// 底层类型：`u32`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IndexId(u32);

impl IndexId {
  #[inline]
  pub fn new(id: u32) -> Self {
    Self(id)
  }

  #[inline]
  pub fn into_inner(self) -> u32 {
    self.0
  }
}

impl From<u32> for IndexId {
  #[inline]
  fn from(id: u32) -> Self {
    Self(id)
  }
}

impl From<IndexId> for u32 {
  #[inline]
  fn from(id: IndexId) -> Self {
    id.0
  }
}

/// 行 ID (等同于 INTEGER PRIMARY KEY)
///
/// 用于唯一标识表中的行
/// 底层类型：`i64` (支持负数，SQLite 兼容)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RowId(i64);

impl RowId {
  #[inline]
  pub fn new(id: i64) -> Self {
    Self(id)
  }

  #[inline]
  pub fn into_inner(self) -> i64 {
    self.0
  }
}

impl From<i64> for RowId {
  #[inline]
  fn from(id: i64) -> Self {
    Self(id)
  }
}

impl From<RowId> for i64 {
  #[inline]
  fn from(id: RowId) -> Self {
    id.0
  }
}

/// 事务 ID
///
/// 用于唯一标识事务
/// 底层类型：`u64` (单调递增)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TransactionId(u64);

impl TransactionId {
  #[inline]
  pub fn new(id: u64) -> Self {
    Self(id)
  }

  #[inline]
  pub fn into_inner(self) -> u64 {
    self.0
  }
}

impl From<u64> for TransactionId {
  #[inline]
  fn from(id: u64) -> Self {
    Self(id)
  }
}

impl From<TransactionId> for u64 {
  #[inline]
  fn from(id: TransactionId) -> Self {
    id.0
  }
}

/// 页 ID
///
/// 用于唯一标识储存页
/// 底层类型：`u32`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageId(u32);

impl PageId {
  #[inline]
  pub fn new(id: u32) -> Self {
    Self(id)
  }

  #[inline]
  pub fn into_inner(self) -> u32 {
    self.0
  }
}

impl From<u32> for PageId {
  #[inline]
  fn from(id: u32) -> Self {
    Self(id)
  }
}

impl From<PageId> for u32 {
  #[inline]
  fn from(id: PageId) -> Self {
    id.0
  }
}

/// 锁 ID
///
/// 用于唯一标识锁
/// 底层类型：`u64`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LockId(u64);

impl LockId {
  #[inline]
  pub fn new(id: u64) -> Self {
    Self(id)
  }

  #[inline]
  pub fn into_inner(self) -> u64 {
    self.0
  }
}

impl From<u64> for LockId {
  #[inline]
  fn from(id: u64) -> Self {
    Self(id)
  }
}

impl From<LockId> for u64 {
  #[inline]
  fn from(id: LockId) -> Self {
    id.0
  }
}
