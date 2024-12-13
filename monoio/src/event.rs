use std::os::unix::prelude::{AsRawFd, RawFd};
use crate::driver::shared_fd::SharedFd;

pub(crate) struct EventFd {
  // RawFd
  raw: RawFd,
  // File hold the ownership of fd, only useful when drop
  _file: std::fs::File,
}

impl EventFd {
  pub fn new() -> std::io::Result<Self> {
    let fd = crate::syscall!(eventfd@RAW(0, libc::EFD_CLOEXEC))?;
    let file = unsafe {
      use std::os::unix::io::FromRawFd;
      std::fs::File::from_raw_fd(fd)
    };
    Ok(EventFd {
      raw: fd,
      _file: file,
    })
  }

  #[inline(always)]
  pub fn notify(&self) -> std::io::Result<()> {
    // Write data into EventFd to wake the executor.
    let buf = 0x1u64.to_ne_bytes();
    unsafe {
      // SAFETY: Writing number to eventfd is thread safe.
      libc::write(self.raw, buf.as_ptr().cast(), buf.len());
      Ok(())
    }
  }

  pub fn file(&self) -> std::io::Result<crate::fs::File> {
    let share_fd = SharedFd::new::<false>(self.raw)?;
    Ok(crate::fs::File::from_shared_fd(share_fd))
  }
}
