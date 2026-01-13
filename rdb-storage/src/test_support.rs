use std::fs::File;
use std::sync::Arc;

use rdb_infrastructure::BufferPool;

pub use crate::pager::{Pager, PagerError, Result};
pub use rdb_domain::PageId;

pub fn new_pager_for_test(file: File) -> Result<Pager<'static>> {
  Pager::new(file, 4096, Arc::new(BufferPool))
}

// ---- wrappers for integration tests (Pager<'static>) ----

pub fn pager_get_page<'a>(
  pager: &'a Pager<'static>,
  page_id: PageId,
) -> Result<&'a crate::page::Page<'static>> {
  pager.get_page(page_id)
}

pub fn pager_get_page_mut<'a>(
  pager: &'a mut Pager<'static>,
  page_id: PageId,
) -> Result<&'a mut crate::page::Page<'static>> {
  pager.get_page_mut(page_id)
}

pub fn pager_allocate_page(pager: &mut Pager<'static>) -> Result<PageId> {
  pager.allocate_page()
}

pub fn pager_flush_page(pager: &mut Pager<'static>, page_id: PageId) -> Result<()> {
  pager.flush_page(page_id)
}

pub fn pager_flush_all(pager: &mut Pager<'static>) -> Result<()> {
  pager.flush_all()
}
