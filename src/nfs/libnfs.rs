// Copyright 2025 NetApp Inc. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

use core::fmt::Debug;
use std::path::Path;
use std::sync::{Arc, RwLock};
use nix::sys::stat::Mode;
use nix::fcntl::OFlag;
use libnfs::Nfs;

use super::{NFS, NFSStat64, NFSDirectory, NFSFile, NFSDirEntry, Result, Time};

pub(super) struct NFS3 {
    nfs: Arc<RwLock<Nfs>>,
}

impl NFS3 {
    pub(super) fn connect(url: String) -> Box<dyn NFS> {
        let mut nfs = Nfs::new().unwrap();
        let _ = nfs.parse_url_mount(url.as_str()).unwrap();
        Box::new(NFS3{nfs: Arc::new(RwLock::new(nfs))})
    }
}

impl Debug for NFS3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NFS3").finish()
    }
}

impl NFS for NFS3 {
    fn access(&self, path: &str, mode: u32) -> Result<()> {
        let my_nfs = self.nfs.write().unwrap();
        my_nfs.access(Path::new(path), mode as i32).map(|_| ())
    }

    fn stat64(&self, path: &str) -> Result<NFSStat64> {
        let my_nfs = self.nfs.write().unwrap();
        my_nfs.stat64(Path::new(path)).map(|res| NFSStat64{
            dev: res.nfs_dev,
            ino: res.nfs_ino,
            mode: res.nfs_mode,
            nlink: res.nfs_nlink,
            uid: res.nfs_uid,
            gid: res.nfs_gid,
            rdev: res.nfs_rdev,
            size: res.nfs_size,
            used: res.nfs_used,
            blksize: res.nfs_blksize,
            blocks: res.nfs_blocks,
            atime: res.nfs_atime,
            mtime: res.nfs_mtime,
            ctime: res.nfs_ctime,
            atime_nsec: res.nfs_atime_nsec,
            mtime_nsec: res.nfs_mtime_nsec,
            ctime_nsec: res.nfs_ctime_nsec,
        })
    }

    fn lchmod(&self, path: &str, mode: u32) -> Result<()> {
        let my_nfs = self.nfs.write().unwrap();
        my_nfs.lchmod(Path::new(path), Mode::from_bits_truncate((mode as u16).into()))
    }

    fn opendir(&mut self, path: &str) -> Result<Box<dyn NFSDirectory>> {
        let mut my_nfs = self.nfs.write().unwrap();
        let dir = my_nfs.opendir(Path::new(path))?;
        Ok(Box::new(NFSDirectory3{dir}))
    }

    fn mkdir(&self, path: &str, _mode: u32) -> Result<()> { // FIXME: mode
        let my_nfs = self.nfs.write().unwrap();
        my_nfs.mkdir(Path::new(path))
    }

    fn create(&mut self, path: &str, flags: u32, mode: u32) -> Result<Box<dyn NFSFile>> {
        let mut my_nfs = self.nfs.write().unwrap();
        let file = my_nfs.create(Path::new(path), OFlag::from_bits_truncate(flags as i32), Mode::from_bits_truncate((mode as u16).into()))?;
        Ok(Box::new(NFSFile3{file}))
    }

    fn rmdir(&self, path: &str) -> Result<()> {
        let my_nfs = self.nfs.write().unwrap();
        my_nfs.rmdir(Path::new(path))
    }

    fn unlink(&self, path: &str) -> Result<()> {
        let my_nfs = self.nfs.write().unwrap();
        my_nfs.unlink(Path::new(path))
    }

    fn open(&mut self, path: &str, flags: u32) -> Result<Box<dyn NFSFile>> {
        let mut my_nfs = self.nfs.write().unwrap();
        let file = my_nfs.open(Path::new(path), OFlag::from_bits_truncate(flags as i32))?;
        Ok(Box::new(NFSFile3{file}))
    }

    fn truncate(&self, path: &str, len: u64) -> Result<()> {
        let my_nfs = self.nfs.write().unwrap();
        my_nfs.truncate(Path::new(path), len)
    }
}

pub struct NFSDirectory3 {
    dir: libnfs::NfsDirectory,
}

impl NFSDirectory for NFSDirectory3 {}

impl Debug for NFSDirectory3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NFSDirectory3").finish()
    }
}

impl Iterator for NFSDirectory3 {
    type Item = Result<NFSDirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        self.dir.next().map(|res| res.map(|entry| NFSDirEntry{
            path: entry.path.into_os_string().into_string().unwrap(),
            inode: entry.inode,
            d_type: (entry.d_type as u32).into(),
            mode: entry.mode.bits() as u32,
            size: entry.size,
            used: entry.used,
            atime: Time{seconds: entry.atime.tv_sec as u32, nseconds: entry.atime_nsec},
            mtime: Time{seconds: entry.mtime.tv_sec as u32, nseconds: entry.mtime_nsec},
            ctime: Time{seconds: entry.ctime.tv_sec as u32, nseconds: entry.ctime_nsec},
            uid: entry.uid,
            gid: entry.gid,
            nlink: entry.nlink,
            dev: entry.dev,
            rdev: entry.rdev,
            blksize: entry.blksize,
            blocks: entry.blocks,
            atime_nsec: entry.atime_nsec,
            mtime_nsec: entry.mtime_nsec,
            ctime_nsec: entry.ctime_nsec,
        }))
    }
}

pub struct NFSFile3 {
    file: libnfs::NfsFile,
}

impl Debug for NFSFile3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NFSFile3").finish()
    }
}

impl NFSFile for NFSFile3 {
    fn fstat64(&self) -> Result<NFSStat64> {
        self.file.fstat64().map(|res| NFSStat64{
            dev: res.nfs_dev,
            ino: res.nfs_ino,
            mode: res.nfs_mode,
            nlink: res.nfs_nlink,
            uid: res.nfs_uid,
            gid: res.nfs_gid,
            rdev: res.nfs_rdev,
            size: res.nfs_size,
            used: res.nfs_used,
            blksize: res.nfs_blksize,
            blocks: res.nfs_blocks,
            atime: res.nfs_atime,
            mtime: res.nfs_mtime,
            ctime: res.nfs_ctime,
            atime_nsec: res.nfs_atime_nsec,
            mtime_nsec: res.nfs_mtime_nsec,
            ctime_nsec: res.nfs_ctime_nsec,
        })
    }

    fn get_max_read_size(&self) -> u64 {
        self.file.get_max_read_size()
    }

    fn pread_into(&self, count: u32, offset: u64, buffer: &mut [u8]) -> Result<u32> {
        self.file.pread_into(count as u64, offset, buffer).map(|res| res as u32)
    }

    fn pwrite(&self, buffer: &[u8], offset: u64) -> Result<u32> {
        self.file.pwrite(buffer, offset).map(|res| res as u32)
    }
}
