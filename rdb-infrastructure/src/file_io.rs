use std::fs::File;
use std::io;
use std::io::ErrorKind;

#[cfg(unix)]
use std::os::unix::fs::FileExt;

#[cfg(windows)]
use std::os::windows::fs::FileExt;

/// 从文件的指定偏移读取，直到把 buf 填满（等价于 pread + read_exact)
pub fn read_exact_at(file: &File, mut buf: &mut [u8], mut off: u64) -> io::Result<()> {
  while !buf.is_empty() {
    let n = file.read_at(buf, off)?;
    if n == 0 {
      return Err(io::Error::new(ErrorKind::UnexpectedEof, "unexpected EOF"));
    }
    off += n as u64;
    buf = &mut buf[n..];
  }
  Ok(())
}

/// 写入到文件的指定偏移，直到把 buf 写完（等价于 pwrite + write_all)
pub fn write_all_at(file: &File, mut buf: &[u8], mut off: u64) -> io::Result<()> {
  while !buf.is_empty() {
    let n = file.write_at(buf, off)?;
    if n == 0 {
      return Err(io::Error::new(
        ErrorKind::WriteZero,
        "failed to write whole buffer",
      ));
    }
    off += n as u64;
    buf = &buf[n..];
  }
  Ok(())
}

/// 获取文件长度（字节数）
pub fn file_len(file: &File) -> io::Result<u64> {
  Ok(file.metadata()?.len())
}

/// 校验文件长度必须是 page_size 的整倍数（否则通常意味着文件损坏或 page_size 配置错）
pub fn validate_file_len_is_multiple_of_page_size(len: u64, page_size: usize) -> io::Result<()> {
  if page_size == 0 {
    return Err(io::Error::new(
      ErrorKind::InvalidInput,
      "page_size must be > 0",
    ));
  }
  if len % page_size as u64 != 0 {
    return Err(io::Error::new(
      ErrorKind::InvalidData,
      format!("file_size={len} is not a multiple of page_size={page_size}"),
    ));
  }
  Ok(())
}

/// 把文件扩展到执行长度（字节）
pub fn set_len(file: &File, len: u64) -> io::Result<()> {
  file.set_len(len)
}

/// 给定 page_id（从 1 开始）与 page_size，计算该页在文件中的起始偏移
pub fn page_offset(page_id: u32, page_size: usize) -> io::Result<u64> {
  if page_size == 0 {
    return Err(io::Error::new(
      ErrorKind::InvalidInput,
      "page_id must start from 1",
    ));
  }
  Ok((page_id as u64 - 1) * page_size as u64)
}
