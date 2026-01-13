use std::fs::{File, OpenOptions};
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rdb_domain::PageId;
use rdb_infrastructure::file_io::{read_exact_at, write_all_at};
use rdb_storage::page::{Page, PageHeader, PageType, PAGE_HEADER_SIZE};
use rdb_storage::test_support::{
  new_pager_for_test, pager_allocate_page, pager_flush_all, pager_flush_page, pager_get_page,
  pager_get_page_mut, PagerError,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct TempFile {
  path: PathBuf,
}

impl TempFile {
  fn new(prefix: &str) -> io::Result<(Self, File)> {
    let mut path = std::env::temp_dir();

    let nanos = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap_or_default()
      .as_nanos();

    path.push(format!("{prefix}_{}_{}.db", std::process::id(), nanos));

    let file = OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .truncate(true)
      .open(&path)?;

    Ok((Self { path }, file))
  }

  fn reopen_rw(&self) -> io::Result<File> {
    OpenOptions::new().read(true).write(true).open(&self.path)
  }
}

impl Drop for TempFile {
  fn drop(&mut self) {
    let _ = std::fs::remove_file(&self.path);
  }
}

fn write_page(path: &TempFile, page_id: u32, page: &Page<'_>) -> io::Result<()> {
  let file = path.reopen_rw()?;
  let off = (page_id as u64 - 1) * 4096;
  write_all_at(&file, page.data(), off)
}

fn read_header(path: &TempFile, page_id: u32) -> io::Result<PageHeader> {
  let file = path.reopen_rw()?;
  let mut buf = [0u8; PAGE_HEADER_SIZE];
  let off = (page_id as u64 - 1) * 4096;
  read_exact_at(&file, &mut buf, off)?;
  Ok(PageHeader::decode(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
}

#[test]
fn pager_new_rejects_corrupt_file_len() -> TestResult {
  let (_tmp, file) = TempFile::new("rdb_pager_corrupt_len")?;
  file.set_len(1)?; // 不是 4096 的整数倍

  let r = new_pager_for_test(file);
  match r {
    Err(PagerError::CorruptFile { .. }) => Ok(()),
    Err(e) => Err(format!("expected CorruptFile, got Err({e})").into()),
    Ok(_) => Err("expected CorruptFile, got Ok(Pager)".into()),
  }
}

#[test]
fn pager_get_page_rejects_page_id_0() -> TestResult {
  let (_tmp, file) = TempFile::new("rdb_pager_pageid0")?;
  let pager = new_pager_for_test(file)?;

  let r = pager_get_page(&pager, PageId::new(0));
  match r {
    Err(PagerError::PageNotFound(id)) => {
      assert_eq!(id, PageId::new(0));
      Ok(())
    }
    Err(e) => Err(format!("expected PageNotFound(0), got: Err({e})").into()),
    Ok(_) => Err("expected PageNotFound(0), got Ok(...)".into()),
  }
}

#[test]
fn pager_get_page_rejects_page_id_greater_than_page_count() -> TestResult {
  let (tmp, file) = TempFile::new("rdb_pager_oob")?;

  // 写入 1 页有效数据
  file.set_len(4096)?;
  let p1 = Page::new(PageId::new(1), PageType::Leaf);
  write_page(&tmp, 1, &p1)?;

  let pager = new_pager_for_test(file)?;

  let r = pager_get_page(&pager, PageId::new(2));
  match r {
    Err(PagerError::PageNotFound(id)) => {
      assert_eq!(id, PageId::new(2));
      Ok(())
    }
    Err(e) => Err(format!("expected PageNotFound(2), got: Err({e})").into()),
    Ok(_) => Err("expected PageNotFound(2), got Ok(...)".into()),
  }
}

#[test]
fn pager_get_page_reads_from_disk_and_is_cached() -> TestResult {
  let (tmp, file) = TempFile::new("rdb_pager_cache")?;

  file.set_len(4096)?;
  let p1 = Page::new(PageId::new(1), PageType::Internal);
  write_page(&tmp, 1, &p1)?;

  let pager = new_pager_for_test(file)?;

  let a = pager_get_page(&pager, PageId::new(1))?;
  let b = pager_get_page(&pager, PageId::new(1))?;

  assert_eq!(a.page_id(), PageId::new(1));
  assert_eq!(a.page_type(), PageType::Internal);

  // 同一页应命中缓存（同一地址）
  assert_eq!((a as *const _), (b as *const _));

  Ok(())
}

#[test]
fn pager_flush_page_without_cache_entry_returns_not_found() -> TestResult {
  let (tmp, file) = TempFile::new("rdb_pager_flush_not_cached")?;

  file.set_len(4096)?;
  let p1 = Page::new(PageId::new(1), PageType::Leaf);
  write_page(&tmp, 1, &p1)?;

  let mut pager = new_pager_for_test(file)?;

  // 没有 get_page/get_page_mut 过，因此 page_index 里没有该页
  let r = pager_flush_page(&mut pager, PageId::new(1));
  match r {
    Err(PagerError::PageNotFound(id)) => {
      assert_eq!(id, PageId::new(1));
      Ok(())
    }
    other => Err(format!("expected PageNotFound(1), got: {other:?}").into()),
  }
}

#[test]
fn pager_get_page_mut_then_flush_page_persists_changes() -> TestResult {
  let (tmp, file) = TempFile::new("rdb_pager_flush_one")?;

  file.set_len(4096)?;
  let p1 = Page::new(PageId::new(1), PageType::Leaf);
  write_page(&tmp, 1, &p1)?;

  let mut pager = new_pager_for_test(file)?;

  {
    let page = pager_get_page_mut(&mut pager, PageId::new(1))?;
    let mut h = page.try_parse_header()?;
    h.num_cells = 7;
    h.right_child = 42;
    page.write_header(&h); // 这会标记 dirty
  }

  pager_flush_page(&mut pager, PageId::new(1))?;

  let h2 = read_header(&tmp, 1)?;
  assert_eq!(h2.page_type, PageType::Leaf);
  assert_eq!(h2.num_cells, 7);
  assert_eq!(h2.right_child, 42);

  Ok(())
}

#[test]
fn pager_flush_all_flushes_multiple_dirty_pages() -> TestResult {
  let (tmp, file) = TempFile::new("rdb_pager_flush_all")?;

  file.set_len(4096 * 2)?;

  let p1 = Page::new(PageId::new(1), PageType::Leaf);
  let p2 = Page::new(PageId::new(2), PageType::Internal);
  write_page(&tmp, 1, &p1)?;
  write_page(&tmp, 2, &p2)?;

  let mut pager = new_pager_for_test(file)?;

  {
    let page1 = pager_get_page_mut(&mut pager, PageId::new(1))?;
    let mut h1 = page1.try_parse_header()?;
    h1.num_cells = 11;
    page1.write_header(&h1);
  }

  {
    let page2 = pager_get_page_mut(&mut pager, PageId::new(2))?;
    let mut h2 = page2.try_parse_header()?;
    h2.num_cells = 22;
    page2.write_header(&h2);
  }

  pager_flush_all(&mut pager)?;

  let h1 = read_header(&tmp, 1)?;
  let h2 = read_header(&tmp, 2)?;

  assert_eq!(h1.page_type, PageType::Leaf);
  assert_eq!(h1.num_cells, 11);

  assert_eq!(h2.page_type, PageType::Internal);
  assert_eq!(h2.num_cells, 22);

  Ok(())
}

#[test]
fn pager_allocate_page_extends_file_and_is_zero_filled() -> TestResult {
  let (tmp, file) = TempFile::new("rdb_pager_alloc")?;
  let mut pager = new_pager_for_test(file)?;

  let id1 = pager_allocate_page(&mut pager)?;
  assert_eq!(id1, PageId::new(1));

  let f = tmp.reopen_rw()?;
  let len = f.metadata()?.len();
  assert_eq!(len, 4096);

  let mut buf = [0u8; 4096];
  read_exact_at(&f, &mut buf, 0)?;
  assert!(buf.iter().all(|&b| b == 0));

  Ok(())
}
