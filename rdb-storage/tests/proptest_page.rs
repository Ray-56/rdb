use proptest::prelude::*;
use rdb_domain::PageId;
use rdb_storage::page::{Page, PageHeader, PageType, PAGE_HEADER_SIZE};

/// 生成任意合法的 PageType
fn arb_page_type() -> impl Strategy<Value = PageType> {
  prop_oneof![
    Just(PageType::Internal),
    Just(PageType::Leaf),
    Just(PageType::Overflow),
    Just(PageType::Freelist),
  ]
}

/// 生成任意合法的 PageHeader
fn arb_page_header() -> impl Strategy<Value = PageHeader> {
  (
    arb_page_type(),
    any::<u16>(),   // first_freeblock
    any::<u16>(),   // num_cells
    0u16..=4096u16, // cell_content_area(必须在页内)
    any::<u8>(),    // fragmented_bytes
    any::<u32>(),   // right_child
    any::<u64>(),   // lsn
    any::<u32>(),   // checksum
    any::<u64>(),   // reserved
  )
    .prop_map(
      |(
        page_type,
        first_freeblock,
        num_cells,
        cell_content_area,
        fragmented_bytes,
        right_child,
        lsn,
        checksum,
        reserved,
      )| {
        PageHeader {
          page_type,
          first_freeblock,
          num_cells,
          cell_content_area,
          fragmented_bytes,
          right_child,
          lsn,
          checksum,
          reserved,
        }
      },
    )
}

/// 生成任意合法的 PageId (1..=u32::MAX)
fn arb_page_id() -> impl Strategy<Value = PageId> {
  (1u32..=u32::MAX).prop_map(PageId::new)
}

proptest! {
  /// 属性测试1: PageHeader 编码/解码往返一致性
  ///
  /// 对于任意合法的 PageHeader，encode 后再 decode 应该得到相同的结果
  #[test]
  fn page_header_encode_decode_roundtrip(
    page_id in arb_page_id(),
    page_type in arb_page_type(),
    header in arb_page_header()
  ) {
    let mut page = Page::new(page_id, page_type);
    page.write_header(&header);

    let decoded = page.try_parse_header().expect("parse should succeed");

    prop_assert_eq!(header, decoded);
  }

  /// 属性测试2: Page 创建后立即读取 header 应该一致
  ///
  /// 使用 Page::new() 创建的页，parse_header() 应该返回对应的 PageType
  #[test]
  fn page_new_header_consistent(page_id in arb_page_id(), page_type in arb_page_type()) {
    let page = Page::new(page_id, page_type);

    prop_assert_eq!(page.page_id(), page_id);
    prop_assert_eq!(page.page_type(), page_type);

    let header = page.try_parse_header().expect("parse header should succeed");
    prop_assert_eq!(header.page_type, page_type);
  }

  /// 属性测试3: Page write_header 后 parse_header 应该一致
  ///
  /// 写入任意合法的 PageHeader, 在读取应该得到相同的值
  #[test]
  fn page_write_read_header_roundtrip(page_id in arb_page_id(), page_type in arb_page_type(), header_to_write in arb_page_header()) {
    let mut page = Page::new(page_id, page_type);

    page.write_header(&header_to_write);

    let header_read = page.try_parse_header().expect("parse should succeed");

    prop_assert_eq!(header_to_write, header_read);
  }

  /// 属性测试4: Page::from_bytes 往返一致性
  ///
  /// 创建页 -> 获取字节 -> 从字节重建 -> 应该保持一致
  #[test]
  fn page_from_bytes_roundtrip(page_id in arb_page_id(), header in arb_page_header()) {
    let mut page1 = Page::new(page_id, header.page_type);
    page1.write_header(&header);

    let data = *page1.data();

    let page2 = Page::from_bytes(page_id, data).expect("from_bytes should succeed");

    prop_assert_eq!(page2.page_id(), page_id);
    prop_assert_eq!(page2.page_type(), header.page_type);

    let header2 = page2.try_parse_header().expect("parse should succeed");
    prop_assert_eq!(header, header2);
  }

  /// 属性测试5: data() 读取写入 header 后的一致性
  ///
  /// 写入 header 后，通过 data() 直接读取页头字节应该能正确解码
  #[test]
  fn page_data_read_after_write_header(
    page_id in arb_page_id(),
    header in arb_page_header()
  ) {
    let mut page = Page::new(page_id, header.page_type);
    page.write_header(&header);

    // 通过 data() 读取前 32 字节
    let data = page.data();
    let mut header_bytes = [0u8; PAGE_HEADER_SIZE];
    header_bytes.copy_from_slice(&data[..PAGE_HEADER_SIZE]);

    let decoded = PageHeader::decode(&header_bytes).expect("decode should succeed");
    prop_assert_eq!(header, decoded);
  }

  /// 属性测试6: 多次写入 header 应保持最后一次的值
  ///
  /// 顺序写入多个不同的 header，最后读取应该是最后一次写入的值
  #[test]
  fn page_multiple_header_write_last_wins(
    page_id in arb_page_id(),
    headers in proptest::collection::vec(arb_page_header(), 1..10)
  ) {
    let initial_type = headers[0].page_type;
    let mut page = Page::new(page_id, initial_type);

    for header in &headers {
      page.write_header(header);
    }

    let final_header = page.try_parse_header().expect("parse should succeed");
    let expected = headers.last().unwrap();

    prop_assert_eq!(&final_header, expected);
  }
}
