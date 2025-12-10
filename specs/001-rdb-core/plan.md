# Implementation Plan: rdb 嵌入式关系型数据库核心实现

**Branch**: `001-rdb-core` | **Date**: 2025-12-10 | **Spec**: [spec.md](./spec.md)  
**Input**: Feature specification from `/specs/001-rdb-core/spec.md`

## Summary

rdb 是一个使用纯 Rust 实现的嵌入式关系型数据库，采用严格的 DDD 架构。核心目标是提供类似 SQLite 的功能，但从第一天起就为 MVCC 和集群化预留接口。技术方案基于工业验证的存储引擎设计（B+Tree + WAL），使用 sqlparser-rs 进行 SQL 解析，所有代码必须满足 rdb Constitution 定义的 7 条核心原则。

**关键技术决策**：
- **存储引擎**：4KB 页式存储 + B+Tree 索引 + WAL 事务日志
- **SQL 层**：sqlparser-rs 解析 + 迭代器模式执行器
- **架构**：Cargo workspace 多 crate DDD 分层
- **并发模型**：v1.0 单写多读（类似 SQLite），预留 MVCC 接口

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel, 2021 edition)  
**Primary Dependencies**:
- `sqlparser` (0.47+) - SQL 解析
- `parking_lot` (0.12+) - 高性能锁原语
- `thiserror` (1.0+) - 错误定义
- `proptest` (1.4+) - 属性测试
- `criterion` (0.5+) - 性能基准测试

**Storage**: 自实现文件存储（无外部数据库依赖）
- 主数据库文件：`.db` 后缀
- WAL 文件：`.db-wal` 后缀
- 4KB 页对齐，支持 mmap（可选）

**Testing**:
- `cargo test` - 单元测试和集成测试
- `proptest` - 属性测试（B+Tree 不变量、事务 ACID）
- `criterion` - 性能回归测试
- `miri` - unsafe 代码验证（CI）

**Target Platform**:
- Primary: Linux (Ubuntu 22.04+, kernel 5.15+) 和 macOS (12.0+)
- Secondary: Windows 10+ (基础支持)
- 架构：x86_64 和 aarch64

**Project Type**: Rust workspace 多 crate 库项目（嵌入式数据库）

**Performance Goals**:
- 顺序插入：≥ 10,000 行/秒（无索引，单线程）
- 主键查询：< 1ms p95（10 万行数据规模）
- 索引加速比：≥ 10x vs 全表扫描
- Checkpoint：< 100ms（1MB WAL）
- 启动时间：< 50ms（空数据库）

**Constraints**:
- 内存占用：启动 < 5MB，缓存池可配置（默认 4MB = 1024 页）
- 文件格式：100% 确定性（支持字节级对比）
- unsafe 代码：仅限 `rdb-storage` crate 的 Pager 和 B+Tree 模块
- 依赖数量：< 20 个直接依赖（避免依赖膨胀）

**Scale/Scope**:
- v1.0 目标：单个数据库文件 ≤ 1GB
- 表数量：< 1000 个表/数据库
- 并发：10 个读线程 + 1 个写线程
- 查询复杂度：支持 3-4 表 JOIN

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Verify compliance with [rdb Constitution](../../.specify/memory/constitution.md):

- [x] **DDD Architecture**: Design follows Domain/Application/Infrastructure/Interface layers
  - ✅ Cargo workspace 分为 6 个 crate：domain, storage, infrastructure, sql, application, interface
  
- [x] **Pure Rust**: No external database kernels or FFI dependencies planned
  - ✅ 所有存储、索引、查询处理均用 Rust 实现
  - ✅ 依赖仅限 Rust 生态的 crate（sqlparser, parking_lot, thiserror, proptest）
  
- [x] **Memory Safety**: unsafe usage limited to Pager/B+Tree with documented invariants
  - ✅ unsafe 仅用于：
    1. Pager: 页内指针操作（从 `[u8; 4096]` 解析结构）
    2. B+Tree: 节点内切片引用（指向 Page.data）
  - ✅ 每个 unsafe 块将包含 SAFETY 注释说明不变量
  
- [x] **MVCC Interface**: Storage design includes version metadata and snapshot support
  - ✅ Page header 预留 8 字节 LSN（日志序列号）字段
  - ✅ WAL frame header 预留 8 字节用于未来事务 ID
  - ✅ Transaction 类型包含 snapshot_version 字段（当前未使用）
  
- [x] **API Boundaries**: Public API minimal, internal types use pub(crate)
  - ✅ 仅 `rdb-interface` crate 公开（`pub`）
  - ✅ 其他所有 crate 使用 `pub(crate)` 或私有
  - ✅ 公共 API：`Database::open()`, `Connection::execute()`, `Statement::query()`
  
- [x] **Property Tests**: proptest tests planned for all domain entities
  - ✅ 计划的 proptest：
    1. B+Tree 有序性不变量
    2. 事务原子性（COMMIT/ROLLBACK）
    3. WAL 恢复一致性
    4. Page 分配/释放（Freelist 无重复）
  
- [x] **Cluster-Ready**: Storage format includes replication metadata and versioning
  - ✅ 文件头包含版本号和特性标志
  - ✅ Page header 预留 LSN 和 8 字节 reserved 字段
  - ✅ WAL frame 格式与 Raft 日志兼容（term/index 可映射）

**Result**: ✅ All checks passed - no violations to justify

## Project Structure

### Documentation (this feature)

```text
specs/001-rdb-core/
├── plan.md              # This file
├── research.md          # Phase 0: 技术研究与决策
├── data-model.md        # Phase 1: 领域模型定义
├── quickstart.md        # Phase 1: 快速开始指南
├── contracts/           # Phase 1: API 契约
│   ├── storage-api.md   # 存储层接口
│   ├── sql-api.md       # SQL 层接口
│   └── public-api.md    # 公共 Rust API
└── tasks.md             # Phase 2: 任务清单（/speckit.tasks 生成）
```

### Source Code (repository root)

```text
rdb/
├── Cargo.toml           # Workspace 根配置
├── README.md
├── LICENSE
├── .gitignore
│
├── rdb-domain/          # 领域层 crate
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs       # pub(crate) 模块根
│   │   ├── database.rs  # Database 聚合根
│   │   ├── table.rs     # Table 实体
│   │   ├── column.rs    # Column 值对象
│   │   ├── transaction.rs  # Transaction 聚合根
│   │   ├── row.rs       # Row 实体
│   │   ├── value.rs     # Value 值对象（数据类型）
│   │   ├── index.rs     # Index 实体
│   │   └── error.rs     # 领域错误类型
│   └── tests/
│       └── proptest_domain.rs  # 领域实体属性测试
│
├── rdb-storage/         # 存储引擎 crate（唯一允许 unsafe）
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── pager.rs     # Pager: 页管理和缓存
│   │   ├── page.rs      # Page: 4KB 页定义
│   │   ├── btree/       # B+Tree 模块
│   │   │   ├── mod.rs
│   │   │   ├── node.rs  # 节点操作（内部/叶子）
│   │   │   ├── cursor.rs   # 游标（迭代器）
│   │   │   └── split.rs    # 节点分裂/合并
│   │   ├── wal.rs       # WAL 写入器
│   │   ├── wal_reader.rs   # WAL 读取和恢复
│   │   ├── freelist.rs  # 空闲页管理
│   │   └── checksum.rs  # 校验和算法
│   └── tests/
│       ├── proptest_btree.rs   # B+Tree 属性测试
│       ├── proptest_wal.rs     # WAL 恢复测试
│       └── integration/
│           └── crash_recovery.rs  # 崩溃恢复测试
│
├── rdb-infrastructure/  # 基础设施层 crate
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── buffer_pool.rs  # 页缓存池（LRU）
│   │   ├── lock_manager.rs # 锁管理器
│   │   └── file_io.rs      # 文件 IO 封装
│   └── tests/
│       └── buffer_pool_test.rs
│
├── rdb-sql/             # SQL 层 crate
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── parser.rs    # SQL 解析器（封装 sqlparser-rs）
│   │   ├── planner/     # 查询计划器
│   │   │   ├── mod.rs
│   │   │   ├── logical_plan.rs  # 逻辑计划
│   │   │   └── physical_plan.rs # 物理计划
│   │   ├── optimizer.rs # 查询优化器
│   │   └── executor/    # 查询执行器
│   │       ├── mod.rs
│   │       ├── scan.rs      # 全表扫描
│   │       ├── index_scan.rs   # 索引扫描
│   │       ├── filter.rs    # WHERE 过滤
│   │       ├── project.rs   # SELECT 投影
│   │       ├── aggregate.rs # 聚合函数
│   │       └── join.rs      # JOIN 执行
│   └── tests/
│       └── sql_integration_test.rs
│
├── rdb-application/     # 应用层 crate
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── database_service.rs  # 数据库服务
│   │   ├── transaction_service.rs  # 事务服务
│   │   └── query_service.rs     # 查询服务
│   └── tests/
│
├── rdb-interface/       # 接口层 crate（唯一公开）
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs       # 公共 API 入口
│   │   ├── database.rs  # Database 公共类型
│   │   ├── connection.rs   # Connection 公共类型
│   │   ├── statement.rs    # Statement 公共类型
│   │   ├── rows.rs      # Rows 迭代器
│   │   └── error.rs     # RdbError 公共错误
│   ├── examples/
│   │   ├── basic_usage.rs
│   │   ├── transactions.rs
│   │   └── indexes.rs
│   └── tests/
│       └── api_test.rs
│
├── tests/               # 端到端集成测试
│   ├── e2e_crud.rs
│   ├── e2e_transactions.rs
│   ├── e2e_indexes.rs
│   └── e2e_recovery.rs
│
└── benches/             # 性能基准测试
    ├── insert_benchmark.rs
    ├── query_benchmark.rs
    └── btree_benchmark.rs
```

**Structure Decision**:

采用 **Cargo workspace 多 crate 结构**（严格 DDD 分层），理由：

1. **明确的层次边界**：每个 crate 对应一个 DDD 层，编译器强制依赖方向（领域层不依赖基础设施层）
2. **可测试性**：每层独立测试，领域逻辑测试无需 IO 或存储
3. **unsafe 隔离**：仅 `rdb-storage` crate 允许 unsafe，其他层完全安全
4. **公共 API 控制**：仅 `rdb-interface` 公开，其他 crate 使用 `pub(crate)`，防止实现细节泄漏
5. **增量编译优化**：修改某层不重新编译无关层

**依赖关系**（自底向上）：
```
rdb-interface
    ↓
rdb-application
    ↓        ↓
rdb-sql   rdb-domain
    ↓        ↓
rdb-infrastructure
    ↓
rdb-storage
```

## Complexity Tracking

> **此项目无宪章违规** - 所有设计符合 7 条核心原则。

无需填写此表格。

---

## Phase 0: Research Summary

详见 [research.md](./research.md) 获取完整的技术研究报告。

### 关键决策总结

1. **存储引擎选择**：B+Tree + WAL（基于 SQLite pager.c 和 btree.c 设计）
2. **页大小**：4KB（与操作系统页大小对齐，优化 mmap 和缓存）
3. **并发模型**：单写多读 + 读写锁（v1.0），预留 MVCC 接口（v2.0）
4. **SQL 解析器**：sqlparser-rs（成熟的 Rust SQL 解析库，避免重复造轮子）
5. **测试策略**：proptest 属性测试 + 示例测试 + 模糊测试（cargo-fuzz）

---

## Phase 1: Design Artifacts

### Data Model

详见 [data-model.md](./data-model.md) 获取完整的领域模型定义。

**核心聚合根**：
- `Database`: 数据库实例，管理表和事务
- `Transaction`: 事务生命周期，保证 ACID 属性

**关键实体**：
- `Table`, `Index`, `Row`, `Column`

**值对象**：
- `Value<'v>`: 数据库值（INTEGER, TEXT, BLOB, REAL, NULL）
- `DataType`: 列数据类型定义

### API Contracts

详见 [contracts/](./contracts/) 目录获取完整的 API 契约定义。

**公共 Rust API**（`rdb-interface`）：
```rust
// 打开或创建数据库
pub fn Database::open(path: impl AsRef<Path>) -> Result<Database>;

// 执行 SQL 语句
pub fn Connection::execute(&self, sql: &str) -> Result<usize>;

// 查询并返回迭代器
pub fn Statement::query(&self, params: &[Value]) -> Result<Rows>;
```

### Quickstart Guide

详见 [quickstart.md](./quickstart.md) 获取快速开始指南。

**5 分钟上手示例**：
```rust
use rdb_interface::{Database, Value};

// 打开数据库
let db = Database::open("test.db")?;

// 创建表
db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)")?;

// 插入数据
db.execute("INSERT INTO users VALUES (1, 'Alice')")?;

// 查询数据
let rows = db.query("SELECT * FROM users")?;
for row in rows {
    let id: i64 = row.get(0)?;
    let name: String = row.get(1)?;
    println!("id={}, name={}", id, name);
}
```

---

## Implementation Roadmap

### Phase Breakdown

实现分为 7 个主要阶段，共 52 周：

| 阶段 | 周数 | 重点 | 关键里程碑 |
|------|------|------|-----------|
| 阶段 0 | 1-4 | 基础设施 | Workspace、领域类型、Pager、BufferPool |
| 阶段 1 | 5-12 | B+Tree 存储引擎 | 节点插入/删除、范围查询、并发读 |
| 阶段 2 | 13-20 | WAL 与事务 | WAL 读写、崩溃恢复、ACID 保证 |
| 阶段 3 | 21-32 | SQL 解析与执行 | DDL/DML、索引、JOIN |
| 阶段 4 | 33-36 | 接口层 | 公共 API、错误处理、线程安全 |
| 阶段 5 | 37-44 | 高级特性 | 约束、外键、VIEW、触发器 |
| 阶段 6 | 45-48 | 性能优化 | 基准测试、内存优化、模糊测试 |
| 阶段 7 | 49-52 | 文档与发布 | API 文档、用户指南、v1.0.0 |

详细的周级别 Milestone 见 [spec.md](./spec.md) 中的"52 周开发路线图"章节。

### Critical Path

**阻塞依赖**（必须按顺序）：
1. Week 1-4: 基础设施 → 阻塞所有后续工作
2. Week 5-12: B+Tree → 阻塞 SQL 执行和事务
3. Week 13-20: WAL → 阻塞事务持久性
4. Week 21-32: SQL → 阻塞公共 API

**并行机会**：
- Week 5-12: B+Tree 开发 || BufferPool 优化
- Week 21-32: SQL 解析 || 查询优化器（不同人员）
- Week 37-44: 高级特性可按优先级并行开发

### Risk Mitigation

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| B+Tree 分裂/合并实现复杂 | 高 | Week 7 专门 Milestone，参考 SQLite btree.c |
| WAL 崩溃恢复边界情况 | 高 | Week 14 专门测试，proptest 模拟崩溃 |
| unsafe 代码内存安全 | 严重 | Miri CI 检查，代码审查，详细 SAFETY 注释 |
| 性能不达标 | 中 | Week 45 基准测试，与 SQLite 对比，早期识别瓶颈 |
| 依赖 sqlparser-rs 限制 | 低 | 评估阶段验证支持的 SQL 特性 |

---

## Next Steps

1. **创建任务清单**：执行 `/speckit.tasks` 生成第一个 Milestone（Week 1）的任务
2. **设置 Cargo workspace**：创建 6 个 crate 目录结构
3. **开始 Week 1 开发**：项目初始化、CI/CD 配置、README

---

**Plan Version**: 1.0  
**Last Updated**: 2025-12-10  
**Status**: Ready for Phase 2 (Task Generation)
