# Tasks: rdb 嵌入式关系型数据库核心实现

**Input**: 设计文档来自 `/specs/001-rdb-core/`  
**Prerequisites**: plan.md (必需), spec.md (必需), research.md, data-model.md, contracts/

**注意**: 本任务清单基于 52 周开发路线图，每周一个 Git 提交点（Milestone）

**组织方式**: 按阶段组织，每个阶段包含多个 Week Milestone

## Format: `[ID] [P?] Description with file path`

- **[P]**: 可并行执行（不同文件，无依赖）
- 每个任务都包含精确的文件路径
- Week X 是 Git 提交点，完成后提交代码

## Path Conventions

- **Cargo workspace**: 根目录下 `rdb/` 包含 6 个 crate
- **rdb-domain/**: `rdb/rdb-domain/src/`
- **rdb-storage/**: `rdb/rdb-storage/src/`
- **rdb-infrastructure/**: `rdb/rdb-infrastructure/src/`
- **rdb-sql/**: `rdb/rdb-sql/src/`
- **rdb-application/**: `rdb/rdb-application/src/`
- **rdb-interface/**: `rdb/rdb-interface/src/`

---

## 阶段 0: 基础设施 (Week 1-4)

### Week 1: 项目初始化

**Milestone**: Cargo workspace 创建，所有 crate 骨架就绪  
**Git Commit**: `feat: initialize cargo workspace and project structure`  
**领域实体**: 无（项目初始化）  
**核心 Struct**: 无  
**测试数量**: 1 个（编译测试）

**要完成的工作**:

- [ ] T001 创建 Cargo workspace 根配置文件 `Cargo.toml`
- [ ] T002 [P] 创建 `rdb-domain` crate 目录结构和 `rdb-domain/Cargo.toml`
- [ ] T003 [P] 创建 `rdb-storage` crate 目录结构和 `rdb-storage/Cargo.toml`
- [ ] T004 [P] 创建 `rdb-infrastructure` crate 目录结构和 `rdb-infrastructure/Cargo.toml`
- [ ] T005 [P] 创建 `rdb-sql` crate 目录结构和 `rdb-sql/Cargo.toml`
- [ ] T006 [P] 创建 `rdb-application` crate 目录结构和 `rdb-application/Cargo.toml`
- [ ] T007 [P] 创建 `rdb-interface` crate 目录结构和 `rdb-interface/Cargo.toml`
- [ ] T008 配置 GitHub Actions CI 文件 `.github/workflows/ci.yml`
- [ ] T009 [P] 创建项目 README 文件 `README.md`
- [ ] T010 [P] 创建 LICENSE 文件 `LICENSE`
- [ ] T011 [P] 配置 `.gitignore` 文件
- [ ] T012 [P] 配置 `rustfmt.toml` 格式化规则
- [ ] T013 [P] 配置 `clippy.toml` Lint 规则
- [ ] T014 验证编译：`cargo build --all` 成功

**备注**: 
- Week 1 主要是搭架子，确保 6 个 crate 能互相依赖且编译通过
- CI 配置包括：格式检查、Lint、测试、构建
- **坑**: workspace 依赖路径要正确（`path = "../rdb-domain"`）

---

### Week 2: 领域层基础类型

**Milestone**: 领域层核心类型定义完成  
**Git Commit**: `feat(domain): implement core domain types`  
**领域实体**: `Database`, `Table`, `Column`, `Row`, `Value`  
**核心 Struct**: 
- `rdb-domain/src/database.rs`: `Database` 聚合根
- `rdb-domain/src/table.rs`: `Table` 实体
- `rdb-domain/src/column.rs`: `Column` 实体
- `rdb-domain/src/row.rs`: `Row` 实体
- `rdb-domain/src/value.rs`: `Value<'v>` 值对象

**测试数量**: 10+ 个单元测试，2 个 proptest

**要完成的工作**:

- [ ] T015 [P] 实现 ID 类型（newtype 模式）在 `rdb-domain/src/ids.rs`: `TableId`, `ColumnId`, `RowId`, `PageId`, `IndexId`, `TransactionId`, `LockId`
- [ ] T016 [P] 实现 `DataType` 枚举在 `rdb-domain/src/data_type.rs`: `Integer`, `Real`, `Text`, `Blob`
- [ ] T017 实现 `Value<'v>` 枚举在 `rdb-domain/src/value.rs`: `Null`, `Integer(i64)`, `Real(f64)`, `Text(Cow<'v, str>)`, `Blob(Cow<'v, [u8]>)`
- [ ] T018 实现 `Column` 结构在 `rdb-domain/src/column.rs`: 列定义（name, data_type, constraints）
- [ ] T019 实现 `ColumnConstraints` 结构在 `rdb-domain/src/column.rs`: `not_null`, `unique`, `primary_key`, `autoincrement`
- [ ] T020 实现 `Table` 结构在 `rdb-domain/src/table.rs`: 表定义（id, name, columns, primary_key, root_page）
- [ ] T021 实现 `Row` 结构在 `rdb-domain/src/row.rs`: 行数据（row_id, values）
- [ ] T022 实现 `Database` 聚合根在 `rdb-domain/src/database.rs`: 数据库实例（path, tables, indexes, schema_version）
- [ ] T023 [P] 实现错误类型在 `rdb-domain/src/error.rs`: `DomainError` 枚举
- [ ] T024 [P] 添加单元测试在 `rdb-domain/tests/domain_tests.rs`: 所有类型的构造和验证测试
- [ ] T025 添加 proptest 在 `rdb-domain/tests/proptest_value.rs`: `Value` 序列化/反序列化往返测试

**备注**: 
- 所有类型都要标注生命周期和 Send/Sync 要求
- `Value<'v>` 使用 `Cow` 避免不必要的拷贝
- **坑**: 生命周期参数容易出错，确保 `'static` 和 `'v` 使用正确

---

### Week 3: Page 与 Pager 基础

**Milestone**: 4KB 页式存储实现  
**Git Commit**: `feat(storage): implement page and pager`  
**领域实体**: `Page`, `Pager`  
**核心 Struct**: 
- `rdb-storage/src/page.rs`: `Page<'page>` (4KB 数据页)
- `rdb-storage/src/pager.rs`: `Pager<'db>` (页管理器)

**测试数量**: 8+ 个单元测试，2 个 proptest

**要完成的工作**:

- [ ] T026 实现 `PageType` 枚举在 `rdb-storage/src/page.rs`: `Internal`, `Leaf`, `Overflow`, `Freelist`
- [ ] T027 实现 `PageHeader` 结构在 `rdb-storage/src/page.rs`: 32 字节页头定义
- [ ] T028 实现 `Page<'page>` 结构在 `rdb-storage/src/page.rs`: 4KB 对齐的数据页（`#[repr(C, align(4096))]`）
- [ ] T029 实现 `Page` 的基础方法在 `rdb-storage/src/page.rs`: `new()`, `from_bytes()`, `page_id()`, `page_type()`, `mark_dirty()`, `data()`, `data_mut()`
- [ ] T030 实现页头解析和写入方法在 `rdb-storage/src/page.rs`: `parse_header()`, `write_header()`
- [ ] T031 实现 `Pager<'db>` 结构在 `rdb-storage/src/pager.rs`: 页管理器（file, page_size, page_count, buffer_pool）
- [ ] T032 实现 `Pager` 的基础方法在 `rdb-storage/src/pager.rs`: `new()`, `get_page()`, `get_page_mut()`, `allocate_page()`, `flush_page()`, `flush_all()`
- [ ] T033 实现文件 IO 封装在 `rdb-infrastructure/src/file_io.rs`: 文件读写辅助函数
- [ ] T034 [P] 添加单元测试在 `rdb-storage/tests/page_tests.rs`: 页创建、读写、header 解析测试
- [ ] T035 [P] 添加单元测试在 `rdb-storage/tests/pager_tests.rs`: 页管理、分配、刷新测试
- [ ] T036 添加 proptest 在 `rdb-storage/tests/proptest_page.rs`: 页读写一致性测试

**备注**: 
- `Page` 必须 4KB 对齐，使用 `#[repr(C, align(4096))]`
- `Pager` 需要使用 `PhantomData` 标记生命周期
- **坑**: 页内指针操作涉及 unsafe，确保所有 unsafe 块都有 SAFETY 注释

---

### Week 4: BufferPool 与 LRU 缓存

**Milestone**: 页缓存系统实现  
**Git Commit**: `feat(infrastructure): implement buffer pool with LRU cache`  
**领域实体**: `BufferPool`  
**核心 Struct**: 
- `rdb-infrastructure/src/buffer_pool.rs`: `BufferPool` (页缓存池)

**测试数量**: 8+ 个单元测试，1 个并发测试

**要完成的工作**:

- [ ] T037 实现 LRU 缓存数据结构在 `rdb-infrastructure/src/lru_cache.rs`: 通用 LRU cache 实现
- [ ] T038 实现 `BufferPool` 结构在 `rdb-infrastructure/src/buffer_pool.rs`: 页缓存池（cache: RefCell<LruCache>, latch: RwLock）
- [ ] T039 实现 `BufferPool` 的基础方法在 `rdb-infrastructure/src/buffer_pool.rs`: `new()`, `get_page()`, `pin_page()`, `unpin_page()`, `flush_page()`, `flush_all()`
- [ ] T040 实现缓存统计功能在 `rdb-infrastructure/src/buffer_pool.rs`: 缓存命中率、淘汰次数统计
- [ ] T041 [P] 添加单元测试在 `rdb-infrastructure/tests/buffer_pool_tests.rs`: 缓存容量、LRU 淘汰、Pin 机制测试
- [ ] T042 添加并发测试在 `rdb-infrastructure/tests/buffer_pool_concurrency_tests.rs`: 10 线程并发读取测试

**备注**: 
- `BufferPool` 使用 `RefCell` 实现 interior mutability
- Pin 机制防止热页被淘汰
- **坑**: `RefCell` 运行时借用检查，确保不会 panic

---

## 阶段 1: B+Tree 存储引擎 (Week 5-12)

### Week 5: B+Tree 叶子节点

**Milestone**: B+Tree 叶子节点读写  
**Git Commit**: `feat(storage): implement btree leaf nodes`  
**领域实体**: `BTreeNode` (Leaf)  
**核心 Struct**: 
- `rdb-storage/src/btree/node.rs`: `BTreeNode<'tree, K, V>`
- `rdb-storage/src/btree/cell.rs`: `LeafCell`

**测试数量**: 6+ 个单元测试，1 个 proptest

**要完成的工作**:

- [ ] T043 创建 BTree 模块在 `rdb-storage/src/btree/mod.rs`: 模块结构定义
- [ ] T044 实现叶子节点 Cell 格式在 `rdb-storage/src/btree/cell.rs`: `LeafCell` 结构（payload_size, row_id, payload）
- [ ] T045 实现 Cell 序列化/反序列化在 `rdb-storage/src/btree/cell.rs`: `serialize()`, `deserialize()` 方法
- [ ] T046 实现 `BTreeNode<'tree, K, V>` 结构在 `rdb-storage/src/btree/node.rs`: 节点抽象（page, is_leaf, num_keys, keys, values）
- [ ] T047 实现 unsafe `from_page()` 方法在 `rdb-storage/src/btree/node.rs`: 从 Page 创建节点（包含详细 SAFETY 注释）
- [ ] T048 实现叶子节点查找方法在 `rdb-storage/src/btree/node.rs`: `search()` 二分查找
- [ ] T049 实现叶子节点插入方法在 `rdb-storage/src/btree/node.rs`: `insert_cell()` 键值对插入
- [ ] T050 [P] 添加单元测试在 `rdb-storage/tests/btree_leaf_tests.rs`: 叶子节点创建、插入、查找测试
- [ ] T051 添加 proptest 在 `rdb-storage/tests/proptest_btree.rs`: 插入后查找必定成功

**备注**: 
- `BTreeNode` 的 keys/values 直接指向 Page.data，避免拷贝
- 使用 `Pin<&'tree mut Page>` 防止页被移动
- **坑**: unsafe 指针操作，确保偏移量和长度计算正确

---

### Week 6: B+Tree 内部节点

**Milestone**: B+Tree 内部节点与树高度扩展  
**Git Commit**: `feat(storage): implement btree internal nodes and tree growth`  
**领域实体**: `BTreeNode` (Internal), `BTree`  
**核心 Struct**: 
- `rdb-storage/src/btree/node.rs`: `BTreeNode` (内部节点)
- `rdb-storage/src/btree/mod.rs`: `BTree<'tree, K, V>`

**测试数量**: 8+ 个单元测试，1 个 proptest

**要完成的工作**:

- [ ] T052 实现内部节点 Cell 格式在 `rdb-storage/src/btree/cell.rs`: `InternalCell` 结构（left_child_page_id, key_size, key_data）
- [ ] T053 实现内部节点查找方法在 `rdb-storage/src/btree/node.rs`: `search_internal()` 定位子节点
- [ ] T054 实现节点分裂方法在 `rdb-storage/src/btree/split.rs`: `split_leaf()` 和 `split_internal()`
- [ ] T055 实现 `BTree<'tree, K, V>` 结构在 `rdb-storage/src/btree/mod.rs`: B+Tree 主结构（root_page, pager, order）
- [ ] T056 实现 `BTree::new()` 方法在 `rdb-storage/src/btree/mod.rs`: 创建新 B+Tree
- [ ] T057 实现 `BTree::open()` 方法在 `rdb-storage/src/btree/mod.rs`: 从已有根页打开 B+Tree
- [ ] T058 实现 `BTree::insert()` 方法在 `rdb-storage/src/btree/mod.rs`: 插入键值对（支持节点分裂）
- [ ] T059 实现 `BTree::get()` 方法在 `rdb-storage/src/btree/mod.rs`: 查找键
- [ ] T060 实现树遍历方法在 `rdb-storage/src/btree/mod.rs`: 递归查找路径
- [ ] T061 [P] 添加单元测试在 `rdb-storage/tests/btree_tests.rs`: 插入 1000 个键，触发节点分裂
- [ ] T062 添加 proptest 在 `rdb-storage/tests/proptest_btree.rs`: B+Tree 有序性不变量测试

**备注**: 
- 节点分裂时创建兄弟节点，并更新父节点
- 树高度增长时需要创建新根节点
- **坑**: 分裂逻辑复杂，参考 SQLite btree.c 实现

---

### Week 7: B+Tree 节点合并与删除

**Milestone**: 支持 DELETE 操作  
**Git Commit**: `feat(storage): implement btree node merge and deletion`  
**领域实体**: 无（扩展 BTree）  
**核心 Struct**: 
- `rdb-storage/src/btree/merge.rs`: 节点合并和借用逻辑

**测试数量**: 8+ 个单元测试，1 个 proptest

**要完成的工作**:

- [ ] T063 实现 `BTree::remove()` 方法在 `rdb-storage/src/btree/mod.rs`: 删除键值对
- [ ] T064 实现节点合并方法在 `rdb-storage/src/btree/merge.rs`: `merge_leaves()` 和 `merge_internal()`
- [ ] T065 实现节点借用方法在 `rdb-storage/src/btree/merge.rs`: `borrow_from_left()` 和 `borrow_from_right()`
- [ ] T066 实现树高度收缩逻辑在 `rdb-storage/src/btree/mod.rs`: 根节点只有一个子节点时下降树高
- [ ] T067 [P] 添加单元测试在 `rdb-storage/tests/btree_delete_tests.rs`: 插入 10000 个键后删除 9000 个，验证树结构
- [ ] T068 添加 proptest 在 `rdb-storage/tests/proptest_btree.rs`: 随机插入删除后树保持平衡

**备注**: 
- 删除后需要检查节点填充率，低于 50% 时触发合并或借用
- 合并和借用逻辑比分裂更复杂
- **坑**: 边界条件多，仔细测试根节点和叶子节点的删除

---

### Week 8: B+Tree 范围查询

**Milestone**: 支持范围扫描  
**Git Commit**: `feat(storage): implement btree range query cursor`  
**领域实体**: `BTreeCursor`  
**核心 Struct**: 
- `rdb-storage/src/btree/cursor.rs`: `BTreeCursor<'tree, K, V>`

**测试数量**: 6+ 个单元测试，1 个 proptest

**要完成的工作**:

- [ ] T069 实现 `BTreeCursor` 结构在 `rdb-storage/src/btree/cursor.rs`: 游标（current_page, cell_index, pager）
- [ ] T070 实现 `Iterator` trait 在 `rdb-storage/src/btree/cursor.rs`: 流式迭代 B+Tree
- [ ] T071 实现 `BTree::range()` 方法在 `rdb-storage/src/btree/mod.rs`: 范围查询，返回游标
- [ ] T072 实现 `BTree::iter()` 方法在 `rdb-storage/src/btree/mod.rs`: 迭代所有键值对
- [ ] T073 实现叶子节点链表在 `rdb-storage/src/btree/node.rs`: 叶子节点包含 `next_leaf` 指针
- [ ] T074 [P] 添加单元测试在 `rdb-storage/tests/btree_range_tests.rs`: 查询 `key >= 100 AND key < 200` 返回正确结果
- [ ] T075 添加 proptest 在 `rdb-storage/tests/proptest_btree.rs`: 范围查询结果有序且完整

**备注**: 
- 游标在叶子层水平移动，通过 `next_leaf` 指针
- 范围查询定位起始键后顺序扫描
- **坑**: 游标生命周期绑定到 BTree，确保不会悬空

---

### Week 9: B+Tree 并发读

**Milestone**: 多线程并发读取支持  
**Git Commit**: `feat(storage): implement concurrent btree reads with latch coupling`  
**领域实体**: 无（扩展 BTree）  
**核心 Struct**: 无（添加锁机制）

**测试数量**: 4+ 个并发测试

**要完成的工作**:

- [ ] T076 实现读锁机制在 `rdb-storage/src/btree/latch.rs`: 页级读写锁
- [ ] T077 实现 Latch Coupling 在 `rdb-storage/src/btree/mod.rs`: 查找时逐层获取读锁
- [ ] T078 优化 `BTree::get()` 支持并发读在 `rdb-storage/src/btree/mod.rs`: 使用 `&self` 而非 `&mut self`
- [ ] T079 [P] 添加并发测试在 `rdb-storage/tests/btree_concurrency_tests.rs`: 10 个线程并发查询，无数据竞争
- [ ] T080 添加压力测试在 `rdb-storage/tests/btree_stress_tests.rs`: 100 线程并发读

**备注**: 
- Latch Coupling: 获取子节点锁后释放父节点锁
- 读锁不互斥，写锁互斥所有读写
- **坑**: 死锁风险，确保锁的获取顺序一致（从根到叶）

---

### Week 10: B+Tree 性能优化

**Milestone**: 性能达到目标指标  
**Git Commit**: `perf(storage): optimize btree performance`  
**领域实体**: 无（性能优化）  
**核心 Struct**: 无

**测试数量**: 4+ 个基准测试

**要完成的工作**:

- [ ] T081 实现批量插入优化在 `rdb-storage/src/btree/mod.rs`: `insert_batch()` 方法
- [ ] T082 实现页预分配在 `rdb-storage/src/pager.rs`: 一次性分配多个连续页
- [ ] T083 优化二分查找在 `rdb-storage/src/btree/node.rs`: 使用 SIMD 加速（可选）
- [ ] T084 [P] 添加基准测试在 `benches/btree_benchmark.rs`: 顺序插入、随机插入、查询性能测试
- [ ] T085 性能调优：确保顺序插入性能 > 10,000 行/秒
- [ ] T086 性能调优：确保主键查询 < 1ms (10万行数据)

**备注**: 
- 使用 criterion 进行基准测试
- 批量插入时可以延迟节点分裂
- **坑**: 过早优化是万恶之源，先确保正确性

---

### Week 11: Freelist 管理

**Milestone**: 页空间复用  
**Git Commit**: `feat(storage): implement freelist for page reuse`  
**领域实体**: `Freelist`  
**核心 Struct**: 
- `rdb-storage/src/freelist.rs`: `Freelist`

**测试数量**: 6+ 个单元测试，1 个 proptest

**要完成的工作**:

- [ ] T087 实现 `Freelist` 结构在 `rdb-storage/src/freelist.rs`: 空闲页列表（free_pages, trunk_page）
- [ ] T088 实现 `Freelist::new()` 在 `rdb-storage/src/freelist.rs`: 创建新 Freelist
- [ ] T089 实现 `Freelist::load()` 在 `rdb-storage/src/freelist.rs`: 从页加载 Freelist
- [ ] T090 实现 `Freelist::allocate()` 在 `rdb-storage/src/freelist.rs`: 分配空闲页
- [ ] T091 实现 `Freelist::free()` 在 `rdb-storage/src/freelist.rs`: 释放页到 Freelist
- [ ] T092 实现 `Freelist::save()` 在 `rdb-storage/src/freelist.rs`: 保存 Freelist 到页
- [ ] T093 集成 Freelist 到 Pager 在 `rdb-storage/src/pager.rs`: `allocate_page()` 优先从 Freelist 分配
- [ ] T094 [P] 添加单元测试在 `rdb-storage/tests/freelist_tests.rs`: 分配、释放、持久化测试
- [ ] T095 添加 proptest 在 `rdb-storage/tests/proptest_freelist.rs`: Freelist 无重复页测试
- [ ] T096 验证：删除数据后文件大小不继续增长

**备注**: 
- Freelist 使用链表结构，trunk page 存储指向其他空闲页的指针
- 每个 trunk page 可以存储数百个空闲页 ID
- **坑**: Freelist 本身也占用页，需要递归处理

---

### Week 12: B+Tree 集成测试

**Milestone**: B+Tree 功能完整  
**Git Commit**: `test(storage): comprehensive btree integration tests`  
**领域实体**: 无（集成测试）  
**核心 Struct**: 无

**测试数量**: 20+ 个集成测试，性能基准测试

**要完成的工作**:

- [ ] T097 [P] 添加端到端测试在 `rdb-storage/tests/integration/btree_e2e.rs`: 完整的 CRUD 场景测试
- [ ] T098 [P] 添加边界测试在 `rdb-storage/tests/integration/btree_edge_cases.rs`: 空树、单元素、溢出等
- [ ] T099 [P] 添加崩溃测试在 `rdb-storage/tests/integration/btree_crash.rs`: 模拟崩溃和恢复
- [ ] T100 运行所有 proptest，确保通过：B+Tree 有序性、平衡性、并发安全
- [ ] T101 运行性能基准测试，确保达标：插入 > 10,000/s，查询 < 1ms
- [ ] T102 代码审查：确保所有 unsafe 块都有 SAFETY 注释

**备注**: 
- 集成测试覆盖所有功能组合
- 性能基准测试与 SQLite 对比
- **坑**: 测试要覆盖各种异常情况（磁盘满、损坏数据等）

---

## 阶段 2: WAL 与事务 (Week 13-20)

### Week 13: WAL 文件格式

**Milestone**: WAL 写入实现  
**Git Commit**: `feat(storage): implement WAL file format and writer`  
**领域实体**: `WalWriter`, `WalFrame`  
**核心 Struct**: 
- `rdb-storage/src/wal.rs`: `WalWriter`, `WalHeader`, `WalFrame`

**测试数量**: 8+ 个单元测试

**要完成的工作**:

- [ ] T103 实现 `WalHeader` 结构在 `rdb-storage/src/wal.rs`: WAL 文件头（magic, version, page_size, checkpoint_seq, salt, checksum）
- [ ] T104 实现 `WalFrame` 结构在 `rdb-storage/src/wal.rs`: WAL 帧（page_id, db_size, salt, checksum, reserved, page_data）
- [ ] T105 实现 checksum 算法在 `rdb-storage/src/checksum.rs`: `wal_checksum()` 函数（SQLite 兼容）
- [ ] T106 实现 `WalWriter` 结构在 `rdb-storage/src/wal.rs`: WAL 写入器（file, current_offset, salt, checksum_state）
- [ ] T107 实现 `WalWriter::create()` 在 `rdb-storage/src/wal.rs`: 创建新 WAL 文件
- [ ] T108 实现 `WalWriter::open()` 在 `rdb-storage/src/wal.rs`: 打开已有 WAL 文件
- [ ] T109 实现 `WalWriter::append_frame()` 在 `rdb-storage/src/wal.rs`: 追加 WAL 帧
- [ ] T110 实现 `WalWriter::mark_commit()` 在 `rdb-storage/src/wal.rs`: 标记提交点（更新 db_size）
- [ ] T111 实现 `WalWriter::sync()` 在 `rdb-storage/src/wal.rs`: 刷新到磁盘（fsync）
- [ ] T112 [P] 添加单元测试在 `rdb-storage/tests/wal_writer_tests.rs`: 写入 1000 个 frame，验证 checksum

**备注**: 
- WAL frame 固定大小：32 字节头 + 4096 字节页数据 = 4128 字节
- checksum 累积计算，每个 frame 依赖前一个 frame
- **坑**: checksum 算法必须与 SQLite 完全一致

---

### Week 14: WAL 读取与恢复

**Milestone**: 崩溃恢复  
**Git Commit**: `feat(storage): implement WAL reader and crash recovery`  
**领域实体**: `WalReader`  
**核心 Struct**: 
- `rdb-storage/src/wal_reader.rs`: `WalReader`, `WalFrameIterator`

**测试数量**: 8+ 个单元测试，2 个崩溃恢复测试

**要完成的工作**:

- [ ] T113 实现 `WalReader` 结构在 `rdb-storage/src/wal_reader.rs`: WAL 读取器（file, header）
- [ ] T114 实现 `WalReader::open()` 在 `rdb-storage/src/wal_reader.rs`: 打开 WAL 文件并解析头部
- [ ] T115 实现 `WalFrameIterator` 在 `rdb-storage/src/wal_reader.rs`: WAL 帧迭代器
- [ ] T116 实现 `WalReader::frames()` 在 `rdb-storage/src/wal_reader.rs`: 返回帧迭代器
- [ ] T117 实现 `WalReader::recover()` 在 `rdb-storage/src/wal_reader.rs`: 恢复数据库（重放 WAL）
- [ ] T118 集成恢复逻辑到 Pager 在 `rdb-storage/src/pager.rs`: 打开数据库时检查 WAL 并恢复
- [ ] T119 [P] 添加单元测试在 `rdb-storage/tests/wal_reader_tests.rs`: 读取 WAL，迭代所有帧
- [ ] T120 添加崩溃恢复测试在 `rdb-storage/tests/integration/crash_recovery.rs`: 模拟崩溃，验证数据恢复
- [ ] T121 添加 proptest 在 `rdb-storage/tests/proptest_wal.rs`: WAL 恢复后数据一致性测试

**备注**: 
- 恢复时需要验证每个 frame 的 checksum
- 只恢复到最后一个完整的提交点（db_size > 0 的 frame）
- **坑**: 部分写入的 frame 需要被忽略

---

### Week 15: Checkpoint 机制

**Milestone**: WAL 同步到主文件  
**Git Commit**: `feat(storage): implement WAL checkpoint mechanism`  
**领域实体**: 无（扩展 Pager 和 WalWriter）  
**核心 Struct**: 无

**测试数量**: 6+ 个单元测试

**要完成的工作**:

- [ ] T122 实现 `Pager::checkpoint()` 方法在 `rdb-storage/src/pager.rs`: 执行 checkpoint
- [ ] T123 实现 checkpoint 流程在 `rdb-storage/src/pager.rs`: 读取 WAL frames → 写入主文件 → 截断 WAL
- [ ] T124 实现 `WalWriter::truncate()` 在 `rdb-storage/src/wal.rs`: 截断 WAL 文件
- [ ] T125 实现自动 checkpoint 触发在 `rdb-storage/src/wal.rs`: WAL 达到阈值时自动触发
- [ ] T126 实现 WAL header 更新在 `rdb-storage/src/wal.rs`: 更新 salt 和 checkpoint_seq
- [ ] T127 [P] 添加单元测试在 `rdb-storage/tests/checkpoint_tests.rs`: Checkpoint 后 WAL 文件清空
- [ ] T128 添加性能测试：Checkpoint 操作完成时间 < 100ms（1MB WAL）

**备注**: 
- Checkpoint 需要获取写锁，阻止并发写入
- Checkpoint 过程中需要 fsync 主数据库文件
- **坑**: Checkpoint 失败时需要保证数据一致性

---

### Week 16: 事务 BEGIN/COMMIT

**Milestone**: 事务生命周期管理  
**Git Commit**: `feat(domain): implement transaction lifecycle management`  
**领域实体**: `Transaction`  
**核心 Struct**: 
- `rdb-domain/src/transaction.rs`: `Transaction<'tx>`, `TransactionMode`, `IsolationLevel`

**测试数量**: 8+ 个单元测试

**要完成的工作**:

- [ ] T129 实现 `TransactionMode` 枚举在 `rdb-domain/src/transaction.rs`: `ReadOnly`, `ReadWrite`
- [ ] T130 实现 `IsolationLevel` 枚举在 `rdb-domain/src/transaction.rs`: `ReadCommitted`, `RepeatableRead`, `Serializable`
- [ ] T131 实现 `Transaction<'tx>` 结构在 `rdb-domain/src/transaction.rs`: 事务实体（id, mode, isolation_level, wal_start_offset, snapshot_version, locks）
- [ ] T132 实现 `Transaction::begin()` 在 `rdb-domain/src/transaction.rs`: 开始事务
- [ ] T133 实现 `Transaction::commit()` 在 `rdb-domain/src/transaction.rs`: 提交事务
- [ ] T134 集成 WAL 提交在 `rdb-domain/src/transaction.rs`: COMMIT 时标记 WAL 提交点
- [ ] T135 实现 `TransactionManager` 服务在 `rdb-domain/src/transaction_manager.rs`: 事务管理器（next_txn_id, active_transactions, lock_manager）
- [ ] T136 [P] 添加单元测试在 `rdb-domain/tests/transaction_tests.rs`: BEGIN, COMMIT 流程测试
- [ ] T137 验证：COMMIT 后数据持久化到 WAL

**备注**: 
- Transaction 生命周期绑定到 Database
- 事务 ID 单调递增（AtomicU64）
- **坑**: 确保事务结束时释放所有锁

---

### Week 17: 事务 ROLLBACK

**Milestone**: 事务回滚  
**Git Commit**: `feat(domain): implement transaction rollback`  
**领域实体**: 无（扩展 Transaction）  
**核心 Struct**: 无

**测试数量**: 6+ 个单元测试，1 个 proptest

**要完成的工作**:

- [ ] T138 实现 `Transaction::rollback()` 在 `rdb-domain/src/transaction.rs`: 回滚事务
- [ ] T139 实现回滚逻辑在 `rdb-storage/src/pager.rs`: 丢弃 wal_start_offset 之后的 WAL frames
- [ ] T140 实现 WAL frame 丢弃在 `rdb-storage/src/wal.rs`: 截断到指定偏移量
- [ ] T141 实现脏页清理在 `rdb-infrastructure/src/buffer_pool.rs`: 回滚时清除脏页缓存
- [ ] T142 [P] 添加单元测试在 `rdb-domain/tests/transaction_tests.rs`: ROLLBACK 后数据恢复到事务前状态
- [ ] T143 添加 proptest 在 `rdb-domain/tests/proptest_transaction.rs`: 随机 COMMIT/ROLLBACK 保持一致性

**备注**: 
- ROLLBACK 不需要 fsync，只需丢弃未提交的 WAL frames
- 缓存中的脏页需要标记为无效
- **坑**: 确保 ROLLBACK 后数据库状态完全恢复

---

### Week 18: 写锁机制

**Milestone**: 单写多读并发模型  
**Git Commit**: `feat(infrastructure): implement write lock mechanism`  
**领域实体**: `LockManager`  
**核心 Struct**: 
- `rdb-infrastructure/src/lock_manager.rs`: `LockManager`, `LockId`

**测试数量**: 6+ 个并发测试

**要完成的工作**:

- [ ] T144 实现 `LockManager` 结构在 `rdb-infrastructure/src/lock_manager.rs`: 锁管理器（write_lock, read_count）
- [ ] T145 实现 `LockManager::acquire_read_lock()` 在 `rdb-infrastructure/src/lock_manager.rs`: 获取读锁
- [ ] T146 实现 `LockManager::acquire_write_lock()` 在 `rdb-infrastructure/src/lock_manager.rs`: 获取写锁（互斥）
- [ ] T147 实现 `LockManager::release_read_lock()` 在 `rdb-infrastructure/src/lock_manager.rs`: 释放读锁
- [ ] T148 实现 `LockManager::release_write_lock()` 在 `rdb-infrastructure/src/lock_manager.rs`: 释放写锁
- [ ] T149 集成锁到 Transaction 在 `rdb-domain/src/transaction.rs`: BEGIN 时获取锁，COMMIT/ROLLBACK 时释放锁
- [ ] T150 [P] 添加并发测试在 `rdb-infrastructure/tests/lock_manager_tests.rs`: 两个写事务不能并发
- [ ] T151 验证：读事务不阻塞

**备注**: 
- 使用 `parking_lot::Mutex` 实现写锁
- 使用 `AtomicU32` 计数读锁
- **坑**: 锁超时机制需要考虑

---

### Week 19: MVCC 接口预留

**Milestone**: 为未来 MVCC 预留钩子  
**Git Commit**: `feat(storage): reserve MVCC interfaces for future snapshot reads`  
**领域实体**: 无（添加预留字段）  
**核心 Struct**: 无

**测试数量**: 3+ 个单元测试（验证接口存在）

**要完成的工作**:

- [ ] T152 在 `PageHeader` 添加 `lsn` 字段在 `rdb-storage/src/page.rs`: 8 字节 LSN（日志序列号）
- [ ] T153 在 `WalFrame` 添加 `reserved` 字段在 `rdb-storage/src/wal.rs`: 8 字节预留用于 txn_id
- [ ] T154 在 `Transaction` 添加 `snapshot_version` 字段在 `rdb-domain/src/transaction.rs`: Option<TransactionId>
- [ ] T155 在 `Row` 添加 MVCC 字段在 `rdb-domain/src/row.rs`: `created_txn_id`, `deleted_txn_id`
- [ ] T156 定义快照读接口在 `rdb-domain/src/transaction.rs`: `Transaction::with_snapshot()` 方法签名（未实现）
- [ ] T157 [P] 添加文档注释：标注所有 MVCC 预留字段的用途
- [ ] T158 验证：编译通过，接口文档完整

**备注**: 
- MVCC 接口暂不实现，仅预留数据结构
- 文档注释说明这些字段将在 v2.0 使用
- **坑**: 确保字段大小足够（LSN 和 txn_id 都是 64 位）

---

### Week 20: 事务集成测试

**Milestone**: ACID 属性验证  
**Git Commit**: `test(domain): comprehensive transaction ACID tests`  
**领域实体**: 无（集成测试）  
**核心 Struct**: 无

**测试数量**: 20+ 个集成测试，ACID 验证测试

**要完成的工作**:

- [ ] T159 [P] 添加原子性测试在 `tests/e2e_transactions.rs`: 崩溃恢复，验证所有已提交事务存在
- [ ] T160 [P] 添加一致性测试在 `tests/e2e_transactions.rs`: 约束违反时事务失败
- [ ] T161 [P] 添加隔离性测试在 `tests/e2e_transactions.rs`: 脏读测试（应该失败）
- [ ] T162 [P] 添加持久性测试在 `tests/e2e_transactions.rs`: 断电模拟（kill -9），验证数据恢复
- [ ] T163 运行所有 ACID 测试，确保通过
- [ ] T164 性能测试：事务 COMMIT 延迟 < 10ms

**备注**: 
- ACID 测试是数据库正确性的核心验证
- 崩溃恢复测试使用子进程模拟
- **坑**: 断电模拟需要 kill -9 而非 SIGTERM

---

## 阶段 3: SQL 解析与执行 (Week 21-32)

### Week 21: SQL 解析器集成

**Milestone**: sqlparser-rs 集成  
**Git Commit**: `feat(sql): integrate sqlparser-rs for SQL parsing`  
**领域实体**: `SqlParser`  
**核心 Struct**: 
- `rdb-sql/src/parser.rs`: `SqlParser`, `SqlStatement`

**测试数量**: 10+ 个单元测试

**要完成的工作**:

- [ ] T165 添加 sqlparser 依赖在 `rdb-sql/Cargo.toml`: `sqlparser = "0.47"`
- [ ] T166 实现 `SqlParser` 结构在 `rdb-sql/src/parser.rs`: SQL 解析器（dialect: SQLiteDialect）
- [ ] T167 实现 `SqlStatement` 枚举在 `rdb-sql/src/parser.rs`: AST 定义（CreateTable, Insert, Select, Update, Delete 等）
- [ ] T168 实现 `SqlParser::parse()` 在 `rdb-sql/src/parser.rs`: 解析单个 SQL 语句
- [ ] T169 实现 `SqlParser::parse_multi()` 在 `rdb-sql/src/parser.rs`: 解析多个 SQL 语句（分号分隔）
- [ ] T170 实现 AST 转换在 `rdb-sql/src/parser.rs`: sqlparser-rs AST → rdb SqlStatement
- [ ] T171 实现 `ColumnDef` 结构在 `rdb-sql/src/parser.rs`: 列定义解析
- [ ] T172 实现 `Expr` 枚举在 `rdb-sql/src/parser.rs`: 表达式 AST
- [ ] T173 [P] 添加单元测试在 `rdb-sql/tests/parser_tests.rs`: 解析 50 个 SQL 语句成功
- [ ] T174 验证：支持 CREATE TABLE, INSERT, SELECT, UPDATE, DELETE, BEGIN, COMMIT, ROLLBACK

**备注**: 
- sqlparser-rs 返回的 AST 需要转换为 rdb 内部类型
- SQLiteDialect 支持 SQLite 特有语法
- **坑**: sqlparser-rs 的 AST 类型复杂，仔细处理各种情况

---

### Week 22: CREATE TABLE 实现

**Milestone**: DDL 支持  
**Git Commit**: `feat(sql): implement CREATE TABLE execution`  
**领域实体**: 无（使用已有 Table 实体）  
**核心 Struct**: 
- `rdb-sql/src/executor/ddl.rs`: `CreateTableExecutor`

**测试数量**: 8+ 个单元测试

**要完成的工作**:

- [ ] T175 实现 `CreateTableExecutor` 在 `rdb-sql/src/executor/ddl.rs`: CREATE TABLE 执行器
- [ ] T176 实现表创建逻辑在 `rdb-sql/src/executor/ddl.rs`: 解析列定义 → 创建 Table 对象 → 调用 `Database::add_table()`
- [ ] T177 实现主键检测在 `rdb-sql/src/executor/ddl.rs`: 识别 PRIMARY KEY 列
- [ ] T178 实现约束解析在 `rdb-sql/src/executor/ddl.rs`: NOT NULL, UNIQUE, DEFAULT 等
- [ ] T179 实现系统表更新在 `rdb-domain/src/system_tables.rs`: 将表定义写入 `sqlite_master`
- [ ] T180 实现 B+Tree 根页分配在 `rdb-sql/src/executor/ddl.rs`: 为新表分配根页
- [ ] T181 [P] 添加单元测试在 `rdb-sql/tests/ddl_tests.rs`: 创建表后验证表定义
- [ ] T182 集成测试：创建表后重启，表定义仍存在

**备注**: 
- `sqlite_master` 是系统表，存储所有表和索引的元数据
- 表创建需要在事务中执行
- **坑**: 表名冲突检查

---

### Week 23: INSERT 实现

**Milestone**: 数据插入  
**Git Commit**: `feat(sql): implement INSERT execution`  
**领域实体**: 无（使用已有 Row 实体）  
**核心 Struct**: 
- `rdb-sql/src/executor/insert.rs`: `InsertExecutor`

**测试数量**: 10+ 个单元测试

**要完成的工作**:

- [ ] T183 实现 `InsertExecutor` 在 `rdb-sql/src/executor/insert.rs`: INSERT 执行器
- [ ] T184 实现行数据构造在 `rdb-sql/src/executor/insert.rs`: 解析 VALUES → 创建 Row 对象
- [ ] T185 实现类型转换在 `rdb-sql/src/executor/insert.rs`: Literal → Value 转换
- [ ] T186 实现约束检查在 `rdb-sql/src/executor/insert.rs`: NOT NULL, UNIQUE 验证
- [ ] T187 实现自动递增在 `rdb-sql/src/executor/insert.rs`: AUTOINCREMENT PRIMARY KEY 处理
- [ ] T188 实现行序列化在 `rdb-domain/src/row.rs`: `Row::serialize()` 方法（SQLite Record Format）
- [ ] T189 实现 B+Tree 插入在 `rdb-sql/src/executor/insert.rs`: 调用 `BTree::insert()` 写入行
- [ ] T190 [P] 添加单元测试在 `rdb-sql/tests/insert_tests.rs`: 插入 10000 行数据
- [ ] T191 验证：插入数据后能通过 B+Tree 查询到

**备注**: 
- 行序列化使用 varint 编码节省空间
- AUTOINCREMENT 需要维护一个全局计数器
- **坑**: 类型转换时需要处理 NULL 和类型不匹配

---

### Week 24: SELECT 基础查询

**Milestone**: 全表扫描查询  
**Git Commit**: `feat(sql): implement SELECT with full table scan`  
**领域实体**: `Executor` trait, `SeqScanExecutor`  
**核心 Struct**: 
- `rdb-sql/src/executor/mod.rs`: `Executor` trait
- `rdb-sql/src/executor/scan.rs`: `SeqScanExecutor`

**测试数量**: 10+ 个单元测试

**要完成的工作**:

- [ ] T192 实现 `Executor` trait 在 `rdb-sql/src/executor/mod.rs`: 查询执行器 trait（Iterator + explain + columns）
- [ ] T193 实现 `SeqScanExecutor` 在 `rdb-sql/src/executor/scan.rs`: 全表扫描执行器
- [ ] T194 实现 `FilterExecutor` 在 `rdb-sql/src/executor/filter.rs`: WHERE 过滤执行器
- [ ] T195 实现 `ProjectExecutor` 在 `rdb-sql/src/executor/project.rs`: SELECT 列投影执行器
- [ ] T196 实现 `ExprEvaluator` 在 `rdb-sql/src/evaluator.rs`: 表达式求值器
- [ ] T197 实现行反序列化在 `rdb-domain/src/row.rs`: `Row::deserialize()` 方法
- [ ] T198 实现查询计划生成在 `rdb-sql/src/planner/mod.rs`: LogicalPlan → PhysicalPlan
- [ ] T199 [P] 添加单元测试在 `rdb-sql/tests/select_tests.rs`: `SELECT * FROM table WHERE id > 100` 返回正确结果
- [ ] T200 验证：查询结果使用迭代器流式返回

**备注**: 
- Executor 使用迭代器模式，避免内存爆炸
- WHERE 过滤在投影之前执行
- **坑**: 表达式求值需要处理 NULL 传播

---

### Week 25: SELECT 聚合函数

**Milestone**: COUNT, SUM, AVG, MIN, MAX  
**Git Commit**: `feat(sql): implement aggregate functions`  
**领域实体**: `AggregateExecutor`  
**核心 Struct**: 
- `rdb-sql/src/executor/aggregate.rs`: `AggregateExecutor`, `AggregateFunction`

**测试数量**: 10+ 个单元测试

**要完成的工作**:

- [ ] T201 实现 `AggregateFunction` 枚举在 `rdb-sql/src/executor/aggregate.rs`: Count, Sum, Avg, Min, Max
- [ ] T202 实现 `AggregateExecutor` 在 `rdb-sql/src/executor/aggregate.rs`: 聚合执行器
- [ ] T203 实现 COUNT 函数在 `rdb-sql/src/executor/aggregate.rs`: 计数
- [ ] T204 实现 SUM/AVG 函数在 `rdb-sql/src/executor/aggregate.rs`: 求和和平均
- [ ] T205 实现 MIN/MAX 函数在 `rdb-sql/src/executor/aggregate.rs`: 最小最大值
- [ ] T206 实现 GROUP BY 在 `rdb-sql/src/executor/aggregate.rs`: 分组聚合（Hash 聚合）
- [ ] T207 [P] 添加单元测试在 `rdb-sql/tests/aggregate_tests.rs`: `SELECT COUNT(*) FROM table` 返回正确行数
- [ ] T208 验证：聚合函数与 GROUP BY 组合使用

**备注**: 
- 聚合函数需要扫描所有行，内存占用需要控制
- GROUP BY 使用 HashMap 存储分组结果
- **坑**: AVG 需要同时计算 SUM 和 COUNT

---

### Week 26: UPDATE 与 DELETE

**Milestone**: 数据修改  
**Git Commit**: `feat(sql): implement UPDATE and DELETE execution`  
**领域实体**: `UpdateExecutor`, `DeleteExecutor`  
**核心 Struct**: 
- `rdb-sql/src/executor/update.rs`: `UpdateExecutor`
- `rdb-sql/src/executor/delete.rs`: `DeleteExecutor`

**测试数量**: 10+ 个单元测试

**要完成的工作**:

- [ ] T209 实现 `UpdateExecutor` 在 `rdb-sql/src/executor/update.rs`: UPDATE 执行器
- [ ] T210 实现 UPDATE 逻辑在 `rdb-sql/src/executor/update.rs`: 扫描表 → 应用 SET 子句 → 更新 B+Tree
- [ ] T211 实现 `DeleteExecutor` 在 `rdb-sql/src/executor/delete.rs`: DELETE 执行器
- [ ] T212 实现 DELETE 逻辑在 `rdb-sql/src/executor/delete.rs`: 扫描表 → 从 B+Tree 删除行
- [ ] T213 实现索引更新在 `rdb-sql/src/executor/update.rs`: UPDATE/DELETE 时更新所有索引
- [ ] T214 [P] 添加单元测试在 `rdb-sql/tests/update_tests.rs`: UPDATE 后查询返回新值
- [ ] T215 [P] 添加单元测试在 `rdb-sql/tests/delete_tests.rs`: DELETE 后数据不可见
- [ ] T216 验证：UPDATE/DELETE 影响行数正确

**备注**: 
- UPDATE 可能改变行的主键，需要先删除再插入
- DELETE 后的页需要加入 Freelist
- **坑**: 索引更新可能失败，需要回滚

---

### Week 27: CREATE INDEX

**Milestone**: 索引创建  
**Git Commit**: `feat(sql): implement CREATE INDEX execution`  
**领域实体**: `Index`  
**核心 Struct**: 
- `rdb-sql/src/executor/ddl.rs`: `CreateIndexExecutor`

**测试数量**: 8+ 个单元测试

**要完成的工作**:

- [ ] T217 实现 `CreateIndexExecutor` 在 `rdb-sql/src/executor/ddl.rs`: CREATE INDEX 执行器
- [ ] T218 实现索引构建在 `rdb-sql/src/executor/ddl.rs`: 扫描表 → 提取索引键 → 插入索引 B+Tree
- [ ] T219 实现索引元数据存储在 `rdb-domain/src/system_tables.rs`: 将索引定义写入 `sqlite_master`
- [ ] T220 实现复合索引在 `rdb-sql/src/executor/ddl.rs`: 支持多列索引
- [ ] T221 实现 UNIQUE 索引在 `rdb-sql/src/executor/ddl.rs`: 检查唯一性约束
- [ ] T222 [P] 添加单元测试在 `rdb-sql/tests/index_tests.rs`: 创建索引后重启，索引仍存在
- [ ] T223 验证：大表（10万行）索引构建时间 < 10 秒

**备注**: 
- 索引构建需要在事务中执行
- 索引 B+Tree 的键是索引列的组合，值是 RowId
- **坑**: 构建索引时内存占用可能很大

---

### Week 28: 索引查询优化

**Milestone**: 使用索引加速查询  
**Git Commit**: `feat(sql): implement index scan and query optimization`  
**领域实体**: `QueryOptimizer`, `IndexScanExecutor`  
**核心 Struct**: 
- `rdb-sql/src/planner/optimizer.rs`: `QueryOptimizer`
- `rdb-sql/src/executor/index_scan.rs`: `IndexScanExecutor`

**测试数量**: 10+ 个单元测试

**要完成的工作**:

- [ ] T224 实现 `QueryOptimizer` 在 `rdb-sql/src/planner/optimizer.rs`: 查询优化器
- [ ] T225 实现索引选择在 `rdb-sql/src/planner/optimizer.rs`: `select_index()` 方法
- [ ] T226 实现 `IndexScanExecutor` 在 `rdb-sql/src/executor/index_scan.rs`: 索引扫描执行器
- [ ] T227 实现谓词下推在 `rdb-sql/src/planner/optimizer.rs`: Predicate Pushdown 优化
- [ ] T228 实现投影下推在 `rdb-sql/src/planner/optimizer.rs`: Projection Pushdown 优化
- [ ] T229 实现 EXPLAIN 在 `rdb-sql/src/executor/mod.rs`: 显示查询计划
- [ ] T230 [P] 添加单元测试在 `rdb-sql/tests/optimizer_tests.rs`: `SELECT * FROM table WHERE name = 'Alice'` 使用索引
- [ ] T231 验证：索引查询性能提升 > 10x vs 全表扫描

**备注**: 
- 索引选择基于 WHERE 条件匹配度
- EXPLAIN 显示查询计划树
- **坑**: 复合索引的前缀匹配规则

---

### Week 29: ORDER BY 与 LIMIT

**Milestone**: 排序和分页  
**Git Commit**: `feat(sql): implement ORDER BY and LIMIT`  
**领域实体**: `SortExecutor`, `LimitExecutor`  
**核心 Struct**: 
- `rdb-sql/src/executor/sort.rs`: `SortExecutor`
- `rdb-sql/src/executor/limit.rs`: `LimitExecutor`

**测试数量**: 8+ 个单元测试

**要完成的工作**:

- [ ] T232 实现 `SortExecutor` 在 `rdb-sql/src/executor/sort.rs`: 排序执行器（内存排序）
- [ ] T233 实现排序键比较在 `rdb-sql/src/executor/sort.rs`: 多列排序、ASC/DESC、NULLS FIRST/LAST
- [ ] T234 实现 `LimitExecutor` 在 `rdb-sql/src/executor/limit.rs`: LIMIT 和 OFFSET 执行器
- [ ] T235 优化：使用索引避免排序在 `rdb-sql/src/planner/optimizer.rs`: ORDER BY 列有索引时跳过排序
- [ ] T236 [P] 添加单元测试在 `rdb-sql/tests/sort_tests.rs`: `SELECT * FROM table ORDER BY id LIMIT 10` 返回前 10 行
- [ ] T237 验证：LIMIT 查询不加载所有行

**备注**: 
- 排序需要先收集所有行，内存占用大
- LIMIT 可以提前终止查询
- **坑**: OFFSET 需要跳过行，性能较低

---

### Week 30: JOIN 实现（嵌套循环）

**Milestone**: 多表关联  
**Git Commit**: `feat(sql): implement nested loop join`  
**领域实体**: `NestedLoopJoinExecutor`  
**核心 Struct**: 
- `rdb-sql/src/executor/join.rs`: `NestedLoopJoinExecutor`

**测试数量**: 10+ 个单元测试

**要完成的工作**:

- [ ] T238 实现 `JoinType` 枚举在 `rdb-sql/src/executor/join.rs`: Inner, Left, Right, Full, Cross
- [ ] T239 实现 `NestedLoopJoinExecutor` 在 `rdb-sql/src/executor/join.rs`: 嵌套循环 JOIN
- [ ] T240 实现 INNER JOIN 在 `rdb-sql/src/executor/join.rs`: 内连接逻辑
- [ ] T241 实现 LEFT JOIN 在 `rdb-sql/src/executor/join.rs`: 左连接逻辑
- [ ] T242 实现 JOIN 条件评估在 `rdb-sql/src/executor/join.rs`: ON 子句求值
- [ ] T243 实现行拼接在 `rdb-sql/src/executor/join.rs`: 合并左右表列
- [ ] T244 [P] 添加单元测试在 `rdb-sql/tests/join_tests.rs`: `SELECT * FROM users JOIN orders ON users.id = orders.user_id` 返回正确结果
- [ ] T245 验证：多表 JOIN 正确性

**备注**: 
- 嵌套循环 JOIN 简单但性能较低
- 小表应该作为外层循环
- **坑**: NULL 值在 JOIN 中的处理

---

### Week 31: JOIN 优化（Hash Join）

**Milestone**: 性能优化  
**Git Commit**: `feat(sql): implement hash join optimization`  
**领域实体**: `HashJoinExecutor`  
**核心 Struct**: 
- `rdb-sql/src/executor/join.rs`: `HashJoinExecutor`

**测试数量**: 6+ 个单元测试，性能基准测试

**要完成的工作**:

- [ ] T246 实现 `HashJoinExecutor` 在 `rdb-sql/src/executor/join.rs`: Hash JOIN 执行器
- [ ] T247 实现 Build 阶段在 `rdb-sql/src/executor/join.rs`: 构建 Hash 表（从右表）
- [ ] T248 实现 Probe 阶段在 `rdb-sql/src/executor/join.rs`: 探测 Hash 表（从左表）
- [ ] T249 实现 JOIN 策略选择在 `rdb-sql/src/planner/optimizer.rs`: 小表用 Hash Join，大表用 Nested Loop
- [ ] T250 [P] 添加单元测试在 `rdb-sql/tests/join_tests.rs`: Hash Join 正确性测试
- [ ] T251 添加性能测试：大表 JOIN 性能对比（Hash vs Nested Loop）

**备注**: 
- Hash Join 适合等值 JOIN
- Build 阶段需要扫描整个右表
- **坑**: Hash 表内存占用可能很大

---

### Week 32: SQL 集成测试

**Milestone**: SQL 功能完整  
**Git Commit**: `test(sql): comprehensive SQL integration tests`  
**领域实体**: 无（集成测试）  
**核心 Struct**: 无

**测试数量**: 50+ 个集成测试，兼容性测试

**要完成的工作**:

- [ ] T252 [P] 添加端到端测试在 `tests/e2e_sql.rs`: 完整的 SQL 场景测试
- [ ] T253 [P] 添加 TPC-C 风格测试在 `tests/tpcc_tests.rs`: 模拟真实工作负载
- [ ] T254 [P] 添加兼容性测试在 `tests/sqlite_compat_tests.rs`: 与 SQLite 对比测试
- [ ] T255 运行所有 SQL 测试，确保通过
- [ ] T256 性能基准测试：核心 SQL 操作性能达标

**备注**: 
- 集成测试覆盖所有 SQL 功能组合
- 兼容性测试确保与 SQLite 行为一致
- **坑**: SQL 标准复杂，边界情况多

---

## 阶段 4: 接口层与 API (Week 33-36)

### Week 33: 公共 API 设计

**Milestone**: rdb-interface 定义  
**Git Commit**: `feat(interface): implement public Rust API`  
**领域实体**: `Database`, `Connection`, `Statement` (公共类型)  
**核心 Struct**: 
- `rdb-interface/src/database.rs`: `Database`
- `rdb-interface/src/connection.rs`: `Connection`
- `rdb-interface/src/statement.rs`: `Statement`

**测试数量**: 15+ 个 API 测试

**要完成的工作**:

- [ ] T257 实现 `Database` 公共类型在 `rdb-interface/src/database.rs`: 数据库句柄（内部持有 Arc<DatabaseInner>）
- [ ] T258 实现 `Database::open()` 在 `rdb-interface/src/database.rs`: 打开或创建数据库
- [ ] T259 实现 `Database::open_with_options()` 在 `rdb-interface/src/database.rs`: 带选项打开数据库
- [ ] T260 实现 `Database::execute()` 在 `rdb-interface/src/database.rs`: 执行 SQL 语句
- [ ] T261 实现 `Database::query()` 在 `rdb-interface/src/database.rs`: 查询数据
- [ ] T262 实现 `Database::prepare()` 在 `rdb-interface/src/database.rs`: 准备参数化语句
- [ ] T263 实现 `Database::begin_transaction()` 在 `rdb-interface/src/database.rs`: 开始事务
- [ ] T264 实现 `Database::close()` 在 `rdb-interface/src/database.rs`: 关闭数据库
- [ ] T265 实现 `Database::checkpoint()` 在 `rdb-interface/src/database.rs`: 执行 checkpoint
- [ ] T266 实现 `Transaction` 公共类型在 `rdb-interface/src/transaction.rs`: 事务句柄
- [ ] T267 实现 `Statement` 公共类型在 `rdb-interface/src/statement.rs`: 预编译语句
- [ ] T268 [P] 添加 API 测试在 `rdb-interface/tests/api_tests.rs`: 所有公共 API 功能测试
- [ ] T269 [P] 添加示例代码在 `rdb-interface/examples/`: basic_usage, transactions, indexes
- [ ] T270 验证：示例代码能运行

**备注**: 
- 公共 API 必须简单易用
- 所有内部类型不暴露
- **坑**: 线程安全保证（Send + Sync）

---

### Week 34: 结果迭代器

**Milestone**: 流式查询结果  
**Git Commit**: `feat(interface): implement result iterator for streaming query results`  
**领域实体**: `Rows`, `Row`  
**核心 Struct**: 
- `rdb-interface/src/rows.rs`: `Rows<'db>`, `Row`

**测试数量**: 8+ 个单元测试

**要完成的工作**:

- [ ] T271 实现 `Rows<'db>` 迭代器在 `rdb-interface/src/rows.rs`: 查询结果迭代器
- [ ] T272 实现 `Iterator` trait 在 `rdb-interface/src/rows.rs`: 流式返回行
- [ ] T273 实现 `Rows::collect_vec()` 在 `rdb-interface/src/rows.rs`: 收集所有行到 Vec
- [ ] T274 实现 `Row` 公共类型在 `rdb-interface/src/row.rs`: 单行数据
- [ ] T275 实现 `Row::get()` 在 `rdb-interface/src/row.rs`: 获取列值（按索引，泛型类型）
- [ ] T276 实现 `Row::get_by_name()` 在 `rdb-interface/src/row.rs`: 获取列值（按列名）
- [ ] T277 实现 `FromValue` trait 在 `rdb-interface/src/value.rs`: Value 到 Rust 类型的转换
- [ ] T278 [P] 添加单元测试在 `rdb-interface/tests/rows_tests.rs`: 查询 100 万行，内存占用 < 10MB
- [ ] T279 验证：流式迭代器不会一次性加载所有行

**备注**: 
- `Rows` 使用迭代器模式，惰性求值
- `Row::get::<T>()` 自动类型转换
- **坑**: 迭代器生命周期绑定到查询

---

### Week 35: 错误处理

**Milestone**: 友好的错误信息  
**Git Commit**: `feat(interface): implement comprehensive error handling`  
**领域实体**: `RdbError`  
**核心 Struct**: 
- `rdb-interface/src/error.rs`: `RdbError`

**测试数量**: 10+ 个错误场景测试

**要完成的工作**:

- [ ] T280 实现 `RdbError` 枚举在 `rdb-interface/src/error.rs`: Io, SqlSyntax, SqlExecution, ConstraintViolation, Corruption, Transaction, TypeConversion, ColumnNotFound, TableNotFound
- [ ] T281 实现错误转换在 `rdb-interface/src/error.rs`: 内部错误 → RdbError 转换
- [ ] T282 实现错误上下文在 `rdb-interface/src/error.rs`: 添加文件名、行号、SQL 位置等信息
- [ ] T283 实现友好的错误消息在 `rdb-interface/src/error.rs`: 用户可读的错误描述
- [ ] T284 [P] 添加错误测试在 `rdb-interface/tests/error_tests.rs`: 所有错误场景测试
- [ ] T285 验证：错误信息包含有用的调试信息

**备注**: 
- 使用 thiserror 简化错误定义
- 错误信息要对用户友好，不暴露内部实现细节
- **坑**: 错误传播链要正确

---

### Week 36: 线程安全保证

**Milestone**: API 线程安全  
**Git Commit**: `test(interface): verify thread safety guarantees`  
**领域实体**: 无（线程安全测试）  
**核心 Struct**: 无

**测试数量**: 10+ 个并发测试

**要完成的工作**:

- [ ] T286 验证 `Database: Send + Sync` 在 `rdb-interface/tests/thread_safety_tests.rs`: 编译时检查
- [ ] T287 验证 `Transaction: !Send + !Sync` 在 `rdb-interface/tests/thread_safety_tests.rs`: 编译时检查
- [ ] T288 [P] 添加多线程测试在 `rdb-interface/tests/concurrent_tests.rs`: 10 个线程并发读取
- [ ] T289 [P] 添加读写并发测试在 `rdb-interface/tests/concurrent_tests.rs`: 1 个写线程 + 9 个读线程
- [ ] T290 [P] 添加压力测试在 `rdb-interface/tests/stress_tests.rs`: 100 个线程并发操作
- [ ] T291 完善文档注释在 `rdb-interface/src/lib.rs`: 线程模型说明
- [ ] T292 验证：多线程压力测试通过

**备注**: 
- `Database` 使用 `Arc` 在线程间共享
- `Transaction` 必须在创建线程使用
- **坑**: 确保所有公共类型的 Send/Sync 实现正确

---

## 阶段 5: 高级特性 (Week 37-44)

由于篇幅限制，Week 37-52 的详细任务清单将采用简化格式。每周一个 Milestone，包含关键任务和验收标准。

### Week 37: 约束支持（NOT NULL, UNIQUE）
- T293-T300: 实现 NOT NULL, UNIQUE 约束检查
- 验收：违反约束时返回错误

### Week 38: 外键支持（基础）
- T301-T308: 实现 FOREIGN KEY 定义和检查
- 验收：级联删除测试

### Week 39: VIEW 支持
- T309-T315: 实现 CREATE VIEW 和视图查询
- 验收：查询视图等同于查询底层表

### Week 40: 触发器（基础）
- T316-T323: 实现 CREATE TRIGGER 和触发器执行
- 验收：触发器正确执行

### Week 41: 子查询支持
- T324-T330: 实现子查询执行器和 IN/EXISTS
- 验收：子查询正确执行

### Week 42: EXPLAIN 查询计划
- T331-T336: 实现 EXPLAIN 输出和可视化
- 验收：EXPLAIN 输出可读

### Week 43: VACUUM 与碎片整理
- T337-T343: 实现 VACUUM 命令和数据库重建
- 验收：VACUUM 后文件大小缩减

### Week 44: PRAGMA 命令
- T344-T350: 实现 PRAGMA page_size, cache_size 等
- 验收：设置 PRAGMA 影响行为

---

## 阶段 6: 性能优化与测试 (Week 45-48)

### Week 45: 性能基准测试
- T351-T357: criterion benchmark 套件，与 SQLite 对比
- 验收：核心操作性能差距 < 2x SQLite

### Week 46: 内存优化
- T358-T364: 减少拷贝，内存池，arena allocator
- 验收：内存占用减少 30%

### Week 47: 模糊测试（Fuzzing）
- T365-T371: cargo-fuzz 集成，模糊测试套件
- 验收：24 小时模糊测试无崩溃

### Week 48: 压力测试
- T372-T378: 长时间运行测试，并发压力测试
- 验收：无内存泄漏，无数据损坏

---

## 阶段 7: 文档与发布 (Week 49-52)

### Week 49: API 文档完善
- T379-T385: 完整的 Rustdoc，架构文档
- 验收：`cargo doc` 无警告

### Week 50: 用户指南
- T386-T392: 快速开始指南，迁移指南，性能调优指南
- 验收：新用户能在 5 分钟内运行示例

### Week 51: 生态集成
- T393-T399: crates.io 发布准备，CI/CD，CHANGELOG
- 验收：通过 crates.io 审核

### Week 52: v1.0.0 发布
- T400-T405: 发布公告，演示视频，社区反馈
- 验收：下载量 > 1000（第一周）

---

## 依赖关系与执行顺序

### 阶段依赖

- **阶段 0 (Week 1-4)**: 无依赖，可立即开始
- **阶段 1 (Week 5-12)**: 依赖阶段 0 完成
- **阶段 2 (Week 13-20)**: 依赖阶段 1 完成（B+Tree 就绪）
- **阶段 3 (Week 21-32)**: 依赖阶段 1-2 完成（存储引擎和事务就绪）
- **阶段 4 (Week 33-36)**: 依赖阶段 3 完成（SQL 层就绪）
- **阶段 5 (Week 37-44)**: 依赖阶段 4 完成（公共 API 就绪）
- **阶段 6 (Week 45-48)**: 依赖阶段 5 完成（所有功能就绪）
- **阶段 7 (Week 49-52)**: 依赖阶段 6 完成（性能优化完成）

### 并行机会

- Week 1: T002-T007 可并行（创建 6 个 crate）
- Week 2: T015-T016, T023-T024 可并行（不同文件）
- Week 5-12: B+Tree 开发 || BufferPool 优化（不同团队成员）
- Week 21-32: SQL 解析 || 查询优化器（不同团队成员）

### 关键路径

**阻塞路径**（必须串行）：
1. Week 1-4: 基础设施 → 所有后续工作
2. Week 5-12: B+Tree → WAL 和 SQL
3. Week 13-20: WAL → 事务支持
4. Week 21-32: SQL → 公共 API

---

## 实施策略

### MVP 优先（前 20 周）

1. 完成 Week 1-4: 基础设施
2. 完成 Week 5-12: B+Tree 存储引擎
3. 完成 Week 13-20: WAL 与事务
4. **停止并验证**：此时已有完整的存储引擎和事务支持

### 增量交付

1. Week 1-20 → 存储引擎 MVP（可嵌入使用，但无 SQL）
2. Week 21-32 → SQL 支持（完整的 SQL 数据库）
3. Week 33-36 → 公共 API（用户友好的接口）
4. Week 37-52 → 高级特性和优化

### 团队协作策略

**单人开发**：按周顺序执行，每周 1 个 Milestone

**2-3 人团队**：
- 开发者 A: 领域层和 SQL 层
- 开发者 B: 存储层和基础设施层
- 开发者 C: 测试和文档

**并行开发**（Week 5 之后）：
- 不同层可以并行开发（前提是接口定义清晰）
- 使用 mock 对象进行单元测试

---

## 备注

### 通用注意事项

1. **每周提交**：每个 Week 完成后提交代码，Git commit message 遵循 Conventional Commits
2. **测试优先**：实现功能前先写测试（proptest 在实现后添加）
3. **代码审查**：所有涉及 unsafe 的代码必须经过审查
4. **性能测试**：关键路径（Pager, B+Tree, WAL）在实现后立即添加基准测试
5. **文档同步**：公共 API 的文档必须与代码同步更新

### 常见坑

- **生命周期参数**：存储层大量使用生命周期，确保理解 `'static`, `'db`, `'page` 的区别
- **unsafe 代码**：仅限 Pager 和 B+Tree，每个 unsafe 块必须有详细 SAFETY 注释
- **并发测试**：多线程测试容易出现 flaky test，使用 `cargo test -- --test-threads=1` 调试
- **WAL 恢复**：崩溃恢复是最复杂的部分，需要仔细测试各种崩溃场景
- **内存占用**：BufferPool 和查询结果集是内存消耗大户，注意监控

---

**Tasks Version**: 1.0  
**Total Tasks**: 405  
**Estimated Duration**: 52 weeks (12 months)  
**Last Updated**: 2025-12-10

