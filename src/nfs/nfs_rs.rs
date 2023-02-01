use super::{NFS, NFSStat64, NFSDirectory, NFSFile, NFSDirEntry, Result, Time};

#[derive(Debug)]
pub(super) struct NFS3{
}

impl NFS3 {
    pub(super) fn connect(url: String) -> Box<dyn NFS> {
        unimplemented!()
    }
}
