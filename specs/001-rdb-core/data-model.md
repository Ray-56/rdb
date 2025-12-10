# 领域模型设计：rdb 核心实体与值对象

**Feature**: rdb 嵌入式关系型数据库  
**Phase**: 1 - Domain Model Design  
**Date**: 2025-12-10

## 概述

本文档定义 rdb 的领域模型，严格遵循 DDD（领域驱动设计）原则。所有类型设计考虑了 Rust 的所有权、生命周期和线程安全要求。

---

## 聚合根（Aggregate Roots）

### 1. Database 聚合根

**职责**：管理数据库的表、索引和全局状态

```rust
/// 数据库聚合根
/// 
/// 不变量：
/// - tables 和 indexes 必须保持一致
/// - 索引必须引用存在的表
/// - schema_version 单调递增
///
/// 生命周期: 'static (拥有所有数据)
/// 线程安全: 需要通过 Arc<Mutex<Database>> 共享
pub struct Database {
    /// 数据库文件路径
    path: PathBuf,
    
    /// 表集合（表 ID -> 表定义）
    tables: HashMap<TableId, Table>,
    
    /// 索引集合（索引 ID -> 索引定义）
    indexes: HashMap<IndexId, Index>,
    
    /// 模式版本号（每次 DDL 操作递增）
    schema_version: u32,
    
    /// 系统表（sqlite_master 等）
    system_tables: SystemTables,
}

impl Database {
    /// 创建或打开数据库
    pub fn open(path: impl AsRef<Path>) -> Result<Self>;
    
    /// 添加表（DDL 操作）
    /// 
    /// 不变量检查：
    /// - 表名唯一
    /// - 至少有一列
    /// - PRIMARY KEY 列存在
    pub fn add_table(&mut self, table: Table) -> Result<TableId>;
    
    /// 删除表（级联删除关联索引）
    pub fn drop_table(&mut self, table_id: TableId) -> Result<()>;
    
    /// 添加索引
    /// 
    /// 不变量检查：
    /// - 引用的表存在
    /// - 索引列在表中存在
    pub fn add_index(&mut self, index: Index) -> Result<IndexId>;
    
    /// 获取表定义（不可变引用）
    pub fn get_table(&self, table_id: TableId) -> Option<&Table>;
}
```

**关系**：
- 聚合 `Table` 实体
- 聚合 `Index` 实体
- 管理 `Transaction` 生命周期

---

### 2. Transaction 聚合根

**职责**：管理事务的 ACID 属性和生命周期

```rust
/// 事务聚合根
/// 
/// 不变量：
/// - 事务必须以 BEGIN 开始，COMMIT 或 ROLLBACK 结束
/// - 同一时刻只有一个活跃的写事务
/// - 读事务可以并发
///
/// 生命周期: 'tx (绑定到事务生命周期)
/// 线程安全: !Send + !Sync (每个事务绑定到一个线程)
pub struct Transaction<'tx> {
    /// 事务 ID（单调递增）
    id: TransactionId,
    
    /// 事务类型（读/写）
    mode: TransactionMode,
    
    /// 隔离级别
    isolation_level: IsolationLevel,
    
    /// WAL 起始位置（用于 ROLLBACK）
    wal_start_offset: u64,
    
    /// 快照版本（MVCC 预留）
    snapshot_version: Option<TransactionId>,
    
    /// 持有的锁
    locks: Vec<LockId>,
    
    /// 生命周期标记（确保事务不能在数据库关闭后使用）
    _phantom: PhantomData<&'tx mut Database>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionMode {
    ReadOnly,
    ReadWrite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    ReadUncommitted,  // v1.0 不支持
    ReadCommitted,    // v1.0 默认
    RepeatableRead,   // v2.0 MVCC
    Serializable,     // v2.0 MVCC
}

impl<'tx> Transaction<'tx> {
    /// 开始事务
    pub fn begin(database: &'tx mut Database, mode: TransactionMode) -> Result<Self>;
    
    /// 提交事务（持久化到 WAL）
    pub fn commit(self) -> Result<()>;
    
    /// 回滚事务（撤销所有更改）
    pub fn rollback(self) -> Result<()>;
    
    /// 执行 SQL 语句（在事务上下文中）
    pub fn execute(&mut self, sql: &str) -> Result<usize>;
}
```

**关系**：
- 依赖 `Database` 聚合根
- 管理 `Row` 的创建/更新/删除

---

## 实体（Entities）

### 3. Table 实体

**职责**：定义表结构和元数据

```rust
/// 表实体
/// 
/// 不变量：
/// - name 非空且唯一（在 Database 范围内）
/// - columns 非空
/// - primary_key 必须引用存在的列
/// - root_page 必须有效
///
/// 生命周期: 'static
/// 线程安全: Send + Sync（不可变共享）
#[derive(Debug, Clone)]
pub struct Table {
    /// 表 ID（唯一标识）
    id: TableId,
    
    /// 表名
    name: String,
    
    /// 列定义
    columns: Vec<Column>,
    
    /// 主键列 ID（如果有）
    primary_key: Option<ColumnId>,
    
    /// B+Tree 根页 ID
    root_page: PageId,
    
    /// 表统计信息（行数、页数等）
    stats: TableStats,
}

#[derive(Debug, Clone, Copy)]
pub struct TableStats {
    /// 估计行数
    pub estimated_rows: u64,
    
    /// 占用页数
    pub page_count: u32,
    
    /// 平均行大小（字节）
    pub avg_row_size: u32,
}

impl Table {
    /// 创建新表
    pub fn new(name: String, columns: Vec<Column>) -> Result<Self>;
    
    /// 查找列（按名称）
    pub fn get_column(&self, name: &str) -> Option<&Column>;
    
    /// 获取主键列
    pub fn primary_key_column(&self) -> Option<&Column>;
    
    /// 验证行数据是否符合表定义
    pub fn validate_row(&self, row: &Row) -> Result<()>;
}
```

---

### 4. Index 实体

**职责**：定义索引结构

```rust
/// 索引实体
/// 
/// 不变量：
/// - name 唯一（在 Table 范围内）
/// - table_id 引用存在的表
/// - columns 非空且引用存在的列
///
/// 生命周期: 'static
/// 线程安全: Send + Sync
#[derive(Debug, Clone)]
pub struct Index {
    /// 索引 ID
    id: IndexId,
    
    /// 索引名
    name: String,
    
    /// 所属表 ID
    table_id: TableId,
    
    /// 索引列（支持复合索引）
    columns: Vec<ColumnId>,
    
    /// 索引类型
    index_type: IndexType,
    
    /// B+Tree 根页 ID
    root_page: PageId,
    
    /// 是否唯一索引
    is_unique: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexType {
    BTree,       // v1.0: B+Tree 索引
    Hash,        // v2.0: 哈希索引（仅等值查询）
    FullText,    // v3.0: 全文索引
}

impl Index {
    /// 创建新索引
    pub fn new(
        name: String,
        table_id: TableId,
        columns: Vec<ColumnId>,
        is_unique: bool,
    ) -> Self;
    
    /// 从行提取索引键
    pub fn extract_key(&self, row: &Row) -> Result<IndexKey>;
}
```

---

### 5. Row 实体

**职责**：表示一行数据

```rust
/// 行实体
/// 
/// 不变量：
/// - values 长度必须等于表的列数
/// - 主键列不能为 NULL（如果有）
///
/// 生命周期: 'r (可能引用外部数据)
/// 线程安全: Send + Sync
#[derive(Debug, Clone)]
pub struct Row<'r> {
    /// 行 ID（等同于 INTEGER PRIMARY KEY）
    row_id: RowId,
    
    /// 列值
    values: Vec<Value<'r>>,
    
    /// MVCC 字段（预留）
    created_txn_id: Option<TransactionId>,
    deleted_txn_id: Option<TransactionId>,
}

impl<'r> Row<'r> {
    /// 创建新行
    pub fn new(row_id: RowId, values: Vec<Value<'r>>) -> Self;
    
    /// 获取列值（按索引）
    pub fn get(&self, index: usize) -> Option<&Value<'r>>;
    
    /// 获取列值（按列名）
    pub fn get_by_name(&self, column_name: &str, table: &Table) -> Option<&Value<'r>>;
    
    /// 序列化为字节（存储到页）
    pub fn serialize(&self) -> Vec<u8>;
    
    /// 从字节反序列化
    pub fn deserialize(data: &'r [u8]) -> Result<Self>;
}
```

---

### 6. Column 实体

**职责**：定义列的属性

```rust
/// 列实体
/// 
/// 不变量：
/// - name 非空
/// - default_value 类型必须与 data_type 匹配
///
/// 生命周期: 'static
/// 线程安全: Send + Sync
#[derive(Debug, Clone)]
pub struct Column {
    /// 列 ID（表内唯一）
    id: ColumnId,
    
    /// 列名
    name: String,
    
    /// 数据类型
    data_type: DataType,
    
    /// 约束
    constraints: ColumnConstraints,
    
    /// 默认值
    default_value: Option<Value<'static>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnConstraints {
    /// NOT NULL 约束
    pub not_null: bool,
    
    /// UNIQUE 约束
    pub unique: bool,
    
    /// PRIMARY KEY 约束
    pub primary_key: bool,
    
    /// AUTOINCREMENT（仅 INTEGER PRIMARY KEY）
    pub autoincrement: bool,
}

impl Column {
    /// 创建新列
    pub fn new(name: String, data_type: DataType) -> Self;
    
    /// 验证值是否符合列定义
    pub fn validate_value(&self, value: &Value) -> Result<()>;
}
```

---

## 值对象（Value Objects）

### 7. Value 值对象

**职责**：表示数据库中的值（类型安全的数据）

```rust
/// 值对象：数据库值
/// 
/// 不变量：
/// - Text 必须是有效的 UTF-8
/// - Real 不能是 NaN 或 Infinity（可选限制）
///
/// 生命周期: 'v (可能引用外部数据，避免拷贝)
/// 线程安全: Send + Sync
#[derive(Debug, Clone, PartialEq)]
pub enum Value<'v> {
    /// NULL 值
    Null,
    
    /// 64-bit 整数
    Integer(i64),
    
    /// 64-bit 浮点数
    Real(f64),
    
    /// UTF-8 文本（使用 Cow 避免拷贝）
    Text(Cow<'v, str>),
    
    /// 二进制数据（使用 Cow 避免拷贝）
    Blob(Cow<'v, [u8]>),
}

impl<'v> Value<'v> {
    /// 转换为拥有所有权的值
    pub fn into_owned(self) -> Value<'static>;
    
    /// 获取类型
    pub fn data_type(&self) -> DataType;
    
    /// 尝试转换为 i64
    pub fn as_integer(&self) -> Option<i64>;
    
    /// 尝试转换为 f64
    pub fn as_real(&self) -> Option<f64>;
    
    /// 尝试转换为 &str
    pub fn as_text(&self) -> Option<&str>;
    
    /// 尝试转换为 &[u8]
    pub fn as_blob(&self) -> Option<&[u8]>;
    
    /// 比较（SQL 语义：NULL != NULL）
    pub fn sql_compare(&self, other: &Self) -> Option<Ordering>;
}
```

---

### 8. DataType 值对象

**职责**：定义列的数据类型

```rust
/// 数据类型值对象
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
    pub fn from_sql_type(sql_type: &str) -> Option<Self>;
    
    /// 转换为 SQL 类型名
    pub fn to_sql_type(&self) -> &'static str;
    
    /// 检查值是否匹配此类型
    pub fn matches(&self, value: &Value) -> bool;
}
```

---

### 9. 标识符类型（ID Types）

**职责**：类型安全的 ID（防止混淆）

```rust
/// 表 ID（newtype 模式）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TableId(u32);

/// 列 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColumnId(u32);

/// 索引 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IndexId(u32);

/// 行 ID（等同于 INTEGER PRIMARY KEY）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RowId(i64);

/// 事务 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TransactionId(u64);

/// 页 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageId(u32);

/// 锁 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LockId(u64);
```

---

## 领域服务（Domain Services）

### 10. TransactionManager 服务

**职责**：管理事务的创建和生命周期

```rust
/// 事务管理器（领域服务）
/// 
/// 职责：
/// - 分配事务 ID
/// - 管理事务锁
/// - 维护活跃事务列表
///
/// 线程安全: Send + Sync（内部使用 Mutex）
pub struct TransactionManager {
    /// 下一个事务 ID
    next_txn_id: AtomicU64,
    
    /// 活跃事务列表
    active_transactions: Mutex<HashSet<TransactionId>>,
    
    /// 锁管理器
    lock_manager: Arc<LockManager>,
}

impl TransactionManager {
    /// 开始新事务
    pub fn begin_transaction(
        &self,
        mode: TransactionMode,
        isolation_level: IsolationLevel,
    ) -> Result<TransactionId>;
    
    /// 提交事务
    pub fn commit_transaction(&self, txn_id: TransactionId) -> Result<()>;
    
    /// 回滚事务
    pub fn rollback_transaction(&self, txn_id: TransactionId) -> Result<()>;
    
    /// 获取活跃事务列表（MVCC 使用）
    pub fn active_transactions(&self) -> Vec<TransactionId>;
}
```

---

### 11. SchemaValidator 服务

**职责**：验证模式定义的正确性

```rust
/// 模式验证器（领域服务）
/// 
/// 职责：
/// - 验证表定义合法性
/// - 验证索引引用正确性
/// - 验证约束一致性
pub struct SchemaValidator;

impl SchemaValidator {
    /// 验证表定义
    pub fn validate_table(&self, table: &Table) -> Result<()>;
    
    /// 验证索引定义
    pub fn validate_index(&self, index: &Index, database: &Database) -> Result<()>;
    
    /// 验证外键约束（v1.0 基础支持）
    pub fn validate_foreign_key(&self, fk: &ForeignKey, database: &Database) -> Result<()>;
}
```

---

## 领域事件（Domain Events）

### 12. 事件类型

**职责**：记录领域中发生的重要变化

```rust
/// 领域事件
/// 
/// 用途：
/// - 审计日志
/// - 触发器实现
/// - 复制（v2.0）
#[derive(Debug, Clone)]
pub enum DomainEvent {
    /// 表已创建
    TableCreated {
        table_id: TableId,
        table_name: String,
        timestamp: SystemTime,
    },
    
    /// 表已删除
    TableDropped {
        table_id: TableId,
        timestamp: SystemTime,
    },
    
    /// 索引已创建
    IndexCreated {
        index_id: IndexId,
        table_id: TableId,
        timestamp: SystemTime,
    },
    
    /// 事务已提交
    TransactionCommitted {
        txn_id: TransactionId,
        affected_rows: usize,
        timestamp: SystemTime,
    },
    
    /// 事务已回滚
    TransactionRolledBack {
        txn_id: TransactionId,
        timestamp: SystemTime,
    },
}
```

---

## 不变量总结

### 全局不变量

1. **唯一性**：
   - 表名在数据库中唯一
   - 列名在表中唯一
   - 索引名在表中唯一
   - 行 ID 在表中唯一

2. **引用完整性**：
   - 索引必须引用存在的表和列
   - 主键列必须在表定义中存在
   - 外键必须引用存在的表和列（v1.0 基础支持）

3. **类型一致性**：
   - 列值类型必须匹配列定义
   - 默认值类型必须匹配列类型
   - 索引键类型必须匹配列类型

4. **事务一致性**：
   - 同一时刻只有一个写事务
   - 事务必须以 BEGIN 开始，以 COMMIT/ROLLBACK 结束
   - 未提交的事务对其他事务不可见

### 模块级不变量

#### Database 聚合根
- `tables` 和 `indexes` 映射必须一致
- `schema_version` 单调递增
- 系统表（sqlite_master）不能被删除

#### Transaction 聚合根
- 事务 ID 单调递增
- 写事务持有写锁直到提交/回滚
- WAL 偏移量有效

#### Table 实体
- `columns` 非空
- `primary_key` 引用的列存在
- `root_page` 指向有效的 B+Tree 根节点

#### B+Tree 不变量（存储层）
- 所有键有序
- 内部节点有 [n/2, n] 个子节点（根节点除外）
- 叶子节点在同一层

---

## 领域模型关系图

```
┌──────────────────────────────────────────────────┐
│                   Database                       │
│  (聚合根)                                        │
│  ┌────────────┐  ┌────────────┐                 │
│  │  Table     │  │   Index    │                 │
│  │  (实体)    │  │   (实体)   │                 │
│  │            │  │            │                 │
│  │ ┌────────┐ │  │            │                 │
│  │ │ Column │ │  │            │                 │
│  │ │ (实体) │ │  │            │                 │
│  │ └────────┘ │  │            │                 │
│  └────────────┘  └────────────┘                 │
└──────────────────────────────────────────────────┘
              │
              │ manages
              ▼
┌──────────────────────────────────────────────────┐
│                 Transaction                      │
│  (聚合根)                                        │
│  ┌────────────┐                                  │
│  │    Row     │                                  │
│  │  (实体)    │                                  │
│  │            │                                  │
│  │ ┌────────┐ │                                  │
│  │ │ Value  │ │                                  │
│  │ │(值对象)│ │                                  │
│  │ └────────┘ │                                  │
│  └────────────┘                                  │
└──────────────────────────────────────────────────┘
```

---

**Data Model Version**: 1.0  
**Prepared By**: Domain Modeling Team  
**Date**: 2025-12-10

