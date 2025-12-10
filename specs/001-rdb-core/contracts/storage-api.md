# 存储层 API 契约

**Feature**: rdb 嵌入式关系型数据库  
**Layer**: Storage Layer (`rdb-storage` crate)  
**Visibility**: `pub(crate)` - 内部 API，不暴露给外部用户  
**Date**: 2025-12-10

## 概述

本文档定义 rdb 存储层的内部 API。此层负责底层存储操作，是唯一允许使用 `unsafe` 代码的模块。

---

## Pager API

### Pager 类型

**职责**：页管理器，负责页的读取、写入和缓存

```rust
/// 页管理器
/// 
/// 职责：
/// - 从磁盘读取页
/// - 写入页到磁盘
/// - 缓存热页（通过 BufferPool）
/// - 管理页分配
/// 
/// 生命周期: 'db (绑定到数据库文件生命周期)
/// 线程安全: !Send + !Sync (内部使用 RefCell)
/// 
/// UNSAFE: 此类型包含 unsafe 代码
pub(crate) struct Pager<'db> {
    file: File,
    page_size: usize,
    page_count: AtomicU32,
    buffer_pool: Arc<BufferPool>,
    _phantom: PhantomData<&'db mut ()>,
}

impl<'db> Pager<'db> {
    /// 创建新 Pager
    /// 
    /// # Parameters
    /// - `file`: 数据库文件句柄
    /// - `page_size`: 页大小（必须是 512 的倍数，默认 4096）
    /// - `buffer_pool`: 缓存池
    pub(crate) fn new(
        file: File,
        page_size: usize,
        buffer_pool: Arc<BufferPool>,
    ) -> Result<Self>;
    
    /// 读取页（通过缓存）
    /// 
    /// # Parameters
    /// - `page_id`: 页 ID（从 1 开始，0 是保留页）
    /// 
    /// # Returns
    /// - `Ok(&Page)`: 页引用（可能来自缓存）
    /// - `Err`: IO 错误或页不存在
    /// 
    /// # Safety
    /// 调用者必须确保在持有页引用期间不修改 Pager
    pub(crate) fn get_page(&self, page_id: PageId) -> Result<&Page>;
    
    /// 获取可变页引用
    /// 
    /// # Parameters
    /// - `page_id`: 页 ID
    /// 
    /// # Returns
    /// - `Ok(&mut Page)`: 可变页引用
    /// 
    /// # Safety
    /// 调用者必须确保独占访问此页
    pub(crate) fn get_page_mut(&mut self, page_id: PageId) -> Result<&mut Page>;
    
    /// 分配新页
    /// 
    /// # Returns
    /// - `Ok(PageId)`: 新分配的页 ID
    /// 
    /// # Implementation
    /// 1. 检查 Freelist 是否有空闲页
    /// 2. 如果没有，扩展文件并返回新页 ID
    pub(crate) fn allocate_page(&mut self) -> Result<PageId>;
    
    /// 释放页（加入 Freelist）
    pub(crate) fn free_page(&mut self, page_id: PageId) -> Result<()>;
    
    /// 刷新所有脏页到磁盘
    pub(crate) fn flush_all(&mut self) -> Result<()>;
    
    /// 刷新特定页到磁盘
    pub(crate) fn flush_page(&mut self, page_id: PageId) -> Result<()>;
    
    /// 获取总页数
    pub(crate) fn page_count(&self) -> u32;
    
    /// UNSAFE: 获取页的原始指针
    /// 
    /// # Safety
    /// 调用者必须确保：
    /// 1. page_id 有效
    /// 2. 返回的指针在 Pager 生命周期内有效
    /// 3. 不会在持有指针时修改页
    pub(crate) unsafe fn get_page_ptr(&self, page_id: PageId) -> *const Page<'db>;
    
    /// UNSAFE: 获取页的可变原始指针
    /// 
    /// # Safety
    /// 调用者必须确保：
    /// 1. page_id 有效
    /// 2. 独占访问此页
    /// 3. 不会导致数据竞争
    pub(crate) unsafe fn get_page_mut_ptr(&mut self, page_id: PageId) -> *mut Page<'db>;
}
```

---

## Page API

### Page 类型

**职责**：4KB 数据页

```rust
/// 4KB 数据页
/// 
/// 布局：
/// [Header: 32 bytes] [Cell Pointer Array] [Free Space] [Cell Content Area]
/// 
/// 生命周期: 'page (绑定到 Pager)
/// 线程安全: !Send + !Sync (包含原始指针)
#[repr(C, align(4096))]
pub(crate) struct Page<'page> {
    data: [u8; 4096],
    page_id: PageId,
    dirty: AtomicBool,
    pin_count: AtomicU32,
    _phantom: PhantomData<&'page mut ()>,
}

impl<'page> Page<'page> {
    /// 创建新页（初始化为零）
    pub(crate) fn new(page_id: PageId, page_type: PageType) -> Self;
    
    /// 从字节数组加载页
    pub(crate) fn from_bytes(page_id: PageId, data: [u8; 4096]) -> Self;
    
    /// 获取页 ID
    pub(crate) fn page_id(&self) -> PageId;
    
    /// 获取页类型
    pub(crate) fn page_type(&self) -> PageType;
    
    /// 标记为脏页
    pub(crate) fn mark_dirty(&self);
    
    /// 是否为脏页
    pub(crate) fn is_dirty(&self) -> bool;
    
    /// Pin 页（防止被淘汰）
    pub(crate) fn pin(&self);
    
    /// Unpin 页
    pub(crate) fn unpin(&self);
    
    /// 获取 Pin 计数
    pub(crate) fn pin_count(&self) -> u32;
    
    /// 获取页数据的不可变引用
    pub(crate) fn data(&self) -> &[u8; 4096];
    
    /// 获取页数据的可变引用
    pub(crate) fn data_mut(&mut self) -> &mut [u8; 4096];
    
    /// 解析页头
    pub(crate) fn parse_header(&self) -> PageHeader;
    
    /// 写入页头
    pub(crate) fn write_header(&mut self, header: &PageHeader);
}

/// 页类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum PageType {
    /// 内部节点（B+Tree）
    Internal = 0x05,
    /// 叶子节点（B+Tree）
    Leaf = 0x0D,
    /// 溢出页
    Overflow = 0x02,
    /// 空闲列表页
    Freelist = 0x01,
}

/// 页头（32 字节）
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub(crate) struct PageHeader {
    pub page_type: PageType,
    pub first_freeblock: u16,
    pub num_cells: u16,
    pub cell_content_area: u16,
    pub fragmented_bytes: u8,
    pub right_child: u32,  // 仅内部节点
    pub lsn: u64,          // MVCC 预留
    pub checksum: u32,
    pub reserved: u64,     // 集群预留
}
```

---

## B+Tree API

### BTree 类型

**职责**：B+Tree 索引实现

```rust
/// B+Tree 索引
/// 
/// 不变量：
/// - 所有键有序
/// - 内部节点有 [n/2, n] 个子节点
/// - 叶子节点在同一层
/// 
/// 线程安全: !Send + !Sync
pub(crate) struct BTree<'tree, K, V> {
    root_page: PageId,
    pager: &'tree mut Pager<'tree>,
    order: usize,  // 最大键数
    _phantom: PhantomData<(K, V)>,
}

impl<'tree, K: Ord, V> BTree<'tree, K, V> {
    /// 创建新 B+Tree
    pub(crate) fn new(pager: &'tree mut Pager<'tree>) -> Result<Self>;
    
    /// 从已有根页打开 B+Tree
    pub(crate) fn open(root_page: PageId, pager: &'tree mut Pager<'tree>) -> Result<Self>;
    
    /// 插入键值对
    /// 
    /// # Returns
    /// - `Ok(None)`: 插入新键
    /// - `Ok(Some(old_value))`: 更新已有键
    pub(crate) fn insert(&mut self, key: K, value: V) -> Result<Option<V>>;
    
    /// 查找键
    pub(crate) fn get(&self, key: &K) -> Result<Option<V>>;
    
    /// 删除键
    pub(crate) fn remove(&mut self, key: &K) -> Result<Option<V>>;
    
    /// 范围查询（返回游标）
    pub(crate) fn range<R>(&self, range: R) -> BTreeCursor<'tree, K, V>
    where
        R: RangeBounds<K>;
    
    /// 迭代所有键值对
    pub(crate) fn iter(&self) -> BTreeCursor<'tree, K, V>;
}

/// B+Tree 游标（迭代器）
pub(crate) struct BTreeCursor<'tree, K, V> {
    current_page: PageId,
    cell_index: usize,
    pager: &'tree Pager<'tree>,
    _phantom: PhantomData<(K, V)>,
}

impl<'tree, K, V> Iterator for BTreeCursor<'tree, K, V> {
    type Item = Result<(K, V)>;
    
    fn next(&mut self) -> Option<Self::Item>;
}
```

### BTreeNode 类型

**职责**：B+Tree 节点操作（内部使用）

```rust
/// B+Tree 节点
/// 
/// 生命周期: 'node (绑定到 Page)
/// 线程安全: !Send + !Sync
/// 
/// UNSAFE: 包含指向 Page 内部的原始指针
pub(crate) struct BTreeNode<'node, K, V> {
    page: Pin<&'node mut Page<'node>>,
    is_leaf: bool,
    num_keys: u16,
    keys: &'node [K],      // UNSAFE: 指向 page.data 内部
    values: &'node [V],    // UNSAFE: 指向 page.data 内部
}

impl<'node, K: Ord, V> BTreeNode<'node, K, V> {
    /// UNSAFE: 从页创建节点
    /// 
    /// # Safety
    /// 调用者必须确保：
    /// 1. page 包含有效的 B+Tree 节点数据
    /// 2. page 不会在节点生命周期内被修改或释放
    /// 3. keys/values 偏移量和长度正确
    pub(crate) unsafe fn from_page(page: Pin<&'node mut Page<'node>>) -> Result<Self>;
    
    /// 查找键（二分查找）
    pub(crate) fn search(&self, key: &K) -> Result<usize, usize>;
    
    /// 插入键值对（节点内）
    pub(crate) fn insert_cell(&mut self, index: usize, key: K, value: V) -> Result<()>;
    
    /// 删除键值对（节点内）
    pub(crate) fn remove_cell(&mut self, index: usize) -> Result<(K, V)>;
    
    /// 分裂节点
    pub(crate) fn split(&mut self, pager: &mut Pager) -> Result<(K, PageId)>;
    
    /// 合并节点
    pub(crate) fn merge(&mut self, sibling: &mut Self) -> Result<()>;
}
```

---

## WAL API

### WalWriter 类型

**职责**：Write-Ahead Log 写入器

```rust
/// WAL 写入器
/// 
/// 线程安全: Send + Sync（内部使用 Mutex）
pub(crate) struct WalWriter {
    file: Mutex<File>,
    current_offset: AtomicU64,
    salt: [u32; 2],
    checksum_state: Mutex<ChecksumState>,
}

impl WalWriter {
    /// 创建新 WAL 文件
    pub(crate) fn create(path: impl AsRef<Path>) -> Result<Self>;
    
    /// 打开已有 WAL 文件
    pub(crate) fn open(path: impl AsRef<Path>) -> Result<Self>;
    
    /// 追加 WAL 帧
    /// 
    /// # Parameters
    /// - `page_id`: 页 ID
    /// - `page_data`: 页数据（4096 字节）
    /// - `db_size`: 提交时的数据库页数（commit 时设置，否则为 0）
    /// 
    /// # Returns
    /// - `Ok(u64)`: WAL 偏移量
    pub(crate) fn append_frame(
        &self,
        page_id: PageId,
        page_data: &[u8; 4096],
        db_size: u32,
    ) -> Result<u64>;
    
    /// 标记提交点
    /// 
    /// 更新最后一帧的 db_size 字段
    pub(crate) fn mark_commit(&self, db_size: u32) -> Result<()>;
    
    /// 刷新到磁盘（fsync）
    pub(crate) fn sync(&self) -> Result<()>;
    
    /// 截断 WAL 文件（checkpoint 后）
    pub(crate) fn truncate(&self) -> Result<()>;
}

/// WAL 帧（32 bytes header + 4096 bytes page data）
#[derive(Debug)]
#[repr(C)]
pub(crate) struct WalFrame {
    pub page_id: PageId,
    pub db_size: u32,
    pub salt: [u32; 2],
    pub checksum: [u32; 2],
    pub reserved: u64,  // MVCC txn_id 预留
    pub page_data: [u8; 4096],
}
```

### WalReader 类型

**职责**：WAL 读取和恢复

```rust
/// WAL 读取器
/// 
/// 线程安全: !Send + !Sync
pub(crate) struct WalReader {
    file: File,
    header: WalHeader,
}

impl WalReader {
    /// 打开 WAL 文件
    pub(crate) fn open(path: impl AsRef<Path>) -> Result<Self>;
    
    /// 迭代所有帧
    pub(crate) fn frames(&self) -> WalFrameIterator;
    
    /// 恢复数据库（重放 WAL）
    /// 
    /// # Parameters
    /// - `pager`: 页管理器
    /// 
    /// # Implementation
    /// 1. 遍历所有 WAL 帧
    /// 2. 验证 checksum
    /// 3. 将页写入 Pager
    pub(crate) fn recover(&self, pager: &mut Pager) -> Result<()>;
}

/// WAL 帧迭代器
pub(crate) struct WalFrameIterator<'wal> {
    reader: &'wal WalReader,
    offset: u64,
}

impl<'wal> Iterator for WalFrameIterator<'wal> {
    type Item = Result<WalFrame>;
    
    fn next(&mut self) -> Option<Self::Item>;
}
```

---

## Freelist API

### Freelist 类型

**职责**：管理空闲页

```rust
/// 空闲页列表
/// 
/// 线程安全: !Send + !Sync
pub(crate) struct Freelist {
    free_pages: Vec<PageId>,
    trunk_page: Option<PageId>,
}

impl Freelist {
    /// 创建新 Freelist
    pub(crate) fn new() -> Self;
    
    /// 从页加载 Freelist
    pub(crate) fn load(page: &Page) -> Result<Self>;
    
    /// 分配空闲页
    pub(crate) fn allocate(&mut self) -> Option<PageId>;
    
    /// 释放页
    pub(crate) fn free(&mut self, page_id: PageId);
    
    /// 保存到页
    pub(crate) fn save(&self, page: &mut Page) -> Result<()>;
}
```

---

## Checksum API

### 校验和函数

**职责**：计算 WAL checksum

```rust
/// 计算 WAL checksum（SQLite 兼容）
/// 
/// # Parameters
/// - `data`: 数据（必须是 8 的倍数）
/// - `prev_c1`: 前一个 checksum 的 c1
/// - `prev_c2`: 前一个 checksum 的 c2
/// 
/// # Returns
/// - `(c1, c2)`: 新的 checksum 值
pub(crate) fn wal_checksum(data: &[u8], prev_c1: u32, prev_c2: u32) -> (u32, u32) {
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

/// CRC32 checksum（页校验）
pub(crate) fn crc32(data: &[u8]) -> u32;
```

---

## 错误类型

```rust
/// 存储层错误
#[derive(Debug, thiserror::Error)]
pub(crate) enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Page {0} not found")]
    PageNotFound(PageId),
    
    #[error("Corruption detected: {0}")]
    Corruption(String),
    
    #[error("WAL checksum mismatch")]
    ChecksumMismatch,
    
    #[error("B+Tree invariant violation: {0}")]
    BTreeInvariant(String),
}
```

---

**Storage API Version**: 1.0  
**Internal Use Only**: `pub(crate)`  
**Date**: 2025-12-10

