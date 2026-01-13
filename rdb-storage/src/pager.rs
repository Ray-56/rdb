use core::marker::PhantomData;
use core::sync::atomic::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, ErrorKind};
use std::rc::Rc;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use std::{cell::RefCell, os::unix::fs::FileExt};

use rdb_domain::PageId;
use rdb_infrastructure::file_io::{page_offset, read_exact_at, write_all_at};
use rdb_infrastructure::BufferPool;

use crate::page::{InvalidPageType, Page};

#[derive(thiserror::Error, Debug)]
pub enum PagerError {
  #[error("io error: {0}")]
  Io(#[from] io::Error),

  #[error("invalid page type: {0}")]
  InvalidPageType(#[from] InvalidPageType),

  #[error("unsupported page_size={0} (currently only 4096 is supported)")]
  UnsupportedPageSize(usize),

  #[error("corrupt db file: file_size={len} is not a multiple of page_size={page_size}")]
  CorruptFile { len: u64, page_size: usize },

  #[error("page not found: {0:?}")]
  PageNotFound(PageId),
}

pub type Result<T> = std::result::Result<T, PagerError>;

/// 页管理器
///
/// - `file`：数据库文件句柄
/// - `page_size`：页大小（通常 4096）
/// - `page_count`：当前总页数
/// - `buffer_pool`：缓存池（占位类型，T38 会实现）
/// - `page_index`：页索引（page_id -> index）
/// - `pages`：页容器（page_id -> Page）
/// - `_not_send_sync`：用 Rc 把 Pager 变成 !Send + !Sync
/// - `_phantom`：绑定 'db 生命周期
pub struct Pager<'db> {
  pub(crate) file: File,
  pub(crate) page_size: usize,
  pub(crate) page_count: AtomicU32,
  pub(crate) buffer_pool: Arc<BufferPool>,

  // 先用最简单的“内部缓存”：page_id -> index, pages 存 Box<Page> 保证地址稳定
  pub(crate) page_index: RefCell<HashMap<PageId, usize>>,
  pub(crate) pages: RefCell<Vec<Box<Page<'db>>>>,

  pub(crate) _not_send_sync: PhantomData<Rc<()>>,
  pub(crate) _phantom: PhantomData<&'db mut ()>,
}

impl<'db> Pager<'db> {
  pub(crate) fn new(file: File, page_size: usize, buffer_pool: Arc<BufferPool>) -> Result<Self> {
    // 由于 Page 固定是 [u8; 4096], 这里先支持 4096
    if page_size != 4096 {
      return Err(PagerError::UnsupportedPageSize(page_size));
    }

    let len = file.metadata()?.len();
    if len % page_size as u64 != 0 {
      return Err(PagerError::CorruptFile { len, page_size });
    }

    let page_count = (len / page_size as u64) as u32;

    Ok(Self {
      file,
      page_size,
      page_count: AtomicU32::new(page_count),
      buffer_pool,

      page_index: RefCell::new(HashMap::new()),
      pages: RefCell::new(Vec::new()),

      _not_send_sync: PhantomData,
      _phantom: PhantomData,
    })
  }

  pub(crate) fn page_count(&self) -> u32 {
    self.page_count.load(Ordering::Relaxed)
  }

  pub(crate) fn get_page(&self, page_id: PageId) -> Result<&Page<'db>> {
    // 1) 命中缓存：用 raw ptr 脱离 RefCell borrow 的生命周期
    if let Some(ptr) = self.get_cached_ptr(page_id) {
      // SAFETY: ptr 指向 Box<Page> 的堆内存，生命周期受 Pager 管控
      return Ok(unsafe { &*ptr });
    }

    // 2) 缓存未命中：从磁盘读入并放入缓存
    let data = self.read_page_bytes(page_id)?;
    let page = Page::from_bytes(page_id, data)?; // 这里会校验 page_type 字节

    let mut pages = self.pages.borrow_mut();
    let mut index = self.page_index.borrow_mut();

    let idx = pages.len();
    pages.push(Box::new(page));
    index.insert(page_id, idx);

    let ptr = (&*pages[idx]) as *const Page<'db>;
    drop(index);
    drop(pages);

    // SAFETY: 同上
    Ok(unsafe { &*ptr })
  }

  pub(crate) fn get_page_mut(&mut self, page_id: PageId) -> Result<&mut Page<'db>> {
    // 先检查缓存（确保 borrow() 的 Ref 在这一行结束后就被 drop）
    let cached_idx = self.page_index.borrow().get(&page_id).copied();
    
    let idx = if let Some(i) = cached_idx {
      i
    } else {
      // 缓存未命中：从磁盘读入
      let data = self.read_page_bytes(page_id)?;
      let page = Page::from_bytes(page_id, data)?;

      let mut pages = self.pages.borrow_mut();
      let mut index = self.page_index.borrow_mut();

      let idx = pages.len();
      pages.push(Box::new(page));
      index.insert(page_id, idx);

      idx
    };

    // 返回 &mut: 同样用 raw ptr 脱离 RefCell borrow
    let mut pages = self.pages.borrow_mut();
    let ptr = (&mut *pages[idx]) as *mut Page<'db>;
    drop(pages);

    // SAFETY: get_page_mut 需要 &mut self，外部无法同时持有同 Pager 的其它引用
    Ok(unsafe { &mut *ptr })
  }

  pub(crate) fn allocate_page(&mut self) -> Result<PageId> {
    // TODO: (T93) 先从 Freelist 分配；这里先实现“文件尾部扩展”
    let next = self.page_count.load(Ordering::Relaxed) + 1;

    // 扩展文件长度
    let new_len = next as u64 * self.page_size as u64;
    self.file.set_len(new_len)?;

    // 把新页内容写成全 0 （避免读到旧垃圾数据）
    let zero = [0u8; 4096];
    let off = (next as u64 - 1) * self.page_size as u64;
    write_all_at(&self.file, &zero, off)?;

    self.page_count.store(next, Ordering::Relaxed);
    Ok(PageId::new(next))
  }

  pub(crate) fn flush_page(&mut self, page_id: PageId) -> Result<()> {
    let idx = self
      .page_index
      .borrow()
      .get(&page_id)
      .copied()
      .ok_or(PagerError::PageNotFound(page_id))?;

    let mut pages = self.pages.borrow_mut();
    let page: &mut Page<'db> = &mut *pages[idx];

    if page.dirty {
      let off = (u64::from(page_id.into_inner()) - 1) * self.page_size as u64;
      write_all_at(&self.file, &page.data, off)?;
      page.dirty = false;
    }

    Ok(())
  }

  pub(crate) fn flush_all(&mut self) -> Result<()> {
    // 把当前缓存里的所有脏页刷盘
    let ids: Vec<PageId> = self.page_index.borrow().keys().copied().collect();
    for id in ids {
      self.flush_page(id)?;
    }
    Ok(())
  }

  pub(crate) fn free_page(&mut self, page_id: PageId) -> Result<()> {
    // TODO: T90/T91 接入 Freelist
    Ok(())
  }

  pub(crate) unsafe fn get_page_ptr(&self, page_id: PageId) -> *const Page<'db> {
    match self.get_page(page_id) {
      Ok(p) => p as *const Page<'db>,
      Err(_) => core::ptr::null(),
    }
  }

  pub(crate) unsafe fn get_page_mut_ptr(&mut self, page_id: PageId) -> *mut Page<'db> {
    match self.get_page_mut(page_id) {
      Ok(p) => p as *mut Page<'db>,
      Err(_) => core::ptr::null_mut(),
    }
  }

  fn get_cached_ptr(&self, page_id: PageId) -> Option<*const Page<'db>> {
    let idx = self.page_index.borrow().get(&page_id).copied()?;
    let pages = self.pages.borrow();
    Some((&*pages[idx]) as *const Page<'db>)
  }

  fn read_page_bytes(&self, page_id: PageId) -> Result<[u8; 4096]> {
    let id = page_id.into_inner();
    if id == 0 {
      return Err(PagerError::PageNotFound(page_id));
    }

    let count = self.page_count();
    if id > count {
      return Err(PagerError::PageNotFound(page_id));
    }

    let mut buf = [0u8; 4096];
    let off = (u64::from(id) - 1) * self.page_size as u64;
    read_exact_at(&self.file, &mut buf, off)?;
    Ok(buf)
  }
}
