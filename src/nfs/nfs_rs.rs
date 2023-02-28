use bytes::BufMut;
use nfs_rs::{Mount, parse_url_and_mount};
use std::io::Error;
use std::sync::{Arc, RwLock};

use super::{NFS, NFSStat64, NFSDirectory, NFSFile, NFSDirEntry, Result, Time};

const NFS_ENTRY_TYPE_DIR: u32 = 2;

#[derive(Debug)]
pub(super) struct NFS3{
    mount: Arc<RwLock<Box<dyn Mount>>>,
}

impl NFS3 {
    pub(super) fn connect(url: String) -> Box<dyn NFS> {
        let mount = parse_url_and_mount(url.as_str()).unwrap();
        Box::new(NFS3{mount: Arc::new(RwLock::new(mount))})
    }
}

impl NFS for NFS3 {
    fn access(&self, path: &str, mode: u32) -> Result<()> {
        let mount = self.mount.read().unwrap();
        let res = mount.access_path(path, mode)?;
        if res != mode {
            return Err(Error::new(std::io::ErrorKind::Other, "access denied"));
        }
        Ok(())
    }

    fn stat64(&self, path: &str) -> Result<NFSStat64> {
        let mount = self.mount.read().unwrap();
        mount.getattr_path(path).map(|fattr| NFSStat64{
            dev: ((fattr.spec_data[0] as u64) << 32) + (fattr.spec_data[1] as u64), // FIXME: verify that this is correct
            ino: fattr.fileid, // FIXME: verify that this is correct
            mode: fattr.file_mode.into(),
            nlink: fattr.nlink.into(),
            uid: fattr.uid.into(),
            gid: fattr.gid.into(),
            rdev: 0, // FIXME: verify that this is correct
            size: fattr.filesize,
            used: fattr.used,
            blksize: 0, // FIXME: verify that this is correct
            blocks: 0, // FIXME: verify that this is correct
            atime: fattr.atime.seconds.into(),
            mtime: fattr.mtime.seconds.into(),
            ctime: fattr.ctime.seconds.into(),
            atime_nsec: fattr.atime.nseconds.into(),
            mtime_nsec: fattr.mtime.nseconds.into(),
            ctime_nsec: fattr.ctime.nseconds.into(),
         })
    }

    fn lchmod(&self, path: &str, mode: u32) -> Result<()> {
        let mount = self.mount.read().unwrap();
        mount.setattr_path(path, true, Some(mode), None, None, None, None, None)
    }

    fn opendir(&mut self, path: &str) -> Result<Box<dyn NFSDirectory>> {
        let mount = self.mount.read().unwrap();
        let fh = mount.lookup(path)?;
        let attr = mount.getattr(&fh)?;
        if attr.type_ != NFS_ENTRY_TYPE_DIR {
            return Err(Error::new(std::io::ErrorKind::InvalidData, "not a directory"));
        }
        Ok(Box::new(NFSDirectory3{nfs: self, dir_fh: fh, entries: None, index: 0}))
    }

    fn mkdir(&self, path: &str, mode: u32) -> Result<()> {
        let mount = self.mount.read().unwrap();
        mount.mkdir_path(path, mode).map(|_| ())
    }

    fn create(&mut self, path: &str, _flags: u32, mode: u32) -> Result<Box<dyn NFSFile>> {
        let mount = self.mount.write().unwrap();
        let fh = mount.create_path(path, mode)?;
        Ok(Box::new(NFSFile3{nfs: self, fh}))
    }

    fn rmdir(&self, path: &str) -> Result<()> {
        let mount = self.mount.read().unwrap();
        mount.rmdir_path(path)
    }

    fn unlink(&self, path: &str) -> Result<()> {
        let mount = self.mount.read().unwrap();
        mount.remove_path(path)
    }

    fn open(&mut self, path: &str, _flags: u32) -> Result<Box<dyn NFSFile>> {
        let mount = self.mount.write().unwrap();
        let fh = mount.lookup(path)?;
        let attr = mount.getattr(&fh)?;
        if attr.type_ == NFS_ENTRY_TYPE_DIR {
            return Err(Error::new(std::io::ErrorKind::InvalidData, "is a directory"));
        }
        Ok(Box::new(NFSFile3{nfs: self, fh}))
    }

    fn truncate(&self, path: &str, len: u64) -> Result<()> {
        let mount = self.mount.read().unwrap();
        mount.setattr_path(path, true, None, None, None, Some(len), None, None)
    }
}

#[derive(Debug)]
pub struct NFSDirectory3 {
    nfs: *const NFS3,
    dir_fh: Vec<u8>,
    entries: Option<Vec<NFSDirEntry>>,
    index: usize,
}

impl NFSDirectory for NFSDirectory3 {}

impl Iterator for NFSDirectory3 {
    type Item = Result<NFSDirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.entries.is_none() {
            let mount = unsafe { (*self.nfs).mount.read().unwrap() };
            let res = mount.readdirplus(&self.dir_fh);
            if res.is_err() {
                return Some(Err(res.unwrap_err()));
            }
            let mut entries = Vec::new();
            for entry in res.unwrap() {
                let mut attr = entry.attr.unwrap_or_default();
                if entry.file_name == "." || entry.file_name == ".." {
                    // go-nfs (at least -- maybe others as well) does not return attributes for '.' and '..' so we
                    // want to explicitly set type_ to 2 (directory) to ensure correct treatment of these entries
                    attr.type_ = 2;
                }
                entries.push(NFSDirEntry{
                    path: entry.file_name.clone(), // FIXME: verify that this is correct
                    inode: entry.fileid, // FIXME: verify that this is correct
                    d_type: attr.type_.into(),
                    mode: attr.file_mode,
                    size: attr.filesize,
                    used: attr.used,
                    uid: attr.uid,
                    gid: attr.gid,
                    nlink: attr.nlink,
                    dev: ((attr.spec_data[0] as u64) << 32) + (attr.spec_data[1] as u64), // FIXME: verify that this is correct
                    rdev: 0, // FIXME: verify that this is correct
                    blksize: 0, // FIXME: verify that this is correct
                    blocks: 0, // FIXME: verify that this is correct
                    atime: Time{seconds: attr.atime.seconds, nseconds: attr.atime.nseconds},
                    mtime: Time{seconds: attr.mtime.seconds, nseconds: attr.mtime.nseconds},
                    ctime: Time{seconds: attr.ctime.seconds, nseconds: attr.ctime.nseconds},
                    atime_nsec: attr.atime.nseconds,
                    mtime_nsec: attr.mtime.nseconds,
                    ctime_nsec: attr.ctime.nseconds,
                })
            }
            self.entries = Some(entries);
            self.index = 0;
        }
        let entries = self.entries.as_ref().unwrap();
        if self.index >= entries.len() {
            self.entries = None;
            self.index = 0;
            return None;
        }
        self.index += 1;
        Some(Ok(entries[self.index-1].clone()))
    }
}

#[derive(Debug)]
pub struct NFSFile3 {
    nfs: *const NFS3,
    fh: Vec<u8>,
}

impl NFSFile for NFSFile3 {
    fn fstat64(&self) -> Result<NFSStat64> {
        let mount = unsafe { (*self.nfs).mount.read().unwrap() };
        mount.getattr(&self.fh).map(|fattr| NFSStat64{
            dev: ((fattr.spec_data[0] as u64) << 32) + (fattr.spec_data[1] as u64), // FIXME: verify that this is correct
            ino: fattr.fileid, // FIXME: verify that this is correct
            mode: fattr.file_mode.into(),
            nlink: fattr.nlink.into(),
            uid: fattr.uid.into(),
            gid: fattr.gid.into(),
            rdev: 0, // FIXME: verify that this is correct
            size: fattr.filesize,
            used: fattr.used,
            blksize: 0, // FIXME: verify that this is correct
            blocks: 0, // FIXME: verify that this is correct
            atime: fattr.atime.seconds.into(),
            mtime: fattr.mtime.seconds.into(),
            ctime: fattr.ctime.seconds.into(),
            atime_nsec: fattr.atime.nseconds.into(),
            mtime_nsec: fattr.mtime.nseconds.into(),
            ctime_nsec: fattr.ctime.nseconds.into(),
         })
    }

    fn pread_into(&self, count: u32, offset: u64, buffer: &mut [u8]) -> Result<u32> {
        let mount = unsafe { (*self.nfs).mount.read().unwrap() };
        let res = mount.read(&self.fh, offset, count)?;
        buffer.as_mut().put_slice(res.as_slice());
        Ok(res.len() as u32)
    }

    fn pwrite(&self, buffer: &[u8], offset: u64) -> Result<u32> {
        let mount = unsafe { (*self.nfs).mount.write().unwrap() };
        mount.write(&self.fh, offset, &buffer.to_vec())
    }
}
