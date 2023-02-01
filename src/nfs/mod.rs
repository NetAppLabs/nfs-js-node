use std::io::Result;
use std::fmt::Debug;

mod libnfs;
mod nfs_rs;
mod mock;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Time {
    pub seconds: u32,
    pub nseconds: u32,
}

pub trait NFS: Debug + Send + Sync {
    fn access(&self, path: &str, mode: u32) -> Result<()>;
    fn stat64(&self, path: &str) -> Result<NFSStat64>;
    fn lchmod(&self, path: &str, mode: u32) -> Result<()>;
    fn opendir(&mut self, path: &str) -> Result<Box<dyn NFSDirectory>>;
    fn mkdir(&self, path: &str, mode: u32) -> Result<()>;
    fn create(&mut self, path: &str, flags: u32, mode: u32) -> Result<Box<dyn NFSFile>>;
    fn rmdir(&self, path: &str) -> Result<()>;
    fn unlink(&self, path: &str) -> Result<()>;
    fn open(&mut self, path: &str, flags: u32) -> Result<Box<dyn NFSFile>>;
    fn truncate(&self, path: &str, len: u64) -> Result<()>;
}

pub trait NFSDirectory: Debug + Iterator<Item = Result<NFSDirEntry>> {}

pub trait NFSFile: Debug {
    fn fstat64(&self) -> Result<NFSStat64>;
    fn pread_into(&self, count: u32, offset: u64, buffer: &mut [u8]) -> Result<u32>;
    fn pwrite(&self, buffer: &[u8], offset: u64) -> Result<u32>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum NFSEntryType {
    Block,
    Character,
    Directory,
    File,
    NamedPipe,
    Symlink,
    Socket,
}

impl From<u32> for NFSEntryType {
    fn from(val: u32) -> Self {
        match val {
            0 => Self::Block,
            1 => Self::Character,
            2 => Self::Directory,
            3 => Self::File,
            4 => Self::NamedPipe,
            5 => Self::Symlink,
            6 => Self::Socket,
            _ => panic!("invalid entry type"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NFSDirEntry {
    pub path: String,
    pub inode: u64,
    pub d_type: NFSEntryType,
    pub mode: u32,
    pub size: u64,
    pub used: u64,
    pub atime: Time,
    pub mtime: Time,
    pub ctime: Time,
    pub uid: u32,
    pub gid: u32,
    pub nlink: u32,
    pub dev: u64,
    pub rdev: u64,
    pub blksize: u64,
    pub blocks: u64,
    pub atime_nsec: u32,
    pub mtime_nsec: u32,
    pub ctime_nsec: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct NFSStat64 {
  pub dev: u64,
  pub ino: u64,
  pub mode: u64,
  pub nlink: u64,
  pub uid: u64,
  pub gid: u64,
  pub rdev: u64,
  pub size: u64,
  pub used: u64,
  pub blksize: u64,
  pub blocks: u64,
  pub atime: u64,
  pub mtime: u64,
  pub ctime: u64,
  pub atime_nsec: u64,
  pub mtime_nsec: u64,
  pub ctime_nsec: u64,
}

pub(crate) fn connect(url: String) -> Box<dyn NFS> {
    if std::env::var("TEST_USING_MOCKS").is_ok() {
        mock::NFS3::connect(url)
    } else if std::env::var("TEST_USING_PURE_RUST").is_ok() {
        nfs_rs::NFS3::connect(url)
    } else {
        libnfs::NFS3::connect(url)
    }
}
