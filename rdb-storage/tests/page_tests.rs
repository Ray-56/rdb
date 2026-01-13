use rdb_domain::PageId;
use rdb_storage::page::{Page, PageHeader, PageType, OFF_PAGE_TYPE, PAGE_HEADER_SIZE};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn page_new_sets_header_and_metadata() -> TestResult {
  let page = Page::new(PageId::new(1), PageType::Leaf);

  // page_id / page_type
  assert_eq!(page.page_id(), PageId::new(1));
  assert_eq!(page.page_type(), PageType::Leaf);

  // header decode must secceed and match
  let header = page.try_parse_header()?;
  assert_eq!(header.page_type, PageType::Leaf);
  assert_eq!(header.num_cells, 0);
  assert_eq!(header.first_freeblock, 0);
  assert_eq!(header.fragmented_bytes, 0);

  // cell_content_area 初始为页尾
  assert_eq!(header.cell_content_area, 4096);

  Ok(())
}

#[test]
fn page_from_bytes_rejects_invalid_page_type() {
  let mut data = [0u8; 4096];
  data[OFF_PAGE_TYPE] = 0xFF; // 非法

  let r = Page::from_bytes(PageId::new(1), data);
  assert!(r.is_err());
}

#[test]
fn page_write_header_roundtrip() -> TestResult {
  let mut page = Page::new(PageId::new(1), PageType::Internal);

  // 改一个 header，再写回，再读出来校验
  let mut header = page.try_parse_header()?;
  header.page_type = PageType::Overflow;
  header.num_cells = 123;
  header.first_freeblock = 0x002A;
  header.cell_content_area = 0x0F00;
  header.fragmented_bytes = 9;
  header.right_child = 42;
  header.lsn = 999;
  header.checksum = 0xDEADBEEF;
  header.reserved = 0x1122334455667788;

  page.write_header(&header);

  let header2 = page.try_parse_header()?;
  assert_eq!(header2, header);

  Ok(())
}

#[test]
fn page_header_bytes_are_written_to_first_32_bytes() -> TestResult {
  let mut page = Page::new(PageId::new(1), PageType::Leaf);

  let header = PageHeader {
    page_type: PageType::Freelist,
    first_freeblock: 1,
    num_cells: 2,
    cell_content_area: 4096,
    fragmented_bytes: 3,
    right_child: 4,
    lsn: 5,
    checksum: 6,
    reserved: 7,
  };

  page.write_header(&header);

  // 前 32 字节能 decode 成同一个 header
  let mut buf = [0u8; PAGE_HEADER_SIZE];
  buf.copy_from_slice(&page.data()[..PAGE_HEADER_SIZE]);
  let decoded = PageHeader::decode(&buf)?;
  assert_eq!(decoded, header);

  Ok(())
}
