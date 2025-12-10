# 特性规格说明：rdb 嵌入式关系型数据库核心实现

**特性分支**: `001-rdb-core`  
**创建日期**: 2025-12-10  
**状态**: Draft  
**输入**: 完整的 rdb 嵌入式关系型数据库实现（52周路线图、DDD架构、存储引擎、B+Tree、WAL等完整设计）

## 项目概述

rdb 是一个使用纯 Rust 实现的嵌入式关系型数据库，目标是提供类似 SQLite 的功能，但采用严格的 DDD（领域驱动设计）架构。项目完全遵循 rdb Constitution 中定义的 7 条核心原则，确保内存安全、可扩展性和工业级质量。

### 设计目标

- **纯 Rust 实现**：零外部数据库内核依赖，所有组件用 Rust 编写
- **DDD 分层架构**：清晰的领域层、应用层、基础设施层和接口层分离
- **MVCC 就绪**：从第一天起就为多版本并发控制预留接口
- **集群就绪**：存储格式设计为 100% 兼容未来分布式版本
- **工业级质量**：所有设计基于 SQLite/CockroachDB/TiDB 验证过的方案

## 用户场景与测试 *(必填)*

### User Story 1 - 基础 SQL 查询能力 (Priority: P1) 🎯 MVP

作为应用程序开发者，我需要能够创建数据库文件、创建表、插入数据、执行简单的 SELECT 查询，以便将 rdb 集成到我的 Rust 应用程序中作为嵌入式数据库。

**为什么是 P1 优先级**：这是数据库最基本的功能，是所有后续功能的基础。没有这个，rdb 无法提供任何价值。

**独立测试**：可以通过创建一个简单的 Rust 程序，执行 `CREATE TABLE`、`INSERT`、`SELECT` 操作，并验证数据能正确持久化和读取来完全测试此功能。

**验收场景**：

1. **Given** 应用程序启动，**When** 调用 `Database::open("test.db")` 创建数据库，**Then** 成功创建数据库文件且返回 Database 句柄
2. **Given** 数据库已打开，**When** 执行 `CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)` ，**Then** 成功创建表结构
3. **Given** 表已创建，**When** 执行 `INSERT INTO users VALUES (1, 'Alice')` ，**Then** 数据成功写入
4. **Given** 数据已插入，**When** 执行 `SELECT * FROM users WHERE id = 1` ，**Then** 返回正确的行 `(1, 'Alice')`
5. **Given** 数据已写入，**When** 关闭数据库并重新打开，**Then** 之前插入的数据依然存在（持久化测试）

---

### User Story 2 - 事务支持（BEGIN/COMMIT/ROLLBACK） (Priority: P2)

作为应用程序开发者，我需要事务支持来确保多个操作的原子性，避免数据不一致。

**为什么是 P2 优先级**：事务是关系型数据库的核心特性，对于生产级应用必不可少。但在 MVP 阶段可以先实现基础查询。

**独立测试**：创建事务，执行多个写操作，验证 COMMIT 后数据持久化，验证 ROLLBACK 后数据回滚。

**验收场景**：

1. **Given** 数据库已打开，**When** 执行 `BEGIN TRANSACTION` ，**Then** 进入事务模式
2. **Given** 事务已开始，**When** 执行多个 INSERT/UPDATE ，**Then** 更改在事务内可见但未提交到磁盘
3. **Given** 事务中有多个更改，**When** 执行 `COMMIT` ，**Then** 所有更改原子性地持久化
4. **Given** 事务中有多个更改，**When** 执行 `ROLLBACK` ，**Then** 所有更改被撤销，数据库恢复到事务开始前的状态
5. **Given** 事务进行中，**When** 程序崩溃，**Then** 重启后数据库恢复到最后一次成功 COMMIT 的状态（WAL 恢复测试）

---

### User Story 3 - 索引支持（CREATE INDEX） (Priority: P3)

作为应用程序开发者，我需要在常用查询列上创建索引，以提升查询性能。

**为什么是 P3 优先级**：索引是性能优化特性，在基础功能和事务支持完成后添加。

**独立测试**：创建表，插入大量数据，创建索引前后对比 SELECT 查询性能。

**验收场景**：

1. **Given** 表已创建且包含数据，**When** 执行 `CREATE INDEX idx_name ON users(name)` ，**Then** 成功创建索引
2. **Given** 索引已创建，**When** 执行 `SELECT * FROM users WHERE name = 'Alice'` ，**Then** 使用索引查询（通过 EXPLAIN 验证）
3. **Given** 索引已创建，**When** 执行 INSERT/UPDATE/DELETE ，**Then** 索引自动更新保持一致

---

### User Story 4 - 并发读取支持 (Priority: P4)

作为应用程序开发者，我需要多个线程能同时读取数据库，以支持高并发场景。

**为什么是 P4 优先级**：并发是生产环境的重要特性，但可以在核心功能稳定后添加。

**独立测试**：启动多个线程同时执行 SELECT 查询，验证无数据竞争且结果正确。

**验收场景**：

1. **Given** 数据库已打开，**When** 多个线程同时执行读操作，**Then** 所有读取都能成功完成且返回一致的数据
2. **Given** 一个线程在写入，**When** 其他线程尝试读取，**Then** 读取线程能看到一致的快照（MVCC 读取隔离）

---

### User Story 5 - JOIN 查询支持 (Priority: P5)

作为应用程序开发者，我需要执行多表 JOIN 查询，以处理关联数据。

**为什么是 P5 优先级**：JOIN 是复杂查询特性，在基础查询、事务、索引完成后实现。

**独立测试**：创建两个有外键关系的表，执行 INNER JOIN 和 LEFT JOIN，验证结果正确。

**验收场景**：

1. **Given** 两个表 `users` 和 `orders` 已创建，**When** 执行 `SELECT * FROM users JOIN orders ON users.id = orders.user_id` ，**Then** 返回正确的关联结果
2. **Given** 多个表关联，**When** 执行复杂的多表 JOIN ，**Then** 查询优化器选择合理的执行计划

---

### 边界情况

- **空数据库**：打开不存在的数据库文件时如何处理？（自动创建）
- **磁盘满**：写入时磁盘空间不足如何处理？（返回错误，保证数据一致性）
- **并发写入冲突**：多个事务同时修改同一行如何处理？（通过锁或 MVCC 机制解决）
- **大事务**：事务中包含数百万行写入时如何处理？（WAL 增长，内存压力）
- **损坏的数据库文件**：如何检测和处理？（checksum 验证，返回错误）
- **SQL 注入**：如何防范？（使用参数化查询接口）
- **超大查询结果**：SELECT 返回数百万行时如何处理？（流式迭代器）

## 需求 *(必填)*

### 功能需求

#### 核心存储引擎

- **FR-001**: 系统必须实现 4KB 页式存储（Pager），支持页的读取、写入、缓存
- **FR-002**: 系统必须实现 B+Tree 索引结构，支持高效的键值查找和范围扫描
- **FR-003**: 系统必须实现 Write-Ahead Log (WAL) 用于事务持久性和崩溃恢复
- **FR-004**: 系统必须支持 Checkpoint 机制，将 WAL 数据同步到主数据库文件
- **FR-005**: 系统必须实现 Freelist 管理，复用已删除的页空间

#### SQL 支持

- **FR-006**: 系统必须支持 SQL 解析（使用 sqlparser-rs），将 SQL 语句解析为 AST
- **FR-007**: 系统必须支持基础 DDL 操作：CREATE TABLE, DROP TABLE, CREATE INDEX, DROP INDEX
- **FR-008**: 系统必须支持基础 DML 操作：INSERT, UPDATE, DELETE, SELECT
- **FR-009**: 系统必须支持 WHERE 子句过滤（=, <, >, <=, >=, !=, AND, OR, NOT）
- **FR-010**: 系统必须支持聚合函数：COUNT, SUM, AVG, MIN, MAX
- **FR-011**: 系统必须支持 ORDER BY 和 LIMIT 子句
- **FR-012**: 系统必须支持 JOIN 操作（INNER JOIN, LEFT JOIN, RIGHT JOIN）

#### 数据类型

- **FR-013**: 系统必须支持以下基础数据类型：INTEGER, REAL, TEXT, BLOB, NULL
- **FR-014**: 系统必须支持 PRIMARY KEY 约束
- **FR-015**: 系统必须支持 NOT NULL, UNIQUE, DEFAULT 约束
- **FR-016**: 系统必须支持自动递增 INTEGER PRIMARY KEY

#### 事务与并发

- **FR-017**: 系统必须支持事务操作：BEGIN, COMMIT, ROLLBACK
- **FR-018**: 系统必须保证事务的 ACID 属性
- **FR-019**: 系统必须支持并发读取（多个线程同时读）
- **FR-020**: 系统必须预留 MVCC 接口，支持快照读（未来实现）
- **FR-021**: 系统必须实现写锁机制，同一时刻只允许一个写事务

#### 持久性与恢复

- **FR-022**: 系统必须在每次 COMMIT 时确保数据持久化到 WAL
- **FR-023**: 系统必须在程序崩溃后能通过 WAL 恢复到一致状态
- **FR-024**: 系统必须支持数据库文件完整性校验（checksum）

#### API 接口

- **FR-025**: 系统必须提供 Rust API 供应用程序调用
- **FR-026**: 系统必须提供 SQL 字符串执行接口（execute 方法）
- **FR-027**: 系统必须提供参数化查询接口（防止 SQL 注入）
- **FR-028**: 系统必须返回结构化查询结果（迭代器模式）

### 核心实体

#### 领域层实体

- **Database**: 数据库实例，代表一个完整的数据库文件，管理表、索引、事务
- **Table**: 表定义，包含列信息、主键、约束
- **Index**: 索引定义，关联表和索引列
- **Transaction**: 事务实体，管理事务的生命周期和隔离级别
- **Row**: 行数据，包含实际的列值
- **Column**: 列定义，包含列名、数据类型、约束

#### 基础设施层实体

- **Pager**: 页管理器，负责页的读取、写入、缓存
- **Page**: 4KB 数据页，存储实际数据或 B+Tree 节点
- **BTree**: B+Tree 实现，提供索引和表数据存储
- **WAL**: Write-Ahead Log，事务日志
- **BufferPool**: 页缓存池，LRU 缓存策略

#### 应用层实体

- **QueryExecutor**: 查询执行器，执行 SQL 查询计划
- **QueryPlanner**: 查询计划器，优化 SQL 查询
- **SqlParser**: SQL 解析器（包装 sqlparser-rs）

## 成功标准 *(必填)*

### 可衡量的成果

#### 功能完整性

- **SC-001**: 系统能够成功执行所有基础 SQL 操作（CREATE TABLE, INSERT, SELECT, UPDATE, DELETE），成功率 100%
- **SC-002**: 系统能够在程序崩溃后通过 WAL 恢复数据，数据丢失率为 0（已提交事务）
- **SC-003**: 系统能够支持至少 10 个并发读取线程，无数据竞争
- **SC-004**: 系统通过所有 proptest 属性测试，验证 B+Tree 不变量和事务原子性

#### 性能指标

- **SC-005**: 单表顺序插入性能达到至少 10,000 行/秒（无索引情况）
- **SC-006**: 通过主键查询单行的响应时间 < 1ms（10万行数据规模）
- **SC-007**: 通过索引查询的性能比全表扫描提升至少 10 倍（10万行数据规模）
- **SC-008**: Checkpoint 操作完成时间 < 100ms（1MB WAL 文件）

#### 内存与资源

- **SC-009**: 数据库启动内存占用 < 5MB（不包括缓存）
- **SC-010**: BufferPool 能够有效缓存热数据，缓存命中率 > 80%（正常工作负载）

#### 代码质量

- **SC-011**: 所有公共 API 都有完整的 Rustdoc 文档
- **SC-012**: 代码通过 `cargo clippy -- -D warnings` 检查，无警告
- **SC-013**: 所有 unsafe 代码块都有详细的安全注释，且限制在 Pager 和 B+Tree 模块
- **SC-014**: 测试覆盖率达到 80% 以上（核心模块）

#### 兼容性与扩展性

- **SC-015**: 存储格式包含版本号和特性标志，支持未来版本升级
- **SC-016**: API 设计支持未来添加 MVCC 快照读，无需破坏性更改
- **SC-017**: 所有核心模块使用 `pub(crate)` 可见性，公共 API 最小化

## 架构设计

### DDD 分层架构

```
rdb/
├── Cargo.toml (workspace root)
├── rdb-domain/         # 领域层：核心业务逻辑
├── rdb-application/    # 应用层：用例编排
├── rdb-infrastructure/ # 基础设施层：存储、IO
├── rdb-interface/      # 接口层：公共 API
├── rdb-sql/            # SQL 解析与执行
└── rdb-storage/        # 存储引擎核心
```

### Cargo Workspace 结构

#### `rdb-domain` - 领域层

**职责**：定义核心业务概念和不变量，不依赖任何外部库（仅 std）

- **聚合根**：`Database`、`Table`、`Index`、`Transaction`
- **实体**：`Row`、`Column`、`Constraint`
- **值对象**：`DataType`、`Value`、`TableId`、`ColumnId`
- **领域服务**：`TransactionManager`、`SchemaValidator`

**关键特性**：
- 零外部依赖（仅 std）
- 所有类型都是 `Send + Sync`（支持并发）
- 丰富的类型系统，编译期保证不变量

#### `rdb-storage` - 存储引擎核心

**职责**：实现 Pager、B+Tree、WAL 等底层存储原语

- **模块**：`pager`、`btree`、`wal`、`freelist`、`page`
- **关键类型**：`Pager<'db>`、`BTree`、`WalWriter`、`Page<'page>`
- **unsafe 使用场景**：页内指针操作、内存映射

**关键特性**：
- 这是唯一允许使用 `unsafe` 的 crate（严格限制）
- 生命周期参数确保内存安全
- 所有 unsafe 块都有详细的安全注释

#### `rdb-infrastructure` - 基础设施层

**职责**：提供缓存、IO、并发控制等基础设施

- **模块**：`buffer_pool`、`lock_manager`、`file_io`
- **关键类型**：`BufferPool`、`LockManager`、`FileHandle`

**关键特性**：
- 使用 `parking_lot` 提供高性能锁
- LRU 缓存实现
- 线程安全保证

#### `rdb-sql` - SQL 层

**职责**：SQL 解析、查询计划、查询优化、执行

- **模块**：`parser`、`planner`、`optimizer`、`executor`
- **关键类型**：`SqlParser`、`QueryPlan`、`Executor`
- **依赖**：使用 `sqlparser-rs` 进行 SQL 解析

**关键特性**：
- 封装 sqlparser-rs，提供类型安全的 AST
- 查询计划器支持索引选择
- 执行器使用迭代器模式

#### `rdb-application` - 应用层

**职责**：编排用例，连接领域层和基础设施层

- **用例**：`CreateTable`、`ExecuteQuery`、`BeginTransaction`、`Commit`
- **关键类型**：`DatabaseService`、`TransactionService`

#### `rdb-interface` - 接口层

**职责**：提供公共 API，暴露给应用程序使用

- **公共 API**：`Database::open()`、`Connection::execute()`、`Statement::query()`
- **结果类型**：`Result<T, RdbError>`、`Rows`（迭代器）

**关键特性**：
- 这是唯一的公共 crate（其他都是 `pub(crate)`）
- 极简 API 设计
- 线程安全保证（`Send + Sync`）

### DDD 核心类型（Rust 伪代码）

#### 领域层核心聚合根

```rust
// rdb-domain/src/database.rs
/// 数据库聚合根
/// 生命周期: 'static (拥有所有数据)
/// 线程安全: !Send + !Sync (需要通过 Arc 共享)
pub struct Database {
    path: PathBuf,
    tables: HashMap<TableId, Table>,
    indexes: HashMap<IndexId, Index>,
    schema_version: u32,
}

impl Database {
    /// 不变量: tables 和 indexes 必须保持一致
    pub fn add_table(&mut self, table: Table) -> Result<TableId>;
    pub fn add_index(&mut self, index: Index) -> Result<IndexId>;
}

// rdb-domain/src/table.rs
/// 表实体
/// 生命周期: 'static
/// 线程安全: Send + Sync
#[derive(Clone)]
pub struct Table {
    id: TableId,
    name: String,
    columns: Vec<Column>,
    primary_key: Option<ColumnId>,
    root_page: PageId,
}

// rdb-domain/src/transaction.rs
/// 事务聚合根
/// 生命周期: 'tx (绑定到事务生命周期)
/// 线程安全: !Send + !Sync (每个事务绑定到一个线程)
pub struct Transaction<'tx> {
    id: TransactionId,
    database: &'tx Database,
    isolation_level: IsolationLevel,
    wal_position: u64,
    _phantom: PhantomData<&'tx mut ()>, // 确保独占访问
}

// rdb-domain/src/value.rs
/// 值对象：数据库值
/// 生命周期: 'v (可能引用外部数据)
/// 线程安全: Send + Sync
pub enum Value<'v> {
    Null,
    Integer(i64),
    Real(f64),
    Text(Cow<'v, str>),
    Blob(Cow<'v, [u8]>),
}
```

#### 存储层核心类型

```rust
// rdb-storage/src/pager.rs
/// 页管理器
/// 生命周期: 'db (绑定到数据库文件)
/// 线程安全: !Send + !Sync (内部使用 RefCell)
pub struct Pager<'db> {
    file: File,
    page_size: usize,
    cache: RefCell<HashMap<PageId, Page<'db>>>, // interior mutability
    _phantom: PhantomData<&'db mut ()>,
}

impl<'db> Pager<'db> {
    /// UNSAFE: 需要确保 page_id 有效且未被并发访问
    pub unsafe fn get_page_ptr(&self, page_id: PageId) -> *const Page<'db>;
    
    /// UNSAFE: 需要确保独占访问
    pub unsafe fn get_page_mut_ptr(&mut self, page_id: PageId) -> *mut Page<'db>;
}

// rdb-storage/src/page.rs
/// 4KB 数据页
/// 生命周期: 'page (绑定到 Pager)
/// 线程安全: !Send + !Sync (包含原始指针)
#[repr(C, align(4096))]
pub struct Page<'page> {
    data: [u8; 4096],
    page_id: PageId,
    dirty: bool,
    pin_count: AtomicU32,
    _phantom: PhantomData<&'page mut ()>,
}

// rdb-storage/src/btree.rs
/// B+Tree 节点
/// 生命周期: 'tree (绑定到 BTree)
/// 线程安全: !Send + !Sync (包含页指针)
pub struct BTreeNode<'tree, K, V> {
    page: Pin<&'tree mut Page<'tree>>, // Pin 防止移动
    is_leaf: bool,
    num_keys: u16,
    keys: &'tree [K],   // UNSAFE: 指向 page.data 内部
    values: &'tree [V], // UNSAFE: 指向 page.data 内部
}

impl<'tree, K: Ord, V> BTreeNode<'tree, K, V> {
    /// UNSAFE: 需要确保 keys/values 切片不越界
    unsafe fn from_page(page: Pin<&'tree mut Page<'tree>>) -> Self;
}
```

#### SQL 执行层类型

```rust
// rdb-sql/src/executor.rs
/// 查询执行器（迭代器模式）
/// 生命周期: 'exec (绑定到查询生命周期)
/// 线程安全: Send (可以跨线程转移，但不能并发访问)
pub struct Executor<'exec> {
    plan: QueryPlan,
    transaction: &'exec Transaction<'exec>,
}

impl<'exec> Iterator for Executor<'exec> {
    type Item = Result<Row>;
    
    fn next(&mut self) -> Option<Self::Item> {
        // 流式返回结果，避免内存爆炸
    }
}
```

### 存储层字节级设计

#### 4KB Page 头部格式

```
Offset | Size | Field              | Description
-------|------|-------------------|----------------------------------
0x0000 | 1    | page_type         | 0x05=内部节点, 0x0D=叶子节点, 0x02=溢出页
0x0001 | 2    | first_freeblock   | 第一个空闲块偏移（0=无空闲块）
0x0003 | 2    | num_cells         | 页内 cell 数量
0x0005 | 2    | cell_content_area | cell 内容区域起始偏移
0x0007 | 1    | fragmented_bytes  | 碎片字节数
0x0008 | 4    | right_child       | 仅内部节点：最右子页 ID
0x000C | 8    | lsn               | MVCC: 日志序列号（预留）
0x0014 | 4    | checksum          | CRC32 校验和
0x0018 | 8    | reserved          | 预留用于集群元数据
-------|------|-------------------|----------------------------------
0x0020 | var  | cell_ptr_array    | cell 指针数组（每个 2 字节）
...    | ...  | ...               | ...
...    | ...  | unallocated       | 未分配空间
...    | ...  | cell_content      | cell 内容区域（从页尾向上增长）
```

**ASCII 图示**:

```
+-------------------------------------------+
| Page Header (32 bytes)                    |
| - page_type, num_cells, checksum, lsn...  |
+-------------------------------------------+
| Cell Pointer Array                        |
| [ptr1][ptr2][ptr3]...                     |
+-------------------------------------------+
|                                           |
|    Unallocated Space (grows down)         |
|                                           |
+-------------------------------------------+
| Cell Content Area (grows up from bottom)  |
| [...cell N...]                            |
| [...cell 2...]                            |
| [...cell 1...]                            |
+-------------------------------------------+
```

#### B+Tree 内部节点布局

```
Cell Pointer Array 中每个指针指向一个 cell:

Internal Node Cell:
Offset | Size | Field
-------|------|------------------
0x00   | 4    | left_child_page_id
0x04   | var  | key_size (varint)
0x??   | var  | key_data
```

#### B+Tree 叶子节点布局

```
Leaf Node Cell:
Offset | Size | Field
-------|------|------------------
0x00   | var  | payload_size (varint)
0x??   | var  | row_id (varint, 等同于 INTEGER PRIMARY KEY)
0x??   | var  | payload (序列化的行数据)

Payload 格式（类似 SQLite Record Format）:
- Header size (varint)
- Column type codes (varint array, 每列一个)
- Column data (紧密排列)
```

**ASCII 图示（叶子节点示例）**:

```
Page Header:
  page_type=0x0D (leaf)
  num_cells=3

Cell Pointer Array:
  [0x0F80][0x0E00][0x0C80]  <- 指向 cell content 偏移

Unallocated Space: 0x0020 - 0x0C7F

Cell Content (从页尾向上):
Offset 0x0F80:
  payload_size=50 row_id=1 [header][type_codes][data...]
Offset 0x0E00:
  payload_size=48 row_id=2 [header][type_codes][data...]
Offset 0x0C80:
  payload_size=52 row_id=3 [header][type_codes][data...]
```

#### WAL 文件格式

```
WAL File Structure:
+-------------------+
| WAL Header        | (32 bytes)
+-------------------+
| Frame 1           | (4128 bytes: 4096 page + 32 frame header)
+-------------------+
| Frame 2           |
+-------------------+
| ...               |
+-------------------+

WAL Header:
Offset | Size | Field
-------|------|------------------
0x00   | 4    | magic (0x377F0682)
0x04   | 4    | version
0x08   | 4    | page_size
0x0C   | 4    | checkpoint_seq
0x10   | 8    | salt-1 (随机值，每次 checkpoint 更新)
0x18   | 4    | checksum-1
0x1C   | 4    | checksum-2

WAL Frame Header:
Offset | Size | Field
-------|------|------------------
0x00   | 4    | page_id
0x04   | 4    | db_size (commit 时的数据库页数)
0x08   | 8    | salt-1 (copy from WAL header)
0x10   | 4    | checksum-1 (frame header + page data)
0x14   | 4    | checksum-2
0x18   | 8    | reserved (预留用于 MVCC txn_id)
-------|------|------------------
0x20   | 4096 | page_data
```

**Checksum 算法**（与 SQLite 兼容）:

```rust
fn wal_checksum(data: &[u8], prev_c1: u32, prev_c2: u32) -> (u32, u32) {
    let mut c1 = prev_c1;
    let mut c2 = prev_c2;
    
    for chunk in data.chunks_exact(8) {
        let x = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let y = u32::from_be_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
        
        c1 = c1.wrapping_add(x).wrapping_add(c2);
        c2 = c2.wrapping_add(y).wrapping_add(c1);
    }
    
    (c1, c2)
}
```

#### Checkpoint 流程图

```
          +------------------+
          | BEGIN CHECKPOINT |
          +------------------+
                    |
                    v
          +------------------+
          | Acquire WAL lock |
          +------------------+
                    |
                    v
          +------------------+
          | Read all frames  |
          | from WAL         |
          +------------------+
                    |
                    v
          +----------------------+
          | For each frame:      |
          | - Verify checksum    |
          | - Write page to DB   |
          +----------------------+
                    |
                    v
          +----------------------+
          | Fsync database file  |
          +----------------------+
                    |
                    v
          +----------------------+
          | Truncate WAL file    |
          | (or mark checkpoint) |
          +----------------------+
                    |
                    v
          +----------------------+
          | Update WAL header    |
          | (new salt, seq++)    |
          +----------------------+
                    |
                    v
          +------------------+
          | Release WAL lock |
          +------------------+
                    |
                    v
          +------------------+
          | CHECKPOINT DONE  |
          +------------------+
```

### unsafe / Pin / Interior Mutability 使用场景

#### 场景 1: Pager 页内指针操作

**位置**: `rdb-storage/src/pager.rs`

**原因**: 需要从 `Page.data: [u8; 4096]` 中解析结构，并返回指向页内数据的引用。

```rust
impl<'db> Pager<'db> {
    /// UNSAFE: 调用者必须确保：
    /// 1. page_id 有效且在文件范围内
    /// 2. 返回的指针在 Pager 生命周期内有效
    /// 3. 不会在持有指针时修改页
    pub unsafe fn get_page_ptr(&self, page_id: PageId) -> *const Page<'db> {
        // 实现：从 mmap 或缓存中获取页指针
    }
}
```

#### 场景 2: B+Tree 节点内切片引用

**位置**: `rdb-storage/src/btree/node.rs`

**原因**: B+Tree 节点的 keys/values 需要直接指向 Page.data 内部，避免拷贝。

```rust
impl<'tree, K, V> BTreeNode<'tree, K, V> {
    /// UNSAFE: 调用者必须确保：
    /// 1. page 包含有效的 B+Tree 节点数据
    /// 2. page 不会在节点生命周期内被修改或释放
    /// 3. keys/values 偏移量和长度正确（已验证）
    pub unsafe fn from_page(page: Pin<&'tree mut Page<'tree>>) -> Self {
        let header = parse_page_header(&page.data);
        let keys_ptr = page.data.as_ptr().add(header.keys_offset) as *const K;
        let keys = std::slice::from_raw_parts(keys_ptr, header.num_keys);
        // ... 类似处理 values
        
        Self { page, keys, values, ... }
    }
}
```

**为什么用 Pin**: 防止 Page 被移动，保证节点内指针有效。

#### 场景 3: BufferPool 内部可变性

**位置**: `rdb-infrastructure/src/buffer_pool.rs`

**原因**: BufferPool 需要在共享引用下修改缓存（LRU 更新），但用户持有 `&Page`。

```rust
pub struct BufferPool {
    cache: RefCell<LruCache<PageId, Page>>, // interior mutability
    latch: RwLock<()>, // 保护并发访问
}

impl BufferPool {
    /// 返回共享引用，但内部需要更新 LRU
    pub fn get_page(&self, page_id: PageId) -> Result<&Page> {
        let _guard = self.latch.read();
        // 需要 RefCell::borrow_mut 来更新 LRU
        let mut cache = self.cache.borrow_mut(); // 运行时借用检查
        cache.get(&page_id).ok_or(Error::PageNotFound)
    }
}
```

**为什么不用 Mutex**: `RefCell` 运行时检查更灵活，且此处有外层 RwLock 保护。

#### 场景 4: WAL Writer 并发写入

**位置**: `rdb-storage/src/wal.rs`

**原因**: 需要在多个事务并发写入 WAL 时保证顺序和原子性。

```rust
pub struct WalWriter {
    file: Mutex<File>, // interior mutability for write
    current_offset: AtomicU64, // lock-free read
}

impl WalWriter {
    pub fn append_frame(&self, frame: &WalFrame) -> Result<u64> {
        let mut file = self.file.lock();
        let offset = self.current_offset.fetch_add(frame.size(), Ordering::SeqCst);
        file.write_all(frame.as_bytes())?;
        Ok(offset)
    }
}
```

### SQLite 源码参考映射表

| rdb 模块                     | SQLite 源文件          | 说明                                |
|-----------------------------|----------------------|-----------------------------------|
| `rdb-storage/pager.rs`      | `pager.c`            | 页管理、缓存、磁盘 IO                    |
| `rdb-storage/btree/mod.rs`  | `btree.c`            | B+Tree 实现                         |
| `rdb-storage/btree/node.rs` | `btree.c` (节点操作)    | 节点分裂、合并、查找                       |
| `rdb-storage/wal.rs`        | `wal.c`              | Write-Ahead Logging                |
| `rdb-storage/page.rs`       | `btree.h` (page def) | 页格式定义                             |
| `rdb-sql/parser.rs`         | `parse.y` (Lemon)    | SQL 解析（rdb 使用 sqlparser-rs）      |
| `rdb-sql/executor.rs`       | `vdbe.c`             | 虚拟机执行器（rdb 使用迭代器而非虚拟机）         |
| `rdb-sql/planner.rs`        | `where.c`, `select.c`| 查询计划与优化                          |
| `rdb-domain/transaction.rs` | `vdbe.c` (txn logic) | 事务管理                              |
| `rdb-infrastructure/buffer_pool.rs` | `pcache.c` | 页缓存                               |

## 52 周开发路线图

### 阶段 0: 基础设施（Week 1-4）

#### Week 1: 项目初始化
- **Milestone**: Cargo workspace 创建，所有 crate 骨架就绪
- **交付物**: 
  - Cargo.toml workspace 配置
  - 所有 crate 目录结构
  - CI/CD 配置（GitHub Actions）
  - README 和文档框架
- **验收测试**: `cargo build --all` 成功编译

#### Week 2: 领域层基础类型
- **Milestone**: 领域层核心类型定义完成
- **交付物**:
  - `Database`, `Table`, `Column`, `Row`, `Value` 类型
  - 数据类型系统（INTEGER, TEXT, BLOB, REAL, NULL）
  - 基础错误类型定义
- **验收测试**: 
  - 单元测试覆盖所有类型构造
  - proptest: `Value` 序列化/反序列化往返测试

#### Week 3: Page 与 Pager 基础
- **Milestone**: 4KB 页式存储实现
- **交付物**:
  - `Page` 结构定义（4KB 对齐）
  - `Pager` 实现（读取、写入、缓存）
  - 文件 IO 封装
- **验收测试**:
  - 创建数据库文件并写入 100 个页
  - proptest: 页读写一致性测试

#### Week 4: BufferPool 与 LRU 缓存
- **Milestone**: 页缓存系统实现
- **交付物**:
  - LRU 缓存实现
  - Pin 机制（防止热页被淘汰）
  - 缓存统计（命中率、淘汰次数）
- **验收测试**:
  - 缓存容量限制测试
  - LRU 淘汰算法正确性测试
  - 并发读取测试（10 线程）

---

### 阶段 1: B+Tree 存储引擎（Week 5-12）

#### Week 5: B+Tree 叶子节点
- **Milestone**: B+Tree 叶子节点读写
- **交付物**:
  - 叶子节点格式定义
  - Cell 序列化/反序列化
  - 键值插入、查找
- **验收测试**:
  - 单页插入 100 个键值对
  - proptest: 插入后查找必定成功

#### Week 6: B+Tree 内部节点
- **Milestone**: B+Tree 内部节点与树高度扩展
- **交付物**:
  - 内部节点格式
  - 树遍历算法
  - 节点分裂（叶子节点分裂）
- **验收测试**:
  - 插入 1000 个键，触发节点分裂
  - proptest: B+Tree 有序性不变量

#### Week 7: B+Tree 节点合并与删除
- **Milestone**: 支持 DELETE 操作
- **交付物**:
  - 键值删除逻辑
  - 节点合并（merge）和借用（borrow）
  - 树高度收缩
- **验收测试**:
  - 插入 10000 个键后删除 9000 个，验证树结构正确
  - proptest: 随机插入删除后树保持平衡

#### Week 8: B+Tree 范围查询
- **Milestone**: 支持范围扫描
- **交付物**:
  - 范围查询迭代器
  - 前向/后向扫描
- **验收测试**:
  - 查询 `key >= 100 AND key < 200` 返回正确结果
  - proptest: 范围查询结果有序且完整

#### Week 9: B+Tree 并发读
- **Milestone**: 多线程并发读取支持
- **交付物**:
  - Latch coupling（闩锁耦合）实现
  - 读锁优化
- **验收测试**:
  - 10 个线程并发查询，无数据竞争
  - 压力测试：100 线程并发读

#### Week 10: B+Tree 性能优化
- **Milestone**: 性能达到目标指标
- **交付物**:
  - 批量插入优化
  - 预分配页优化
  - SIMD 加速（可选）
- **验收测试**:
  - 顺序插入性能 > 10,000 行/秒
  - 主键查询 < 1ms (10万行数据)

#### Week 11: Freelist 管理
- **Milestone**: 页空间复用
- **交付物**:
  - Freelist 数据结构
  - 删除页回收逻辑
  - 碎片整理（可选）
- **验收测试**:
  - 删除数据后文件大小不继续增长
  - proptest: freelist 无重复页

#### Week 12: B+Tree 集成测试
- **Milestone**: B+Tree 功能完整
- **交付物**:
  - 端到端测试套件
  - 性能基准测试
- **验收测试**:
  - 所有 proptest 通过
  - 性能基准达标

---

### 阶段 2: WAL 与事务（Week 13-20）

#### Week 13: WAL 文件格式
- **Milestone**: WAL 写入实现
- **交付物**:
  - WAL header 和 frame 格式
  - WAL 文件创建与写入
- **验收测试**:
  - 写入 1000 个 frame
  - 验证 checksum 正确性

#### Week 14: WAL 读取与恢复
- **Milestone**: 崩溃恢复
- **交付物**:
  - WAL 读取逻辑
  - 恢复算法（重放 frames）
- **验收测试**:
  - 模拟崩溃，验证数据恢复
  - proptest: 恢复后数据一致性

#### Week 15: Checkpoint 机制
- **Milestone**: WAL 同步到主文件
- **交付物**:
  - Checkpoint 流程实现
  - WAL 截断逻辑
- **验收测试**:
  - Checkpoint 后 WAL 文件清空
  - 性能测试：100ms 内完成 checkpoint (1MB WAL)

#### Week 16: 事务 BEGIN/COMMIT
- **Milestone**: 事务生命周期管理
- **交付物**:
  - `Transaction` 类型
  - BEGIN, COMMIT 逻辑
  - WAL 提交点标记
- **验收测试**:
  - COMMIT 后数据持久化
  - 未 COMMIT 数据不可见

#### Week 17: 事务 ROLLBACK
- **Milestone**: 事务回滚
- **交付物**:
  - Undo log（或利用 WAL）
  - ROLLBACK 逻辑
- **验收测试**:
  - ROLLBACK 后数据恢复到事务前状态
  - proptest: 随机 COMMIT/ROLLBACK 保持一致性

#### Week 18: 写锁机制
- **Milestone**: 单写多读并发模型
- **交付物**:
  - 写锁获取与释放
  - 锁超时机制
- **验收测试**:
  - 两个写事务不能并发（第二个等待或失败）
  - 读事务不阻塞

#### Week 19: MVCC 接口预留
- **Milestone**: 为未来 MVCC 预留钩子
- **交付物**:
  - 事务 ID 字段
  - LSN（日志序列号）字段
  - 快照读接口定义（未实现）
- **验收测试**:
  - 编译通过，接口文档完整

#### Week 20: 事务集成测试
- **Milestone**: ACID 属性验证
- **交付物**:
  - 原子性测试（崩溃恢复）
  - 隔离性测试（脏读测试）
  - 持久性测试（断电模拟）
- **验收测试**:
  - 所有 ACID 测试通过

---

### 阶段 3: SQL 解析与执行（Week 21-32）

#### Week 21: SQL 解析器集成
- **Milestone**: sqlparser-rs 集成
- **交付物**:
  - 封装 sqlparser-rs
  - 类型安全的 AST 转换
- **验收测试**:
  - 解析 50 个 SQL 语句成功

#### Week 22: CREATE TABLE 实现
- **Milestone**: DDL 支持
- **交付物**:
  - CREATE TABLE 执行器
  - 表元数据持久化（系统表 `sqlite_master`）
- **验收测试**:
  - 创建表后重启，表定义仍存在

#### Week 23: INSERT 实现
- **Milestone**: 数据插入
- **交付物**:
  - INSERT 执行器
  - 行数据序列化
  - 自动递增 rowid
- **验收测试**:
  - 插入 10000 行数据
  - 验证数据正确性

#### Week 24: SELECT 基础查询
- **Milestone**: 全表扫描查询
- **交付物**:
  - SELECT 执行器（迭代器）
  - 投影（选择列）
  - WHERE 过滤（简单表达式）
- **验收测试**:
  - `SELECT * FROM table WHERE id > 100` 返回正确结果

#### Week 25: SELECT 聚合函数
- **Milestone**: COUNT, SUM, AVG, MIN, MAX
- **交付物**:
  - 聚合函数实现
  - GROUP BY（简单情况）
- **验收测试**:
  - `SELECT COUNT(*) FROM table` 返回正确行数

#### Week 26: UPDATE 与 DELETE
- **Milestone**: 数据修改
- **交付物**:
  - UPDATE 执行器
  - DELETE 执行器
  - 索引更新（如果存在）
- **验收测试**:
  - UPDATE 后查询返回新值
  - DELETE 后数据不可见

#### Week 27: CREATE INDEX
- **Milestone**: 索引创建
- **交付物**:
  - CREATE INDEX 执行器
  - 索引构建（扫描表并插入 B+Tree）
  - 索引元数据持久化
- **验收测试**:
  - 创建索引后重启，索引仍存在

#### Week 28: 索引查询优化
- **Milestone**: 使用索引加速查询
- **交付物**:
  - 查询计划器（选择索引）
  - 索引扫描执行器
- **验收测试**:
  - `SELECT * FROM table WHERE name = 'Alice'` 使用索引（EXPLAIN 验证）
  - 性能提升 > 10x

#### Week 29: ORDER BY 与 LIMIT
- **Milestone**: 排序和分页
- **交付物**:
  - 排序执行器（内存排序或利用索引）
  - LIMIT 和 OFFSET
- **验收测试**:
  - `SELECT * FROM table ORDER BY id LIMIT 10` 返回前 10 行

#### Week 30: JOIN 实现（嵌套循环）
- **Milestone**: 多表关联
- **交付物**:
  - INNER JOIN 执行器（嵌套循环）
  - JOIN 条件评估
- **验收测试**:
  - `SELECT * FROM users JOIN orders ON users.id = orders.user_id` 返回正确结果

#### Week 31: JOIN 优化（Hash Join）
- **Milestone**: 性能优化
- **交付物**:
  - Hash Join 实现
  - 查询计划器选择 JOIN 策略
- **验收测试**:
  - 大表 JOIN 性能测试

#### Week 32: SQL 集成测试
- **Milestone**: SQL 功能完整
- **交付物**:
  - 端到端 SQL 测试套件
  - 兼容性测试（与 SQLite 对比）
- **验收测试**:
  - 所有 SQL 测试通过

---

### 阶段 4: 接口层与 API（Week 33-36）

#### Week 33: 公共 API 设计
- **Milestone**: rdb-interface 定义
- **交付物**:
  - `Database::open()` API
  - `Connection::execute()` API
  - `Statement::query()` API（参数化查询）
- **验收测试**:
  - 文档示例代码能运行

#### Week 34: 结果迭代器
- **Milestone**: 流式查询结果
- **交付物**:
  - `Rows` 迭代器
  - 类型安全的列访问（`row.get::<i64>(0)`）
- **验收测试**:
  - 查询 100 万行，内存占用 < 10MB

#### Week 35: 错误处理
- **Milestone**: 友好的错误信息
- **交付物**:
  - `RdbError` 类型
  - 错误分类（IO, SQL, Corruption）
  - 错误上下文（文件名、行号）
- **验收测试**:
  - 错误信息包含有用的调试信息

#### Week 36: 线程安全保证
- **Milestone**: API 线程安全
- **交付物**:
  - `Database: Send + Sync`
  - `Connection: !Send + !Sync`（每线程一个连接）
  - 文档明确线程模型
- **验收测试**:
  - 多线程压力测试

---

### 阶段 5: 高级特性（Week 37-44）

#### Week 37: 约束支持（NOT NULL, UNIQUE）
- **Milestone**: 数据完整性
- **交付物**:
  - NOT NULL 约束检查
  - UNIQUE 约束检查（使用索引）
- **验收测试**:
  - 违反约束时返回错误

#### Week 38: 外键支持（基础）
- **Milestone**: 引用完整性
- **交付物**:
  - FOREIGN KEY 定义
  - 插入/删除时检查外键
- **验收测试**:
  - 级联删除测试

#### Week 39: VIEW 支持
- **Milestone**: 视图
- **交付物**:
  - CREATE VIEW
  - 视图查询展开
- **验收测试**:
  - 查询视图等同于查询底层表

#### Week 40: 触发器（基础）
- **Milestone**: TRIGGER 支持
- **交付物**:
  - CREATE TRIGGER
  - BEFORE/AFTER INSERT/UPDATE/DELETE
- **验收测试**:
  - 触发器正确执行

#### Week 41: 子查询支持
- **Milestone**: 嵌套查询
- **交付物**:
  - 子查询执行器
  - IN, EXISTS 支持
- **验收测试**:
  - `SELECT * FROM users WHERE id IN (SELECT user_id FROM orders)` 正确执行

#### Week 42: EXPLAIN 查询计划
- **Milestone**: 查询调试
- **交付物**:
  - EXPLAIN 输出
  - 查询计划可视化
- **验收测试**:
  - EXPLAIN 输出可读

#### Week 43: VACUUM 与碎片整理
- **Milestone**: 数据库维护
- **交付物**:
  - VACUUM 命令
  - 重建数据库文件
- **验收测试**:
  - VACUUM 后文件大小缩减

#### Week 44: PRAGMA 命令
- **Milestone**: 数据库配置
- **交付物**:
  - PRAGMA page_size, cache_size, journal_mode 等
- **验收测试**:
  - 设置 PRAGMA 影响行为

---

### 阶段 6: 性能优化与测试（Week 45-48）

#### Week 45: 性能基准测试
- **Milestone**: 性能对比 SQLite
- **交付物**:
  - criterion benchmark 套件
  - 与 SQLite 性能对比报告
- **验收测试**:
  - 核心操作性能差距 < 2x SQLite

#### Week 46: 内存优化
- **Milestone**: 内存占用优化
- **交付物**:
  - 减少不必要的拷贝
  - 内存池（arena allocator）
- **验收测试**:
  - 内存占用减少 30%

#### Week 47: 模糊测试（Fuzzing）
- **Milestone**: 鲁棒性测试
- **交付物**:
  - cargo-fuzz 集成
  - 模糊测试套件
- **验收测试**:
  - 24 小时模糊测试无崩溃

#### Week 48: 压力测试
- **Milestone**: 稳定性验证
- **交付物**:
  - 长时间运行测试（7x24h）
  - 并发压力测试（1000 线程）
- **验收测试**:
  - 无内存泄漏，无数据损坏

---

### 阶段 7: 文档与发布（Week 49-52）

#### Week 49: API 文档完善
- **Milestone**: 完整的 Rustdoc
- **交付物**:
  - 所有公共 API 都有文档和示例
  - 架构文档
- **验收测试**:
  - `cargo doc` 无警告

#### Week 50: 用户指南
- **Milestone**: 教程和最佳实践
- **交付物**:
  - 快速开始指南
  - 迁移指南（从 SQLite）
  - 性能调优指南
- **验收测试**:
  - 新用户能在 5 分钟内运行示例

#### Week 51: 生态集成
- **Milestone**: 集成到 Rust 生态
- **交付物**:
  - crates.io 发布准备
  - CI/CD 完善
  - CHANGELOG 和版本号
- **验收测试**:
  - 通过 crates.io 审核

#### Week 52: v1.0.0 发布
- **Milestone**: 正式发布
- **交付物**:
  - 发布公告
  - 演示视频
  - 社区反馈收集
- **验收测试**:
  - 下载量 > 1000（第一周）

---

## Proptest 示例

### 示例 1: B+Tree 有序性不变量

```rust
// rdb-storage/tests/proptest_btree.rs
use proptest::prelude::*;

proptest! {
    #[test]
    fn btree_maintains_order(ops in vec(btree_op(), 0..1000)) {
        let mut tree = BTree::new();
        
        for op in ops {
            match op {
                Op::Insert(k, v) => tree.insert(k, v),
                Op::Delete(k) => tree.delete(&k),
            }
        }
        
        // 验证不变量：所有键有序
        let keys: Vec<_> = tree.iter().map(|(k, _)| k).collect();
        prop_assert!(keys.windows(2).all(|w| w[0] < w[1]));
    }
}

#[derive(Debug, Clone)]
enum Op {
    Insert(u64, Vec<u8>),
    Delete(u64),
}

fn btree_op() -> impl Strategy<Value = Op> {
    prop_oneof![
        (any::<u64>(), vec(any::<u8>(), 0..100)).prop_map(|(k, v)| Op::Insert(k, v)),
        any::<u64>().prop_map(Op::Delete),
    ]
}
```

### 示例 2: 事务原子性

```rust
proptest! {
    #[test]
    fn transaction_atomicity(
        initial_data in vec((any::<u64>(), any::<i64>()), 0..100),
        txn_ops in vec(sql_op(), 0..50),
        should_commit in any::<bool>()
    ) {
        let db = Database::open(":memory:").unwrap();
        
        // 初始数据
        for (k, v) in &initial_data {
            db.execute(&format!("INSERT INTO t VALUES ({}, {})", k, v)).unwrap();
        }
        
        let snapshot_before = dump_table(&db, "t");
        
        // 执行事务
        db.execute("BEGIN").unwrap();
        for op in txn_ops {
            let _ = db.execute(&op); // 可能失败
        }
        
        if should_commit {
            db.execute("COMMIT").unwrap();
            // 数据应该改变（或保持不变）
        } else {
            db.execute("ROLLBACK").unwrap();
            // 数据必须完全恢复
            let snapshot_after = dump_table(&db, "t");
            prop_assert_eq!(snapshot_before, snapshot_after);
        }
    }
}
```

### 示例 3: WAL 恢复一致性

```rust
proptest! {
    #[test]
    fn wal_recovery_consistency(
        operations in vec(write_op(), 0..100),
        crash_point in 0..100usize
    ) {
        let path = tempfile::NamedTempFile::new().unwrap().path().to_owned();
        
        // 执行操作直到崩溃点
        {
            let db = Database::open(&path).unwrap();
            for op in &operations[..crash_point.min(operations.len())] {
                db.execute(&op.to_sql()).unwrap();
            }
            // 模拟崩溃（不调用 db.close()）
        }
        
        // 重新打开数据库（触发 WAL 恢复）
        let db = Database::open(&path).unwrap();
        
        // 验证：所有已提交的事务都存在
        for op in &operations[..crash_point.min(operations.len())] {
            if op.is_committed() {
                prop_assert!(verify_op_applied(&db, op));
            }
        }
    }
}
```

## 假设与约束

### 假设

1. **单机部署**：v1.0 仅支持单机嵌入式场景，不支持网络多客户端
2. **Linux/macOS 优先**：优先支持 Unix-like 系统，Windows 作为次要目标
3. **页大小固定**：4KB 页大小（与 SQLite 默认一致），不支持运行时修改
4. **内存限制**：假设可用内存至少 100MB（用于缓存池）

### 约束

1. **性能目标**：核心操作性能不低于 SQLite 的 50%（作为 v1.0 基线）
2. **并发模型**：v1.0 仅支持单写多读（与 SQLite 默认模式一致）
3. **SQL 兼容性**：不追求 100% SQL 标准兼容，以 SQLite 常用功能为基准
4. **依赖限制**：仅依赖 `sqlparser-rs`、`parking_lot`、`thiserror` 等少量成熟 crate

## 下一步行动

本规格说明完成后，建议：

1. **执行 `/speckit.plan`** 创建详细的技术实现计划
2. **执行 `/speckit.tasks`** 生成第一个 Milestone（Week 1）的任务清单
3. **开始迭代开发**，按周提交 Milestone

---

**文档版本**: 1.0  
**最后更新**: 2025-12-10
