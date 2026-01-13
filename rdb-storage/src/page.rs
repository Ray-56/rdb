use core::fmt;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU32, Ordering};

use rdb_domain::PageId;

/// 页类型（写入/读取页头的第 0 字节）
///
/// 磁盘编码
/// - 0x05: Internal
/// - 0x0D: Leaf
/// - 0x02: Overflow
/// - 0x01: Freelist
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PageType {
  /// B+Tree 内部节点页
  Internal = 0x05,
  /// B+Tree 叶子节点页
  Leaf = 0x0D,
  /// 溢出页：存放超过单页容量的 payload 的后续片段
  Overflow = 0x02,
  /// Freelist 管理页：记录可复用的空闲页
  Freelist = 0x01,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidPageType(pub u8);

impl fmt::Display for InvalidPageType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "invalid page type byte: 0x{:02X}", self.0)
  }
}

impl std::error::Error for InvalidPageType {}

impl TryFrom<u8> for PageType {
  type Error = InvalidPageType;

  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      0x05 => Ok(Self::Internal),
      0x0D => Ok(Self::Leaf),
      0x02 => Ok(Self::Overflow),
      0x01 => Ok(Self::Freelist),
      other => Err(InvalidPageType(other)),
    }
  }
}

impl PageType {
  /// 写回页头时用（保证与 #[repr(u8)] 的判别一致）
  pub(crate) const fn as_u8(self) -> u8 {
    self as u8
  }
}

// 页头固定为 32 字节（0x20）
pub const PAGE_HEADER_SIZE: usize = 32;

// 各字段在页头中的固定便宜（byte offset)
pub const OFF_PAGE_TYPE: usize = 0x0000; // 页类型（1 byte
pub(crate) const OFF_FIRST_FREEBLOCK: usize = 0x0001; // 第一个空闲块偏移（2 bytes）
pub(crate) const OFF_NUM_CELLS: usize = 0x0003; // 页内 cell 数量（2 bytes）
pub(crate) const OFF_CELL_CONTENT_AREA: usize = 0x0005; // cell 内容区域起始偏移（2 bytes）
pub(crate) const OFF_FRAGMENTED_BYTES: usize = 0x0007; // 碎片字节数（1 byte）
pub(crate) const OFF_RIGHT_CHILD: usize = 0x0008; // 仅内部节点：最右子页 ID（4 bytes）
pub(crate) const OFF_LSN: usize = 0x000C; // MVCC: 日志序列号（8 bytes）
pub(crate) const OFF_CHECKSUM: usize = 0x0014; // CRC32 校验和（4 bytes）
pub(crate) const OFF_RESERVED: usize = 0x0018; // 预留用于集群元数据（8 bytes）

/// 页头（逻辑结构）
///
/// 注意：不要依赖这个 struct 的内存部署来"直接 transmute/读写磁盘"。
/// 磁盘布局请使用 `decode/encode`，按固定 offset 读写。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageHeader {
  pub page_type: PageType,
  pub first_freeblock: u16,
  pub num_cells: u16,
  pub cell_content_area: u16,
  pub fragmented_bytes: u8,
  pub right_child: u32, // 仅 Internal 有意义；其它页一般写 0
  pub lsn: u64,         // 预留
  pub checksum: u32,    // 预留/或后续做 CRC32
  pub reserved: u64,    // 预留用于集群元数据
}

impl PageHeader {
  pub const SIZE: usize = PAGE_HEADER_SIZE;

  /// 从"页头 32 字节"解析出 PageHeader（小端序）
  pub fn decode(buf: &[u8; PAGE_HEADER_SIZE]) -> Result<Self, InvalidPageType> {
    let page_type = PageType::try_from(buf[OFF_PAGE_TYPE])?;

    Ok(Self {
      page_type,
      first_freeblock: read_u16_le(buf, OFF_FIRST_FREEBLOCK),
      num_cells: read_u16_le(buf, OFF_NUM_CELLS),
      cell_content_area: read_u16_le(buf, OFF_CELL_CONTENT_AREA),
      fragmented_bytes: buf[OFF_FRAGMENTED_BYTES],
      right_child: read_u32_le(buf, OFF_RIGHT_CHILD),
      lsn: read_u64_le(buf, OFF_LSN),
      checksum: read_u32_le(buf, OFF_CHECKSUM),
      reserved: read_u64_le(buf, OFF_RESERVED),
    })
  }

  /// 把 PageHeader 写入“页头 32 字节”（小端序）
  pub(crate) fn encode(&self, buf: &mut [u8; PAGE_HEADER_SIZE]) {
    // 先清零，避免旧数据残留（尤其 reserved / right_child 这类字段）
    *buf = [0u8; PAGE_HEADER_SIZE];

    buf[OFF_PAGE_TYPE] = self.page_type.as_u8();
    write_u16_le(buf, OFF_FIRST_FREEBLOCK, self.first_freeblock);
    write_u16_le(buf, OFF_NUM_CELLS, self.num_cells);
    write_u16_le(buf, OFF_CELL_CONTENT_AREA, self.cell_content_area);
    buf[OFF_FRAGMENTED_BYTES] = self.fragmented_bytes;
    write_u32_le(buf, OFF_RIGHT_CHILD, self.right_child);
    write_u64_le(buf, OFF_LSN, self.lsn);
    write_u32_le(buf, OFF_CHECKSUM, self.checksum);
    write_u64_le(buf, OFF_RESERVED, self.reserved);
  }
}

// ---- 小端序读写工具（只操作 buf，不做任何 unsafe）----

fn read_u16_le(buf: &[u8; PAGE_HEADER_SIZE], off: usize) -> u16 {
  u16::from_le_bytes([buf[off], buf[off + 1]])
}

fn write_u16_le(buf: &mut [u8; PAGE_HEADER_SIZE], off: usize, v: u16) {
  let b = v.to_le_bytes();
  buf[off] = b[0];
  buf[off + 1] = b[1];
}

fn read_u32_le(buf: &[u8; PAGE_HEADER_SIZE], off: usize) -> u32 {
  u32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]])
}

fn write_u32_le(buf: &mut [u8; PAGE_HEADER_SIZE], off: usize, v: u32) {
  let b = v.to_le_bytes();
  buf[off] = b[0];
  buf[off + 1] = b[1];
  buf[off + 2] = b[2];
  buf[off + 3] = b[3];
}

fn read_u64_le(buf: &[u8; PAGE_HEADER_SIZE], off: usize) -> u64 {
  u64::from_le_bytes([
    buf[off],
    buf[off + 1],
    buf[off + 2],
    buf[off + 3],
    buf[off + 4],
    buf[off + 5],
    buf[off + 6],
    buf[off + 7],
  ])
}

fn write_u64_le(buf: &mut [u8; PAGE_HEADER_SIZE], off: usize, v: u64) {
  let b = v.to_le_bytes();
  buf[off] = b[0];
  buf[off + 1] = b[1];
  buf[off + 2] = b[2];
  buf[off + 3] = b[3];
  buf[off + 4] = b[4];
  buf[off + 5] = b[5];
  buf[off + 6] = b[6];
  buf[off + 7] = b[7];
}

/// 单个 4KB 数据页
///
/// - 磁盘上的"页容器"就是 `data` 这 4096 字节（其中前 32 字节是 PageHeader)
/// - `page_id/dirty/pin_count` 是内存运行时元数据，不写入磁盘
///
/// 生命周期 `'page`：把 Page 绑定到 Pager 的生命周期（避免悬垂引用/指针）。
/// 线程安全：后续如果你在 Page 内保存原始指针做内存映射，通常会选择 !Send + !Sync。
#[repr(C, align(4096))]
pub struct Page<'page> {
  /// 页的原始字节内容（包含页头、cell pointer array、cell content 等）
  pub(crate) data: [u8; 4096],

  /// 页 ID（逻辑地址：第几页）
  pub(crate) page_id: PageId,

  /// 脏标记：页内容是否被修改，需要 flush 回磁盘
  pub(crate) dirty: bool,

  /// Pin 计数：>0 表示该页正在被使用，不能被缓存淘汰
  pub(crate) pin_count: AtomicU32,

  /// 把生命周期 `'page` 绑定到这个类型上（后续 Pager/BufferPoll 会用到）
  pub(crate) _phantom: PhantomData<&'page mut ()>,
}

impl<'page> Page<'page> {
  /// 创建一个新页：初始化 4KB 全 0,并写入基础页头
  pub fn new(page_id: PageId, page_type: PageType) -> Self {
    let mut page = Self {
      data: [0u8; 4096],
      page_id,
      dirty: false,
      pin_count: AtomicU32::new(0),
      _phantom: PhantomData,
    };

    // 初始化页头（最小可用）
    let header = PageHeader {
      page_type,
      first_freeblock: 0,
      num_cells: 0,
      // 初始化 cell content 从页尾开始（SQLite/很多 BTree 页都是这么做）
      cell_content_area: 4096,
      fragmented_bytes: 0,
      right_child: 0,
      lsn: 0,
      checksum: 0,
      reserved: 0,
    };

    // 写入 data[0..32]
    let mut buf = [0u8; PAGE_HEADER_SIZE];
    header.encode(&mut buf);
    page.data[..PAGE_HEADER_SIZE].copy_from_slice(&buf);

    page
  }

  /// 从磁盘字节载入一个页（会校验第 0 字节的 page_type 是否合法）
  pub fn from_bytes(page_id: PageId, data: [u8; 4096]) -> Result<Self, InvalidPageType> {
    // 校验页类型字节，避免后续解析把坏页当好页
    let _ = PageType::try_from(data[OFF_PAGE_TYPE])?;

    Ok(Self { data, page_id, dirty: false, pin_count: AtomicU32::new(0), _phantom: PhantomData })
  }

  #[inline]
  pub fn page_id(&self) -> PageId {
    self.page_id
  }

  /// 返回页类型（因为 from_bytes/new 已保证合法，所以这里不需要 Result）
  pub fn page_type(&self) -> PageType {
    // SAFETY: new()/from_bytes 保证 data[0] 必定合法 page_type
    match self.data[OFF_PAGE_TYPE] {
      0x05 => PageType::Internal,
      0x0D => PageType::Leaf,
      0x02 => PageType::Overflow,
      0x01 => PageType::Freelist,
      _ => PageType::Freelist, // 理论上到不了；为了避免 panic/unwrap，写个兜底
    }
  }

  #[inline]
  pub(crate) fn mark_dirty(&mut self) {
    self.dirty = true;
  }

  #[inline]
  pub fn data(&self) -> &[u8; 4096] {
    &self.data
  }

  #[inline]
  pub(crate) fn data_mut(&mut self) -> &mut [u8; 4096] {
    self.mark_dirty();
    &mut self.data
  }

  // （可选）后面做 BufferPool 会用到：pin/unpin
  #[allow(dead_code)]
  pub(crate) fn pin(&self) {
    self.pin_count.fetch_add(1, Ordering::Relaxed);
  }

  #[allow(dead_code)]
  pub(crate) fn unpin(&self) {
    self.pin_count.fetch_sub(1, Ordering::Relaxed);
  }

  /// 安全版：推荐内部都用这个（不吞错误）
  pub fn try_parse_header(&self) -> Result<PageHeader, InvalidPageType> {
    let mut buf = [0u8; PAGE_HEADER_SIZE];
    buf.copy_from_slice(&self.data[..PAGE_HEADER_SIZE]);
    PageHeader::decode(&buf)
  }

  /// 兼容 spec 草图：返回 PageHeader（遇到坏页类型会退成一个“最保守的 header”）
  pub(crate) fn parse_header(&self) -> PageHeader {
    match self.try_parse_header() {
      Ok(h) => h,
      Err(_) => PageHeader {
        page_type: PageType::Freelist, // 兜底：避免继续按 BTree 页解析
        first_freeblock: 0,
        num_cells: 0,
        cell_content_area: 4096,
        fragmented_bytes: 0,
        right_child: 0,
        lsn: 0,
        checksum: 0,
        reserved: 0,
      },
    }
  }

  /// 写入页头：写回 data[0..32] 并标记脏页
  pub fn write_header(&mut self, header: &PageHeader) {
    let mut buf = [0u8; PAGE_HEADER_SIZE];
    header.encode(&mut buf);
    self.data[..PAGE_HEADER_SIZE].copy_from_slice(&buf);
    self.mark_dirty();
  }
}
