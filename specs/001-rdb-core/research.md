# 技术研究报告：rdb 存储引擎与架构设计

**Feature**: rdb 嵌入式关系型数据库  
**Phase**: 0 - Research & Technology Selection  
**Date**: 2025-12-10

## 执行摘要

本研究报告为 rdb 数据库的实现提供技术决策依据。所有决策基于工业验证的方案（SQLite、CockroachDB、TiDB），并严格遵循 rdb Constitution 的 7 条核心原则。

**关键结论**：
- ✅ 采用 B+Tree + WAL 架构（SQLite 验证方案）
- ✅ 使用 sqlparser-rs 避免重复造轮子
- ✅ 4KB 页大小与操作系统对齐
- ✅ 单写多读并发模型（v1.0），预留 MVCC 接口

---

## 1. 存储引擎架构选择

### 决策：B+Tree + Write-Ahead Log (WAL)

**选择理由**：
1. **工业验证**：SQLite 使用超过 20 年，极度稳定
2. **读写性能平衡**：B+Tree 提供 O(log n) 读写，WAL 提供顺序写优化
3. **崩溃恢复**：WAL 天然支持 ACID 事务和崩溃恢复
4. **范围查询友好**：B+Tree 叶子节点连接，适合 SQL 的 ORDER BY 和范围扫描

**替代方案评估**：

| 方案 | 优点 | 缺点 | 为何未采用 |
|------|------|------|-----------|
| **LSM-Tree** (RocksDB) | 写入极快，压缩率高 | 读放大严重，需要多层合并 | rdb 目标是通用数据库，读写平衡更重要；且 RocksDB 是 C++ 需要 FFI |
| **HashMap + AOF** | 实现简单 | 无范围查询，不支持 ORDER BY | SQL 需要范围查询和排序 |
| **Copy-on-Write B-Tree** (LMDB) | MVCC 天然支持 | 写入时复制开销大 | v1.0 不需要 MVCC，未来可升级 |

**参考实现**：
- SQLite: `pager.c` + `btree.c` + `wal.c`
- 文档：[SQLite File Format](https://www.sqlite.org/fileformat.html)

---

## 2. 页大小选择

### 决策：4KB 固定页大小

**选择理由**：
1. **操作系统对齐**：Linux/macOS 默认页大小为 4KB，减少内存拷贝
2. **mmap 友好**：内存映射文件时，4KB 是最小粒度
3. **缓存友好**：CPU L1 cache line 通常 64 字节，4KB = 64 个 cache line
4. **SQLite 默认值**：SQLite 默认 4096 字节，已被广泛验证

**替代方案评估**：

| 页大小 | 优点 | 缺点 | 为何未采用 |
|--------|------|------|-----------|
| **8KB/16KB** | 减少元数据开销，大记录存储效率高 | 小记录浪费空间，缓存效率降低 | rdb 目标通用场景，不偏向大记录 |
| **512B/1KB** | 小记录空间利用率高 | 元数据开销大，B+Tree 深度增加 | 现代系统内存充足，优化意义不大 |
| **可变页大小** | 灵活适应不同记录 | 实现复杂，碎片管理困难 | 违反 Simplicity 原则 |

**实现细节**：
```rust
#[repr(C, align(4096))]
pub struct Page {
    data: [u8; 4096],
    // ...
}
```

---

## 3. B+Tree 设计

### 决策：经典 B+Tree（内部节点仅存键，叶子节点存数据）

**选择理由**：
1. **范围查询高效**：叶子节点链表结构，顺序扫描无需回溯
2. **键值分离**：内部节点仅存键，可容纳更多分支，减少树高度
3. **SQLite 兼容**：方便参考 SQLite 的 `btree.c` 实现

**关键参数**：
- **阶数（Order）**：动态计算，取决于键大小（目标 50-200 个键/节点）
- **最小填充率**：50%（标准 B+Tree 要求）
- **分裂策略**：节点满时分裂为两个 50% 填充的节点

**节点类型**：
```rust
pub enum NodeType {
    Internal, // 内部节点：[key1, ptr1, key2, ptr2, ...]
    Leaf,     // 叶子节点：[key1, value1, key2, value2, ...]
}
```

**并发控制（v1.0）**：
- **Latch Coupling**（闩锁耦合）：读操作获取读锁，写操作获取写锁
- **单写多读**：同一时刻只有一个写事务，多个读事务不互斥

**MVCC 预留（v2.0）**：
- 叶子节点 value 包含版本链指针（当前为 NULL）
- 每个记录包含 `created_txn_id` 和 `deleted_txn_id` 字段（预留）

**参考**：
- SQLite: `btree.c` (5000+ 行)
- 教材：《Database System Concepts》第 11 章 Indexing

---

## 4. Write-Ahead Log (WAL) 设计

### 决策：SQLite WAL 格式兼容设计

**选择理由**：
1. **经过验证**：SQLite WAL 格式已在生产环境使用超过 10 年
2. **崩溃恢复简单**：重放 WAL 即可恢复到一致状态
3. **Checkpoint 灵活**：可配置何时将 WAL 同步到主文件

**WAL 文件格式**：
```
[WAL Header: 32 bytes]
[Frame 1: 32 bytes header + 4096 bytes page data]
[Frame 2: 32 bytes header + 4096 bytes page data]
...
```

**关键字段**：
- **Salt**: 每次 checkpoint 更新，防止旧 WAL 被误用
- **Checksum**: 双重校验和（累积式），检测损坏
- **db_size**: commit 时的数据库页数，支持回滚

**Checkpoint 策略**：
- **Passive**: WAL 达到阈值（默认 1000 页）时自动触发
- **Full**: 关闭数据库时强制 checkpoint
- **Truncate**: checkpoint 后截断 WAL 文件

**MVCC 预留**：
- Frame header 预留 8 字节用于未来的 `txn_id`（事务 ID）
- 支持按事务 ID 重放部分 WAL（快照读）

**参考**：
- SQLite: `wal.c` (4000+ 行)
- 文档：[SQLite WAL Mode](https://www.sqlite.org/wal.html)

---

## 5. SQL 解析器选择

### 决策：使用 sqlparser-rs

**选择理由**：
1. **成熟稳定**：sqlparser-rs 是 Rust 生态最成熟的 SQL 解析库（4000+ stars）
2. **方言支持**：支持 PostgreSQL, MySQL, SQLite 等多种 SQL 方言
3. **避免重复造轮子**：SQL 解析是已解决的问题，无需重新实现
4. **类型安全**：返回类型化的 AST，避免字符串操作

**支持的 SQL 特性**（sqlparser-rs 0.47+）：
- ✅ DDL: CREATE TABLE, DROP TABLE, CREATE INDEX, DROP INDEX
- ✅ DML: INSERT, UPDATE, DELETE, SELECT
- ✅ WHERE: 表达式、比较、逻辑运算符
- ✅ JOIN: INNER, LEFT, RIGHT, CROSS
- ✅ 聚合: COUNT, SUM, AVG, MIN, MAX, GROUP BY
- ✅ 排序: ORDER BY, LIMIT, OFFSET
- ✅ 子查询: IN, EXISTS, scalar subquery

**替代方案评估**：

| 方案 | 优点 | 缺点 | 为何未采用 |
|------|------|------|-----------|
| **手写递归下降解析器** | 完全控制，可定制 | 工作量大（2000+ 行），维护成本高 | 不符合 Simplicity 原则 |
| **Pest PEG 解析器** | 声明式语法，简洁 | 性能较低，错误信息不友好 | SQL 语法复杂，PEG 不适合 |
| **nom 解析器组合子** | 零拷贝，高性能 | 学习曲线陡峭，代码可读性差 | sqlparser-rs 已足够快 |

**集成方式**：
```rust
use sqlparser::ast::Statement;
use sqlparser::dialect::SQLiteDialect;
use sqlparser::parser::Parser;

pub fn parse_sql(sql: &str) -> Result<Statement> {
    let dialect = SQLiteDialect {};
    let ast = Parser::parse_sql(&dialect, sql)?;
    Ok(ast[0].clone())
}
```

**参考**：
- 项目：[sqlparser-rs](https://github.com/sqlparser-rs/sqlparser-rs)
- 文档：[API Docs](https://docs.rs/sqlparser/)

---

## 6. 查询执行模型

### 决策：迭代器模型（Iterator Model / Volcano Model）

**选择理由**：
1. **内存高效**：流式处理，不需要一次性加载所有结果
2. **Rust 友好**：`Iterator` trait 是 Rust 核心抽象
3. **组合性好**：不同算子（Scan, Filter, Join）可以组合
4. **惰性求值**：仅在需要时计算，支持 LIMIT 优化

**执行器接口**：
```rust
pub trait Executor: Iterator<Item = Result<Row>> {
    fn explain(&self) -> String; // 查询计划说明
}
```

**算子类型**：
- **SeqScan**: 全表顺序扫描
- **IndexScan**: 使用 B+Tree 索引扫描
- **Filter**: WHERE 条件过滤
- **Project**: SELECT 列投影
- **HashJoin**: Hash Join 算法
- **NestedLoopJoin**: 嵌套循环 Join
- **Aggregate**: 聚合函数（COUNT, SUM 等）

**替代方案评估**：

| 模型 | 优点 | 缺点 | 为何未采用 |
|------|------|------|-----------|
| **虚拟机模型**（SQLite VDBE） | 灵活，支持复杂控制流 | 实现复杂（3000+ 行），调试困难 | 迭代器模型更符合 Rust 习惯 |
| **编译模型**（HyPer） | 性能极高（LLVM JIT） | 实现极复杂，依赖 LLVM | v1.0 不需要极致性能 |
| **向量化模型**（DuckDB） | 批处理高效，SIMD 友好 | 与迭代器模型冲突 | v1.0 优先简单，v2.0 可优化 |

**参考**：
- 论文：[Volcano - An Extensible and Parallel Query Evaluation System](https://paperhub.s3.amazonaws.com/dace52a42c07f7f8348b08dc2b186061.pdf)
- 代码：Apache Arrow DataFusion (Rust 查询引擎)

---

## 7. 并发控制模型

### 决策：v1.0 单写多读，v2.0 乐观 MVCC

**v1.0: 单写多读（类似 SQLite 默认模式）**

**选择理由**：
1. **实现简单**：一个写锁 + 多个读锁，无需复杂的版本管理
2. **足够用**：大多数嵌入式数据库场景写入不频繁
3. **渐进式**：v1.0 验证架构正确性，v2.0 再优化并发

**实现方式**：
```rust
pub struct LockManager {
    write_lock: Mutex<()>,      // 写锁（独占）
    read_count: AtomicU32,      // 读锁计数
}
```

**规则**：
- 读事务：获取读锁，可并发
- 写事务：获取写锁，阻塞其他写事务
- 写事务不阻塞读事务（读取快照）

**v2.0: 乐观 MVCC（Multi-Version Concurrency Control）**

**预留设计**：
- 每个事务有唯一的 `txn_id`（单调递增）
- 每行记录包含 `created_txn_id` 和 `deleted_txn_id`
- 读事务看到 `txn_id <= snapshot_version` 的数据

**可见性规则**：
```rust
fn is_visible(row: &Row, snapshot_version: TxnId) -> bool {
    row.created_txn_id <= snapshot_version
        && (row.deleted_txn_id.is_none() || row.deleted_txn_id > snapshot_version)
}
```

**参考**：
- PostgreSQL MVCC 实现
- CockroachDB 的时间戳排序（Timestamp Ordering）

---

## 8. 数据类型系统

### 决策：SQLite 类型系统（动态类型 + 亲和性）

**支持的类型**：
```rust
pub enum Value<'v> {
    Null,
    Integer(i64),        // 64-bit 整数
    Real(f64),           // 64-bit 浮点数
    Text(Cow<'v, str>),  // UTF-8 字符串
    Blob(Cow<'v, [u8]>), // 二进制数据
}
```

**类型亲和性（Type Affinity）**：
- 列定义为 `INTEGER` → 尝试将值转换为整数
- 列定义为 `TEXT` → 尝试将值转换为文本
- 存储时保留原始类型

**选择理由**：
1. **简单灵活**：无需复杂的类型转换规则
2. **SQLite 兼容**：用户熟悉的行为
3. **Rust 友好**：`Cow<'v, str>` 避免不必要的拷贝

**未来扩展**：
- v1.1: DATE, TIME, DATETIME（存储为 INTEGER 或 TEXT）
- v1.2: JSON（存储为 TEXT，提供 JSON 函数）

---

## 9. 错误处理策略

### 决策：使用 thiserror 定义错误层次

**错误类型**：
```rust
#[derive(Error, Debug)]
pub enum RdbError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("SQL syntax error: {0}")]
    SqlSyntax(String),
    
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
    
    #[error("Database corruption detected: {0}")]
    Corruption(String),
    
    #[error("Transaction error: {0}")]
    Transaction(String),
}
```

**错误传播**：
- 内部层：使用 `Result<T, RdbError>`
- 公共 API：转换为用户友好的错误信息

**选择理由**：
1. **类型安全**：编译器强制错误处理
2. **上下文信息**：每个错误包含位置和原因
3. **用户友好**：公共 API 错误信息清晰

---

## 10. 测试策略

### 决策：多层次测试（单元 + 集成 + 属性 + 模糊）

**测试类型**：

| 测试类型 | 工具 | 覆盖范围 | 目标 |
|---------|------|---------|------|
| **单元测试** | `cargo test` | 每个函数/模块 | 功能正确性 |
| **集成测试** | `cargo test --test` | 跨模块交互 | 组件集成 |
| **属性测试** | `proptest` | B+Tree, WAL, 事务 | 不变量验证 |
| **模糊测试** | `cargo-fuzz` | SQL 解析, 存储引擎 | 边界情况 |
| **基准测试** | `criterion` | 关键路径 | 性能回归 |

**proptest 示例**：
```rust
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
```

**测试覆盖率目标**：
- 核心模块（storage, btree, wal）：≥ 90%
- 其他模块：≥ 80%
- unsafe 代码：100% + Miri 验证

---

## 11. 依赖管理

### 决策：最小化依赖，优先使用成熟 crate

**直接依赖列表**（预计 < 20 个）：

| Crate | 用途 | 版本 | 理由 |
|-------|------|------|------|
| `sqlparser` | SQL 解析 | 0.47+ | 成熟稳定，避免重复造轮子 |
| `parking_lot` | 高性能锁 | 0.12+ | 比 std::sync 快 2-5x |
| `thiserror` | 错误定义 | 1.0+ | 标准错误处理库 |
| `proptest` | 属性测试 | 1.4+ | 验证不变量 |
| `criterion` | 基准测试 | 0.5+ | 性能回归检测 |
| `serde` (可选) | 序列化 | 1.0+ | 导出数据为 JSON |

**避免的依赖**：
- ❌ `tokio`, `async-std`: v1.0 不需要异步 IO
- ❌ `diesel`, `sqlx`: rdb 本身是数据库，不需要 ORM
- ❌ `rocksdb`, `sled`: 避免嵌入其他数据库引擎

**依赖审查标准**：
1. ✅ crates.io 上至少 100k 下载
2. ✅ 最近 6 个月内有维护
3. ✅ 无已知安全漏洞（cargo-audit）
4. ✅ 许可证兼容（MIT/Apache-2.0）

---

## 12. 性能优化策略

### 初期优化（Week 1-20）

**优先级**：正确性 > 可维护性 > 性能

**基础优化**：
1. **避免不必要的拷贝**：使用 `Cow<'a, str>` 和引用
2. **内存对齐**：`#[repr(C, align(4096))]` 优化缓存
3. **批量操作**：事务中批量插入避免单次 fsync

### 后期优化（Week 45-48）

**性能热点**（通过 flamegraph 识别）：
1. B+Tree 节点查找 → SIMD 加速二分查找
2. Page 序列化/反序列化 → 零拷贝设计
3. WAL 写入 → 批量 fsync

**基准测试目标**：
- 顺序插入：≥ 10,000 行/秒
- 随机插入：≥ 5,000 行/秒
- 主键查询：< 1ms p95
- 索引范围查询：< 10ms (扫描 1000 行)

**参考**：
- SQLite 性能特征：[SQLite Speed Comparison](https://www.sqlite.org/speed.html)
- 优化手册：《Database Internals》第 4 章

---

## 13. 部署与分发

### 决策：发布为 Rust crate（crates.io）

**发布计划**：
- v0.1.0-alpha: Week 20（基础功能）
- v0.5.0-beta: Week 40（高级特性）
- v1.0.0: Week 52（生产就绪）

**文档**：
- README: 快速开始
- rustdoc: API 文档
- mdBook: 用户指南

**CI/CD**：
- GitHub Actions
- 测试矩阵：Rust stable/nightly, Linux/macOS/Windows

---

## 总结

本研究报告涵盖了 rdb 实现的所有关键技术决策。所有决策基于以下原则：

1. ✅ **工业验证**：优先采用 SQLite/PostgreSQL 等成熟方案
2. ✅ **渐进式**：v1.0 先保证正确性，v2.0 再优化性能
3. ✅ **Rust 友好**：充分利用 Rust 的类型系统和零成本抽象
4. ✅ **宪章合规**：严格遵循 rdb Constitution 的 7 条原则

**下一步**：进入 Phase 1 设计阶段，生成领域模型和 API 契约。

---

**Research Version**: 1.0  
**Prepared By**: Technical Research Team  
**Date**: 2025-12-10

