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

use std::collections::{BTreeSet, BTreeMap};
use std::io::Error;
use std::path::Path;
use std::sync::{Arc, RwLock};
use bytes::BufMut;
use regex::Regex;

use super::{NFS, NFSStat64, NFSDirectory, NFSFile, NFSDirEntry, NFSEntryType, Result, Time};
use crate::get_parent_path_and_name;

fn get_rsize_from_url(url: &str) -> u32 {
    let re = Regex::new("[?&]rsize=(?<rsize>[1-9][0-9]*)").unwrap();
    re.captures(url)
        .map_or(
            1048576, // XXX: mimic libnfs default of 1 MiB
            |caps| u32::from_str_radix(&caps["rsize"], 10).unwrap(),
        )
}

#[derive(Debug)]
struct Mocks {
    dirs: BTreeSet<String>,
    files: BTreeMap<String, Vec<u8>>
}

#[derive(Debug)]
pub(super) struct NFS3 {
    mocks: Arc<RwLock<Mocks>>,
    rsize: u32,
}

impl NFS3 {
    pub(super) fn connect(url: String) -> Box<dyn NFS> {
        const MAXIMUM_READ_SIZE: u32 = 4194304; // XXX: according to libnfs, 4 MiB is the maximum
        const MINIMUM_READ_SIZE: u32 = 8192; // XXX: according to libnfs, 8 KiB is the minimum
        let rsize = get_rsize_from_url(&url).min(MAXIMUM_READ_SIZE).max(MINIMUM_READ_SIZE);
        let mut mocks = Mocks{dirs: BTreeSet::new(), files: BTreeMap::new()};
        let _ = mocks.dirs.insert("/first/".into());
        let _ = mocks.dirs.insert("/quatre/".into());
        let _ = mocks.files.insert("/3".into(), Vec::new());
        let _ = mocks.files.insert("/annar".into(), "In order to make sure that this file is exactly 123 bytes in size, I have written this text while watching its chars count.".as_bytes().to_vec());
        let _ = mocks.files.insert("/first/comment".into(), Vec::new());
        let _ = mocks.files.insert("/quatre/points".into(), Vec::new());
        Box::new(NFS3{mocks: Arc::new(RwLock::new(mocks)), rsize})
    }
}

impl NFS for NFS3 {
    fn access(&self, path: &str, mode: u32) -> Result<()> {
        let p = Path::new(path);
        if let Some(name) = p.file_name() {
            if (name != "3" && name != "quatre") || mode & 0o222 != 0 {
                return Ok(());
            }
        }
        Err(Error::new(std::io::ErrorKind::PermissionDenied, "permission denied"))
    }

    fn stat64(&self, path: &str) -> Result<NFSStat64> {
        let mocks = &self.mocks.read().unwrap();
        let size = if let Some(c) = mocks.files.get(&path.to_string()) {
            Some(c.len() as u64)
        } else {
            None
        };
        let mode = if size.is_some() {
            if path == "/3" { 0o444 } else { 0o664 }
        } else {
            if path == "/quatre" || path == "/quatre/" { 0o555 } else { 0o775 }
        };

        Ok(NFSStat64{
            dev: Default::default(),
            ino: Default::default(),
            mode,
            nlink: Default::default(),
            uid: Default::default(),
            gid: Default::default(),
            rdev: Default::default(),
            size: size.unwrap_or_default(),
            used: Default::default(),
            blksize: Default::default(),
            blocks: Default::default(),
            atime: 1658159058723,
            mtime: 1658159058723,
            ctime: 1658159058720,
            atime_nsec: Default::default(),
            mtime_nsec: Default::default(),
            ctime_nsec: Default::default(),
        })
    }

    fn lchmod(&self, _path: &str, _mode: u32) -> Result<()> {
        Ok(())
    }

    fn opendir(&mut self, path: &str) -> Result<Box<dyn NFSDirectory>> {
        let mocks = &self.mocks.read().unwrap();
        if path != "/" && mocks.dirs.get(&path.to_string()).is_none() {
            return Err(Error::new(std::io::ErrorKind::Other, "not found or not a directory"));
        }
        Ok(Box::new(NFSDirectory3{nfs: &*self, path: path.to_string(), entries: None, index: 0}))
    }

    fn mkdir(&self, path: &str, _mode: u32) -> Result<()> {
        let mocks = &mut self.mocks.write().unwrap();
        let _ = mocks.dirs.insert(path.to_string() + "/");
        Ok(())
    }

    fn create(&mut self, path: &str, _flags: u32, _mode: u32) -> Result<Box<dyn NFSFile>> {
        let mocks = &mut self.mocks.write().unwrap();
        let _ = mocks.files.insert(path.to_string(), Vec::new());
        Ok(Box::new(NFSFile3{nfs: &*self, path: path.to_string()}))
    }

    fn rmdir(&self, path: &str) -> Result<()> {
        let mocks = &mut self.mocks.write().unwrap();
        let path = path.to_string() + "/";
        let _ = mocks.dirs.remove(&path);
        Ok(())
    }

    fn unlink(&self, path: &str) -> Result<()> {
        let mocks = &mut self.mocks.write().unwrap();
        let _ = mocks.files.remove(&path.to_string());
        Ok(())
    }

    fn open(&mut self, path: &str, _flags: u32) -> Result<Box<dyn NFSFile>> {
        let mocks = &mut self.mocks.write().unwrap();
        if mocks.dirs.get(&path.to_string()).is_some() {
            return Err(Error::new(std::io::ErrorKind::Other, "is a directory"));
        }
        if mocks.files.get(&path.to_string()).is_none() {
            mocks.files.insert(path.to_string(), Vec::new());
        }
        Ok(Box::new(NFSFile3{nfs: &*self, path: path.to_string()}))
    }

    fn truncate(&self, path: &str, len: u64) -> Result<()> {
        let mocks = &mut self.mocks.write().unwrap();
        let contents = mocks.files.entry(path.to_string()).or_default();
        contents.resize(len as usize, 0);
        Ok(())
      }
}

#[derive(Debug)]
pub struct NFSDirectory3 {
    nfs: *const NFS3,
    path: String,
    entries: Option<Vec<NFSDirEntry>>,
    index: usize,
}

impl NFSDirectory for NFSDirectory3 {}

impl Iterator for NFSDirectory3 {
    type Item = Result<NFSDirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.entries.is_none() {
            let mut entries = Vec::new();
            let mocks = unsafe { &(*self.nfs).mocks.read().unwrap() };
            // XXX: technically should add '.' and '..' to entries but don't bother since they will be ignored anyway
            for (mock_file, content) in &mocks.files {
                let (parent_path, name) = get_parent_path_and_name(&mock_file);
                if parent_path == self.path {
                        let mode = if mock_file == "/3" { 0o444 } else { 0o664 };
                        entries.push(NFSDirEntry{
                        path: name,
                        inode: Default::default(),
                        d_type: NFSEntryType::File,
                        mode,
                        size: content.len() as u64,
                        used: Default::default(),
                        atime: Time{seconds: 1658159058, nseconds: 0},
                        mtime: Time{seconds: 1658159058, nseconds: 0},
                        ctime: Time{seconds: 1658159055, nseconds: 0},
                        uid: Default::default(),
                        gid: Default::default(),
                        nlink: Default::default(),
                        dev: Default::default(),
                        rdev: Default::default(),
                        blksize: Default::default(),
                        blocks: Default::default(),
                        atime_nsec: Default::default(),
                        mtime_nsec: Default::default(),
                        ctime_nsec: Default::default(),
                    });
                }
            }
            for mock_dir in mocks.dirs.iter().rev() {
                let (parent_path, name) = get_parent_path_and_name(&mock_dir.trim_end_matches('/').into());
                if parent_path == self.path {
                    let mode = if mock_dir == "/quatre/" { 0o555 } else { 0o775 };
                    entries.push(NFSDirEntry{
                        path: name,
                        inode: Default::default(),
                        d_type: NFSEntryType::Directory,
                        mode,
                        size: Default::default(),
                        used: Default::default(),
                        atime: Time{seconds: 1658159058, nseconds: 0},
                        mtime: Time{seconds: 1658159058, nseconds: 0},
                        ctime: Time{seconds: 1658159055, nseconds: 0},
                        uid: Default::default(),
                        gid: Default::default(),
                        nlink: Default::default(),
                        dev: Default::default(),
                        rdev: Default::default(),
                        blksize: Default::default(),
                        blocks: Default::default(),
                        atime_nsec: Default::default(),
                        mtime_nsec: Default::default(),
                        ctime_nsec: Default::default(),
                    });
                }
            }
            self.entries = Some(entries);
            self.index = 0;
        }

        let mut ret = None;
        if let Some(entries) = &self.entries {
            if self.index < entries.len() {
                ret = Some(Ok(entries[self.index].clone()));
                self.index += 1;
            } else {
                self.entries = None;
                self.index = 0;
            }
        }
        ret
    }
}

#[derive(Debug)]
pub struct NFSFile3 {
    nfs: *const NFS3,
    path: String,
}

impl NFSFile for NFSFile3 {
    fn fstat64(&self) -> Result<NFSStat64> {
        let mocks = unsafe { &(*self.nfs).mocks.read().unwrap() };
        let size = if let Some(c) = mocks.files.get(&self.path) {
            c.len() as u64
        } else {
            0
        };
        Ok(NFSStat64{
            dev: Default::default(),
            ino: Default::default(),
            mode: Default::default(),
            nlink: Default::default(),
            uid: Default::default(),
            gid: Default::default(),
            rdev: Default::default(),
            size,
            used: Default::default(),
            blksize: Default::default(),
            blocks: Default::default(),
            atime: 1658159058723,
            mtime: 1658159058723,
            ctime: 1658159058720,
            atime_nsec: Default::default(),
            mtime_nsec: Default::default(),
            ctime_nsec: Default::default(),
        })
    }

    fn get_max_read_size(&self) -> u64 {
        unsafe { (*self.nfs).rsize as u64 }
    }

    fn pread_into(&self, count: u32, offset: u64, buffer: &mut [u8]) -> Result<u32> {
        let mocks = unsafe { &(*self.nfs).mocks.read().unwrap() };
        let readlen = if let Some(content) = mocks.files.get(&self.path) {
            let (offset, count, len) = (offset as usize, count as usize, content.len());
            let start = if offset <= len { offset } else { len };
            let end = if start + count <= len { start + count } else { len };
            let data = content.get(start..end).unwrap_or_default();
            buffer.as_mut().put_slice(data);
            data.len() as u32
        } else {
            0
        };
        Ok(readlen)
    }

    fn pwrite(&self, buffer: &[u8], offset: u64) -> Result<u32> {
        let mocks = unsafe { &mut (*self.nfs).mocks.write().unwrap() };
        let contents = mocks.files.entry(self.path.clone()).or_default();
        let offset = offset as usize;
        let writelen = if contents.len() >= offset + buffer.len() {
            contents.splice(offset..(offset + buffer.len()), buffer.iter().cloned());
            buffer.len() as u32
        } else {
            let padlen = offset - contents.len();
            contents.resize(offset, 0);
            contents.append(&mut buffer.to_vec());
            (padlen + buffer.len()) as u32
        };
        Ok(writelen)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_rsize_from_url_returns_default() {
        let res = get_rsize_from_url("");
        assert_eq!(res, 1048576);
        let res = get_rsize_from_url("nfs://localhost/remote/rsize=1234");
        assert_eq!(res, 1048576);
        let res = get_rsize_from_url("nfs://localhost/remote/export?rsize=");
        assert_eq!(res, 1048576);
        let res = get_rsize_from_url("nfs://localhost/remote/export?rsize=&wsize=8192");
        assert_eq!(res, 1048576);
        let res = get_rsize_from_url("nfs://localhost/remote/export?wsize=8192&rsize=");
        assert_eq!(res, 1048576);
        let res = get_rsize_from_url("nfs://localhost/remote/export?wsize=8192&rsize=&uid=0");
        assert_eq!(res, 1048576);
        let res = get_rsize_from_url("nfs://localhost/remote/export?wsize=8192&rsize=0&uid=0");
        assert_eq!(res, 1048576);
        let res = get_rsize_from_url("nfs://localhost/remote/export?wsize=8192&rsize=def&uid=0");
        assert_eq!(res, 1048576);
    }

    #[test]
    fn get_rsize_from_url_works() {
        let res = get_rsize_from_url("nfs://localhost/remote/export?rsize=10240");
        assert_eq!(res, 10240);
        let res = get_rsize_from_url("nfs://localhost/remote/export?rsize=20480&wsize=8192");
        assert_eq!(res, 20480);
        let res = get_rsize_from_url("nfs://localhost/remote/export?wsize=8192&rsize=30720");
        assert_eq!(res, 30720);
        let res = get_rsize_from_url("nfs://localhost/remote/export?wsize=8192&rsize=40960&uid=666");
        assert_eq!(res, 40960);
    }

    #[test]
    fn mock_implementation_works() {
        let mut nfs = NFS3::connect(String::new());
        let res = nfs.opendir("/");
        assert!(res.is_ok(), "err = {}", res.unwrap_err());
        let dir = res.unwrap();
        let mut entries = Vec::new();
        for entry in dir {
            let res: Result<NFSDirEntry> = entry;
            if let Some(e) = res.ok() {
                entries.push((e.path, e.d_type));
            }
        }
        let expected_entries = vec![
            ("3".to_string(), NFSEntryType::File),
            ("annar".to_string(), NFSEntryType::File),
            ("quatre".to_string(), NFSEntryType::Directory),
            ("first".to_string(), NFSEntryType::Directory),
        ];
        assert_eq!(entries, expected_entries);
        let res = nfs.opendir("/first/");
        assert!(res.is_ok(), "err = {}", res.unwrap_err());
        let subdir = res.unwrap();
        let mut subentries = Vec::new();
        for subentry in subdir {
            let res: Result<NFSDirEntry> = subentry;
            if let Some(e) = res.ok() {
                subentries.push((e.path, e.d_type));
            }
        }
        let expected_subentries = vec![
            ("comment".to_string(), NFSEntryType::File),
        ];
        assert_eq!(subentries, expected_subentries);
    }
}
