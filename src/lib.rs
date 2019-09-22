#![cfg(unix)]
#![feature(rustc_private)]
extern crate libc;

use async_std::fs::File;
use async_std::task::blocking;
use async_trait::async_trait;
use std::io::{Error, Result};
use std::os::unix::io::AsRawFd;

#[async_trait]
pub trait FileExt {
    async fn lock_shared(&self) -> Result<()>;

    async fn lock_exclusive(&self) -> Result<()>;

    async fn unlock(&self) -> Result<()>;
}

#[async_trait]
impl FileExt for File {
    async fn lock_shared(&self) -> Result<()> {
        flock(self, libc::LOCK_SH).await
    }

    async fn lock_exclusive(&self) -> Result<()> {
        flock(self, libc::LOCK_EX).await
    }

    async fn unlock(&self) -> Result<()> {
        flock(self, libc::LOCK_UN).await
    }
}

#[inline]
async fn flock(file: &File, flag: libc::c_int) -> Result<()> {
    let fd = file.as_raw_fd();
    blocking::spawn(async move {
        let ret = unsafe { libc::flock(fd, flag) };
        if ret < 0 {
           return Err(Error::last_os_error());
        }
        Ok(())
    }).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::{fs, task};

    async fn run_lock_unlock() -> Result<()> {
        let tempdir = tempdir::TempDir::new("async-filelock")?;
        let path = tempdir.path().join("lock");
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .await?;
        file.lock_exclusive().await?;
        file.unlock().await?;
        Ok(())
    }

    #[test]
    fn lock_unlock() {
        task::block_on(run_lock_unlock()).unwrap();
    }
}
