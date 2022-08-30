#![deny(clippy::all)]

use libnfs::Nfs;
use napi::{JsArrayBuffer, JsDataView, JsString, JsTypedArray, NapiRaw, bindgen_prelude::*};
use napi_derive::napi;
use nix::sys::stat::Mode;
use send_wrapper::SendWrapper;
use std::{path::Path, collections::{BTreeMap, BTreeSet}, sync::{Arc, RwLock}};

/*

See https://wicg.github.io/file-system-access/
And https://developer.mozilla.org/en-US/docs/Web/API/File_System_Access_API
and https://web.dev/file-system-access/


// Example use:
const nfs_url = "nfs://1.2.3.4/export?vers=3";
const rootHandle = new NFSDirectoryHandle(nfs_url);

for await (const [name, entry] of rootHandle) {
  console.log("FileName: ", name, "Entry: ", entry);
}

const fileHandle = await rootHandle.getFileHandle("testfile.txt", { create: true });
const wfs = await fileHandle.createWritable({ keepExistingData: false });
await wfs.write("Hello from Javascript");
await wfs.close();

// Interface definition

type FileSystemHandlePermissionMode = "read" | "readwrite";
type FileSystemHandleKind = "directory" | "file";

interface FileSystemHandlePermissionDescriptor {
    mode: FileSystemHandlePermissionMode;
}

interface FileSystemHandle {
    readonly kind: FileSystemHandleKind;
    readonly name: string;
    isSameEntry(other: FileSystemHandle): boolean;
    queryPermission(perm: FileSystemHandlePermissionDescriptor): Promise<String>;
    requestPermission(perm: FileSystemHandlePermissionDescriptor): Promise<String>;
}

interface FileSystemGetDirectoryOptions {
    create: boolean;
}

interface FileSystemGetFileOptions {
    create: boolean;
}

interface FileSystemRemoveOptions {
    recursive: boolean;
}

interface FileSystemDirectoryHandle extends FileSystemHandle {
    readonly kind: 'directory';
    getDirectoryHandle(name: string, options?: FileSystemGetDirectoryOptions): Promise<FileSystemDirectoryHandle>;
    getFileHandle(name: string, options?: FileSystemGetFileOptions): Promise<FileSystemFileHandle>;
    removeEntry(name: string, options?: FileSystemRemoveOptions): Promise<void>;
    resolve(possibleDescendant: FileSystemHandle): Promise<string[] | null>;
    keys(): AsyncIterableIterator<string>;
    values(): AsyncIterableIterator<FileSystemDirectoryHandle | FileSystemFileHandle>;
    entries(): AsyncIterableIterator<[string, FileSystemDirectoryHandle | FileSystemFileHandle]>;
    [Symbol.asyncIterator]: FileSystemDirectoryHandle['entries'];
}

interface FileSystemCreateWritableOptions {
    keepExistingData: boolean;
}

interface FileSystemFileHandle extends FileSystemHandle {
    readonly kind: "file";
    getFile(): Promise<File>;
    createWritable(options?: FileSystemCreateWritableOptions): Promise<FileSystemWritableFileStream>;
}

interface FileSystemWritableFileStream extends WritableStream {
    readonly locked: true;                    // from WritableStream
    abort(reason: string): Promise<string>;   // from WritableStream
    close(): Promise<void>;                   // from WritableStream
    getWriter(): WritableStreamDefaultWriter; // from WritableStream
    write(data: ArrayBuffer | TypedArray | DataView | Blob | String | string | {type: 'write' | 'seek' | 'truncate', data?: ArrayBuffer | TypedArray | DataView | Blob | String | string, position?: number, size?: number}): Promise<void>;
    seek(position: number): Promise<void>;
    truncate(size: number): Promise<void>;
}

interface File extends Blob {
    readonly lastModified: number;
    readonly name: string;
}

interface Blob {
    readonly size: number;
    readonly type: string;
    arrayBuffer(): Promise<ArrayBuffer>;
    slice(start?: number, end?: number, contentType?: string): Blob;
    stream(): ReadableStream<Uint8Array>;
    text(): Promise<string>;
}

*/

const FIELD_KIND: &str = "kind";
const FIELD_NAME: &str = "name";
const FIELD_PATH: &str = "path";
const FIELD_DATA: &str = "data";
const FIELD_TYPE: &str = "type";
const FIELD_SIZE: &str = "size";
const FIELD_CLOSE: &str = "close";
const FIELD_LENGTH: &str = "length";
const FIELD_BUFFER: &str = "buffer";
const FIELD_ENQUEUE: &str = "enqueue";
const FIELD_POSITION: &str = "position";
const FIELD_SUBSTRING: &str = "substring";
const FIELD_BYTE_LENGTH: &str = "byteLength";

const KIND_FILE: &str = "file";
const KIND_DIRECTORY: &str = "directory";

const PERM_READ: &str = "read";
const PERM_READWRITE: &str = "readwrite";

const PERM_STATE_GRANTED: &str = "granted";
const PERM_STATE_DENIED: &str = "denied";
const _PERM_STATE_PROMPT: &str = "prompt";

const WRITE_TYPE_WRITE: &str = "write";
const WRITE_TYPE_SEEK: &str = "seek";
const WRITE_TYPE_TRUNCATE: &str = "truncate";

const DIR_ROOT: &str = "/";
const DIR_CURRENT: &str = ".";
const DIR_PARENT: &str = "..";

const MIME_TYPE_UNKNOWN: &str = "unknown";

const JS_TYPE_BLOB: &str = "Blob";
const JS_TYPE_READABLE_STREAM: &str = "ReadableStream";
const JS_TYPE_WRITABLE_STREAM: &str = "WritableStream";
const JS_TYPE_WRITABLE_STREAM_DEFAULT_WRITER: &str = "WritableStreamDefaultWriter";

const READABLE_STREAM_SOURCE_TYPE_BYTES: &str = "bytes";

struct Mocks {
  dirs: BTreeSet<String>,
  files: BTreeMap<String, Vec<u8>>
}

static mut MOCKS: Option<Arc<RwLock<Mocks>>> = None;

fn get_mocks() -> &'static mut Arc<RwLock<Mocks>> {
  unsafe { MOCKS.get_or_insert(Arc::new(RwLock::new(Mocks{dirs: BTreeSet::new(), files: BTreeMap::new()}))) }
}

#[napi(iterator)]
struct JsNfsDirectoryHandleEntries {
  #[napi(js_name="[Symbol.asyncIterator]", ts_type="AsyncIterableIterator<[string, JsNfsDirectoryHandle | JsNfsFileHandle]>")]
  pub _sym: bool, // unused fake member, just to so that generated JsNfsDirectoryHandleEntries class specifies `[Symbol.asyncIterator]: AsyncIterableIterator<[string, JsNfsDirectoryHandle | JsNfsFileHandle]>`
  env: SendWrapper<Env>,
  entries: Vec<JsNfsHandle>,
  count: usize
}

impl Generator for JsNfsDirectoryHandleEntries {

  type Yield = Vec<Unknown>;

  type Next = ();

  type Return = ();

  fn next(&mut self, _: Option<Self::Next>) -> Option<Self::Yield> {
    if self.entries.len() <= self.count {
      return None;
    }
    let entry = &self.entries[self.count];
    let mut res = Vec::new();
    res.push(self.env.create_string(entry.name.as_str()).ok()?.into_unknown());
    match entry.kind.as_str() {
      KIND_DIRECTORY => unsafe { res.push(Unknown::from_napi_value(self.env.raw(), JsNfsDirectoryHandle::from(entry.to_owned()).into_instance(*self.env).ok()?.raw()).ok()?) },
      _ => unsafe { res.push(Unknown::from_napi_value(self.env.raw(), JsNfsFileHandle::from(entry.to_owned()).into_instance(*self.env).ok()?.raw()).ok()?) },
    };
    self.count += 1;
    Some(res)
  }
}

#[napi(iterator)]
struct JsNfsDirectoryHandleKeys {
  #[napi(js_name="[Symbol.asyncIterator]", ts_type="AsyncIterableIterator<string>")]
  pub _sym: bool, // unused fake member, just to so that generated JsNfsDirectoryHandleKeys class specifies `[Symbol.asyncIterator]: AsyncIterableIterator<string>`
  entries: Vec<JsNfsHandle>,
  count: usize
}

impl Generator for JsNfsDirectoryHandleKeys {

  type Yield = String;

  type Next = ();

  type Return = ();

  fn next(&mut self, _: Option<Self::Next>) -> Option<Self::Yield> {
    if self.entries.len() <= self.count {
      return None;
    }
    let entry = &self.entries[self.count];
    let res = entry.name.to_owned();
    self.count += 1;
    Some(res)
  }
}

#[napi(iterator)]
struct JsNfsDirectoryHandleValues {
  #[napi(js_name="[Symbol.asyncIterator]", ts_type="AsyncIterableIterator<JsNfsDirectoryHandle | JsNfsFileHandle>")]
  pub _sym: bool, // unused fake member, just to so that generated JsNfsDirectoryHandleValues class specifies `[Symbol.asyncIterator]: AsyncIterableIterator<JsNfsDirectoryHandle | JsNfsFileHandle>`
  entries: Vec<JsNfsHandle>,
  count: usize
}

impl Generator for JsNfsDirectoryHandleValues {

  type Yield = Either<JsNfsDirectoryHandle, JsNfsFileHandle>;

  type Next = ();

  type Return = ();

  fn next(&mut self, _: Option<Self::Next>) -> Option<Self::Yield> {
    if self.entries.len() <= self.count {
      return None;
    }
    let entry = &self.entries[self.count];
    let res = match entry.kind.as_str() {
      KIND_DIRECTORY => Either::A(JsNfsDirectoryHandle::from(entry.to_owned())),
      _ => Either::B(JsNfsFileHandle::from(entry.to_owned()))
    };
    self.count += 1;
    Some(res)
  }
}

#[napi(object)]
struct JsNfsHandlePermissionDescriptor {
  #[napi(ts_type="'read' | 'readwrite'")]
  pub mode: String
}

impl JsNfsHandlePermissionDescriptor {

  fn to_mode(&self, kind: &str) -> Mode {
    match (kind, self.mode.as_str()) {
      (KIND_DIRECTORY, PERM_READWRITE) => Mode::S_IRWXU | Mode::S_IRWXG, // 770
      (KIND_DIRECTORY, PERM_READ) => Mode::S_IRUSR | Mode::S_IXUSR | Mode::S_IRGRP | Mode::S_IXGRP, // 550
      (KIND_FILE, PERM_READWRITE) => Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IWGRP, // 660
      _ => Mode::S_IRUSR | Mode::S_IRGRP // 440
    }
  }

  fn to_u64(&self, kind: &str) -> u64 {
    self.to_mode(kind).bits().into()
  }
}

#[napi(object)]
struct JsNfsGetDirectoryOptions {
  pub create: bool
}

impl Default for JsNfsGetDirectoryOptions {

  fn default() -> Self {
    Self{create: Default::default()}
  }
}

#[napi(object)]
struct JsNfsGetFileOptions {
  pub create: bool
}

impl Default for JsNfsGetFileOptions {

  fn default() -> Self {
    Self{create: Default::default()}
  }
}

#[napi(object)]
struct JsNfsRemoveOptions {
  pub recursive: bool
}

impl Default for JsNfsRemoveOptions {

  fn default() -> Self {
    Self{recursive: Default::default()}
  }
}

#[napi(object)]
struct JsNfsCreateWritableOptions {
  pub keep_existing_data: bool
}

impl Default for JsNfsCreateWritableOptions {

  fn default() -> Self {
    Self{keep_existing_data: Default::default()}
  }
}

#[derive(Clone)]
#[napi]
struct JsNfsHandle {
  nfs: Option<Nfs>,
  path: String,
  #[napi(readonly, ts_type="'directory' | 'file'")]
  pub kind: String,
  #[napi(readonly)]
  pub name: String
}

#[napi]
impl JsNfsHandle {

  fn get_nfs(url: String) -> Option<Nfs> {
    std::env::var("TEST_USING_MOCKS").err().and_then(|_| {
      let mut nfs = Nfs::new().unwrap();
      let _ = nfs.parse_url_mount(url.as_str()).unwrap();
      Some(nfs)
    })
  }

  pub fn open(url: String) -> Self {
    if let Some(nfs) = Self::get_nfs(url) {
      Self{nfs: Some(nfs), path: DIR_ROOT.into(), kind: KIND_DIRECTORY.into(), name: DIR_ROOT.into()}
    } else {
      let mut mocks = get_mocks().write().unwrap();
      let _ = mocks.dirs.insert("/first/".into());
      let _ = mocks.dirs.insert("/quatre/".into());
      let _ = mocks.files.insert("/3".into(), Vec::new());
      let _ = mocks.files.insert("/annar".into(), "In order to make sure that this file is exactly 123 bytes in size, I have written this text while watching its chars count.".as_bytes().to_vec());
      let _ = mocks.files.insert("/first/comment".into(), Vec::new());
      let _ = mocks.files.insert("/quatre/points".into(), Vec::new());
      Self{nfs: None, path: DIR_ROOT.into(), kind: KIND_DIRECTORY.into(), name: DIR_ROOT.into()}
    }
  }

  fn is_same(&self, other: &JsNfsHandle) -> bool {
    other.kind == self.kind && other.name == self.name && (other.path.is_empty() || self.path.is_empty() || other.path == self.path)
  }

  #[napi]
  pub fn is_same_entry(&self, other: &JsNfsHandle) -> Result<bool> {
    Ok(self.is_same(other))
  }

  #[napi]
  pub async fn query_permission(&self, perm: JsNfsHandlePermissionDescriptor) -> Result<String> {
    if let Some(nfs) = &self.nfs {
      let my_nfs = nfs.to_owned();
      let nfs_stat = my_nfs.stat64(Path::new(self.path.as_str()))?;
      let perm_u64 = perm.to_u64(self.kind.as_str());
      if nfs_stat.nfs_mode & perm_u64 == perm_u64 {
        return Ok(PERM_STATE_GRANTED.into());
      }
    }
    if self.nfs.is_none() && ((self.name != "3" && self.name != "quatre") || perm.mode != PERM_READWRITE) {
      return Ok(PERM_STATE_GRANTED.into());
    }
    Ok(PERM_STATE_DENIED.into())
  }

  #[napi]
  pub async fn request_permission(&self, perm: JsNfsHandlePermissionDescriptor) -> Result<String> {
    if let Some(nfs) = &self.nfs {
      let my_nfs = nfs.to_owned();
      let nfs_stat = my_nfs.stat64(Path::new(self.path.as_str()))?;
      let perm_u64 = perm.to_u64(self.kind.as_str());
      if nfs_stat.nfs_mode & perm_u64 == perm_u64 {
        return Ok(PERM_STATE_GRANTED.into());
      }
      let mode = perm.to_mode(self.kind.as_str()).union(Mode::from_bits_truncate(nfs_stat.nfs_mode as u16));
      if !my_nfs.lchmod(Path::new(self.name.as_str()), mode).is_ok() {
        return Ok(PERM_STATE_DENIED.into());
      }
    }
    self.query_permission(perm).await
  }
}

impl FromNapiValue for JsNfsHandle {

  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    Self::from_napi_ref(env, napi_val)
      .map_or_else(
      |err| {
        if err.status != Status::InvalidArg || err.reason != "Failed to recover `JsNfsHandle` type from napi value" {
          return Err(err);
        }
        let obj = Object::from_napi_value(env, napi_val)?;
        let kind = obj.get::<&str, &str>(FIELD_KIND)?.unwrap_or_default().into();
        let name = obj.get::<&str, &str>(FIELD_NAME)?.unwrap_or_default().into();
        let path = obj.get::<&str, &str>(FIELD_PATH)?.unwrap_or_default().into();
        Ok(Self{nfs: None, path, kind, name})
      },
      |handle| Ok(handle.to_owned())
    )
  }
}

#[napi]
struct JsNfsDirectoryHandle {
  handle: JsNfsHandle,
  #[napi(js_name="[Symbol.asyncIterator]", ts_type="JsNfsDirectoryHandle['entries']")]
  pub _sym: bool, // unused fake member, just to so that generated JsNfsDirectoryHandle class specifies `[Symbol.asyncIterator]: JsNfsDirectoryHandle['entries']`
  #[napi(readonly, ts_type="'directory'")]
  pub kind: String,
  #[napi(readonly)]
  pub name: String
}

#[napi]
impl JsNfsDirectoryHandle {

  #[napi(constructor)]
  pub fn open(url: String) -> Self {
    JsNfsHandle::open(url).into()
  }

  #[napi]
  pub fn to_handle(&self) -> Result<JsNfsHandle> {
    Ok(self.handle.clone())
  }

  #[napi]
  pub fn is_same_entry(&self, other: &JsNfsHandle) -> Result<bool> {
    self.handle.is_same_entry(other)
  }

  #[napi]
  pub async fn query_permission(&self, perm: JsNfsHandlePermissionDescriptor) -> Result<String> {
    self.handle.query_permission(perm).await
  }

  #[napi]
  pub async fn request_permission(&self, perm: JsNfsHandlePermissionDescriptor) -> Result<String> {
    self.handle.request_permission(perm).await
  }

  fn nfs_entries(&self) -> Result<Vec<JsNfsHandle>> {
    let mut entries = Vec::new();
    if let Some(nfs) = &self.handle.nfs {
      let mut my_nfs = nfs.to_owned();
      let dir = my_nfs.opendir(Path::new(self.handle.path.as_str()))?;
      for entry in dir {
        if let Some(e) = entry.ok() {
          let name = e.path.into_os_string().into_string().unwrap();
          let (kind, path) = match e.d_type {
            libnfs::EntryType::Directory => (KIND_DIRECTORY.into(), format_dir_path(&self.handle.path, &name)),
            _ => (KIND_FILE.into(), format_file_path(&self.handle.path, &name))
          };
          entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path, kind, name});
        }
      }
    } else {
      let mocks = get_mocks().read().unwrap();
      for (mock_file, _) in &mocks.files {
        let (parent_path, name) = get_parent_path_and_name(&mock_file);
        if parent_path == self.handle.path {
          entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path: mock_file.clone(), kind: KIND_FILE.into(), name});
        }
      }
      for mock_dir in mocks.dirs.iter().rev() {
        let (parent_path, name) = get_parent_path_and_name(&mock_dir.trim_end_matches('/').into());
        if parent_path == self.handle.path {
          entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path: mock_dir.clone(), kind: KIND_DIRECTORY.into(), name});
        }
      }
      entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path: format_dir_path(&self.handle.path, &"..".into()), kind: KIND_DIRECTORY.into(), name: DIR_PARENT.into()});
      entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path: format_dir_path(&self.handle.path, &".".into()), kind: KIND_DIRECTORY.into(), name: DIR_CURRENT.into()});
    }
    Ok(entries)
  }

  #[napi(iterator, ts_return_type="AsyncIterableIterator<[string, JsNfsDirectoryHandle | JsNfsFileHandle]>")]
  pub fn entries(&self, env: Env) -> Result<JsNfsDirectoryHandleEntries> {
    Ok(JsNfsDirectoryHandleEntries{entries: self.nfs_entries()?, env: SendWrapper::new(env), count: 0, _sym: false})
  }

  #[napi(iterator, ts_return_type="AsyncIterableIterator<string>")]
  pub fn keys(&self) -> Result<JsNfsDirectoryHandleKeys> {
    Ok(JsNfsDirectoryHandleKeys{entries: self.nfs_entries()?, count: 0, _sym: false})
  }

  #[napi(iterator, ts_return_type="AsyncIterableIterator<JsNfsDirectoryHandle | JsNfsFileHandle>")]
  pub fn values(&self) -> Result<JsNfsDirectoryHandleValues> {
    Ok(JsNfsDirectoryHandleValues{entries: self.nfs_entries()?, count: 0, _sym: false})
  }

  #[napi]
  pub async fn get_directory_handle(&mut self, name: String, #[napi(ts_arg_type="JsNfsGetDirectoryOptions")] options: Option<JsNfsGetDirectoryOptions>) -> Result<JsNfsDirectoryHandle> {
    for entry in self.nfs_entries()? {
      if entry.kind == KIND_DIRECTORY && entry.name == name {
        return Ok(entry.into());
      }
    }
    if !options.unwrap_or_default().create {
      return Err(Error::new(Status::GenericFailure, format!("Directory {:?} not found", name)));
    }
    let path = format_dir_path(&self.handle.path, &name);
    if let Some(nfs) = &self.handle.nfs {
      let my_nfs = nfs.to_owned();
      let _ = my_nfs.mkdir(Path::new(path.trim_end_matches('/')))?;
      Ok(JsNfsHandle{nfs: self.handle.nfs.clone(), path, kind: KIND_DIRECTORY.into(), name}.into())
    } else {
      let mut mocks = get_mocks().write().unwrap();
      mocks.dirs.insert(path.clone());
      Ok(JsNfsHandle{nfs: self.handle.nfs.clone(), path, kind: KIND_DIRECTORY.into(), name}.into())
    }
  }

  #[napi]
  pub async fn get_file_handle(&mut self, name: String, #[napi(ts_arg_type="JsNfsGetFileOptions")] options: Option<JsNfsGetFileOptions>) -> Result<JsNfsFileHandle> {
    for entry in self.nfs_entries()? {
      if entry.kind == KIND_FILE && entry.name == name {
        return Ok(entry.into());
      }
    }
    if !options.unwrap_or_default().create {
      return Err(Error::new(Status::GenericFailure, format!("File {:?} not found", name)));
    }
    let path = format_file_path(&self.handle.path, &name);
    if let Some(nfs) = &self.handle.nfs {
      let mut my_nfs = nfs.to_owned();
      let _ = my_nfs.create(Path::new(path.as_str()), nix::fcntl::OFlag::O_SYNC, Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IWGRP | Mode::S_IROTH | Mode::S_IWOTH)?;
      Ok(JsNfsHandle{nfs: self.handle.nfs.clone(), path, kind: KIND_FILE.into(), name}.into())
    } else {
      let mut mocks = get_mocks().write().unwrap();
      mocks.files.insert(path.clone(), Vec::new());
      Ok(JsNfsHandle{nfs: self.handle.nfs.clone(), path, kind: KIND_FILE.into(), name}.into())
    }
  }

  fn nfs_remove(&mut self, entry: &JsNfsHandle, recursive: bool) -> Result<()> {
    let subentries = match entry.kind.as_str() {
      KIND_DIRECTORY => JsNfsDirectoryHandle::from(entry.to_owned()).nfs_entries()?,
      _ => Vec::new(),
    };
    if let Some(nfs) = &self.handle.nfs {
      let my_nfs = nfs.to_owned();
      if entry.kind == KIND_DIRECTORY && !recursive && subentries.len() > 2 {
        return Err(Error::new(Status::GenericFailure, format!("Directory {:?} is not empty", entry.name)));
      }

      if entry.kind == KIND_DIRECTORY {
        for subentry in subentries {
          if subentry.name != DIR_CURRENT && subentry.name != DIR_PARENT {
            self.nfs_remove(&subentry, recursive)?;
          }
        }

        my_nfs.rmdir(Path::new(entry.path.trim_end_matches('/')))?;
      } else {
        my_nfs.unlink(Path::new(entry.path.as_str()))?;
      }
    } else {
      let mut mocks = get_mocks().write().unwrap();
      if entry.kind == KIND_DIRECTORY {
        if !recursive && subentries.len() > 2 {
          return Err(Error::new(Status::GenericFailure, format!("Directory {:?} is not empty", entry.name)));
        }
        mocks.dirs.remove(&entry.path);
      } else {
        mocks.files.remove(&entry.path);
      }
    }

    Ok(())
  }

  #[napi]
  pub async fn remove_entry(&mut self, name: String, #[napi(ts_arg_type="JsNfsRemoveOptions")] options: Option<JsNfsRemoveOptions>) -> Result<()> {
    for entry in self.nfs_entries()? {
      if entry.name == name {
        return self.nfs_remove(&entry, options.unwrap_or_default().recursive);
      }
    }
    Err(Error::new(Status::GenericFailure, format!("Entry {:?} not found", name)))
  }

  fn nfs_resolve(&self, subentries: Vec<JsNfsHandle>, possible_descendant: &JsNfsHandle) -> Result<Vec<String>> {
    for subentry in subentries {
      if subentry.is_same(possible_descendant) {
        return Ok(subentry.path.trim_matches('/').split('/').map(str::to_string).collect());
      }

      if subentry.kind == KIND_DIRECTORY && subentry.name != DIR_CURRENT && subentry.name != DIR_PARENT {
        let subdir = JsNfsDirectoryHandle::from(subentry);
        let res = subdir.nfs_resolve(subdir.nfs_entries()?, possible_descendant);
        if res.is_ok() {
          return res;
        }
      }
    }
    Err(Error::new(Status::GenericFailure, format!("Possible descendant {} {:?} not found", possible_descendant.kind, possible_descendant.name)))
  }

  #[napi(ts_return_type="Promise<Array<string> | null>")]
  pub fn resolve(&self, possible_descendant: JsNfsHandle) -> AsyncTask<JsNfsDirectoryHandleResolve> {
    AsyncTask::new(JsNfsDirectoryHandleResolve{handle: JsNfsDirectoryHandle{handle: self.handle.clone(), kind: self.kind.clone(), name: self.name.clone(), _sym: false}, possible_descendant})
  }
}

impl From<JsNfsHandle> for JsNfsDirectoryHandle {

  fn from(handle: JsNfsHandle) -> Self {
    Self{kind: handle.kind.clone(), name: handle.name.clone(), handle, _sym: false}
  }
}

struct JsNfsDirectoryHandleResolve {
  handle: JsNfsDirectoryHandle,
  possible_descendant: JsNfsHandle
}

#[napi]
impl Task for JsNfsDirectoryHandleResolve {

  type Output = Either<Vec<String>, Null>;

  type JsValue = Either<Vec<String>, Null>;

  fn compute(&mut self) -> Result<Self::Output> {
    self.handle.nfs_resolve(self.handle.nfs_entries()?, &self.possible_descendant)
      .map_or_else(
        |_| Ok(Either::B(Null)),
        |resolved| Ok(Either::A(resolved))
      )
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output)
  }
}

#[napi]
struct JsNfsFileHandle {
  handle: JsNfsHandle,
  #[napi(readonly, ts_type="'file'")]
  pub kind: String,
  #[napi(readonly)]
  pub name: String
}

#[napi]
impl JsNfsFileHandle {

  #[napi]
  pub fn to_handle(&self) -> Result<JsNfsHandle> {
    Ok(self.handle.clone())
  }

  #[napi]
  pub fn is_same_entry(&self, other: &JsNfsHandle) -> Result<bool> {
    self.handle.is_same_entry(other)
  }

  #[napi]
  pub async fn query_permission(&self, perm: JsNfsHandlePermissionDescriptor) -> Result<String> {
    self.handle.query_permission(perm).await
  }

  #[napi]
  pub async fn request_permission(&self, perm: JsNfsHandlePermissionDescriptor) -> Result<String> {
    self.handle.request_permission(perm).await
  }

  #[napi(ts_return_type="Promise<File>")]
  pub async fn get_file(&self) -> Result<JsNfsFile> {
    let path = Path::new(self.handle.path.as_str());
    let type_ = mime_guess::from_path(path).first_raw().unwrap_or(MIME_TYPE_UNKNOWN).into();
    if let Some(nfs) = &self.handle.nfs {
      let my_nfs = nfs.to_owned();
      let nfs_stat = my_nfs.stat64(path)?;
      return Ok(JsNfsFile{handle: self.handle.clone(), size: nfs_stat.nfs_size as i64, type_, last_modified: (nfs_stat.nfs_mtime * 1000) as i64, name: self.name.clone()});
    }
    let mut mocks = get_mocks().write().unwrap();
    let size = mocks.files.entry(self.handle.path.clone()).or_default().len() as i64;
    Ok(JsNfsFile{handle: self.handle.clone(), size, type_, last_modified: 1658159058723, name: self.name.clone()})
  }

  #[napi]
  pub async fn create_writable(&self, #[napi(ts_arg_type="JsNfsCreateWritableOptions")] options: Option<JsNfsCreateWritableOptions>) -> Result<JsNfsWritableFileStream> {
    let position = (!options.unwrap_or_default().keep_existing_data).then(|| 0);
    Ok(JsNfsWritableFileStream{handle: self.handle.clone(), position, locked: false})
  }
}

impl From<JsNfsHandle> for JsNfsFileHandle {

  fn from(handle: JsNfsHandle) -> Self {
    Self{kind: handle.kind.clone(), name: handle.name.clone(), handle}
  }
}

#[napi]
struct JsNfsFile {
  handle: JsNfsHandle,
  #[napi(readonly)]
  pub size: i64,
  #[napi(readonly)]
  pub type_: String,
  #[napi(readonly)]
  pub last_modified: i64,
  #[napi(readonly)]
  pub name: String
}

#[napi]
impl JsNfsFile {

  #[napi(ts_return_type="Promise<ArrayBuffer>")]
  pub fn array_buffer(&self) -> AsyncTask<JsNfsFileArrayBuffer> {
    AsyncTask::new(JsNfsFileArrayBuffer(JsNfsFile{handle: self.handle.clone(), size: self.size, type_: self.type_.clone(), last_modified: self.last_modified, name: self.name.clone()}))
  }

  fn get_index_from_optional(&self, pos: Option<i64>, max: i64, def: i64) -> usize {
    pos.and_then(|mut pos| {
      if pos < 0 {
        pos += max;
        if pos < 0 {
          pos = 0;
        }
      } else if pos > max {
        pos = max;
      }
      Some(pos)
    }).unwrap_or(def) as usize
  }

  pub fn nfs_slice(&self, start: Option<i64>, end: Option<i64>) -> Result<Vec<u8>> {
    let content = self.nfs_bytes()?;
    let len = content.len() as i64;
    let start = self.get_index_from_optional(start, len, 0);
    let end = self.get_index_from_optional(end, len, len);
    Ok(content.get(start..end).unwrap_or_default().to_vec())
  }

  #[napi(ts_return_type="Blob")]
  pub fn slice(&self, env: Env, #[napi(ts_arg_type="number")] start: Option<i64>, #[napi(ts_arg_type="number")] end: Option<i64>, #[napi(ts_arg_type="string")] content_type: Option<String>) -> Result<Object> {
    let sliced = self.nfs_slice(start, end)?;
    let mut arg1 = env.create_array_with_length(1)?;
    let _ = arg1.set_element(0, env.create_arraybuffer_with_data(sliced)?.into_raw().coerce_to_object()?)?;
    let mut arg2 = env.create_object()?;
    let _ = arg2.set_named_property(FIELD_TYPE, env.create_string(content_type.unwrap_or_default().as_str())?)?;
    let global = env.get_global()?;
    let constructor = global.get_named_property::<JsFunction>(JS_TYPE_BLOB)?;
    let blob = constructor.new_instance(&[arg1, arg2])?;
    Ok(blob)
  }

  #[napi(ts_return_type="ReadableStream<Uint8Array>")]
  pub fn stream(&self, env: Env) -> Result<Object> {
    let global = env.get_global()?;
    let constructor = global.get_named_property::<JsFunction>(JS_TYPE_READABLE_STREAM)?;
    let arg = JsNfsReadableStreamSource{content: self.nfs_bytes()?, count: 0, type_: READABLE_STREAM_SOURCE_TYPE_BYTES.into()}.into_instance(env)?;
    let stream = constructor.new_instance(&[arg])?;
    Ok(stream)
  }

  fn nfs_bytes(&self) -> Result<Vec<u8>> {
    if let Some(nfs) = &self.handle.nfs {
      let mut my_nfs = nfs.to_owned();
      let nfs_file = my_nfs.open(Path::new(self.handle.path.as_str()), nix::fcntl::OFlag::O_SYNC)?;
      let nfs_stat = nfs_file.fstat64()?;
      let buffer = &mut vec![0u8; nfs_stat.nfs_size as usize];
      let _ = nfs_file.pread_into(nfs_stat.nfs_size, 0, buffer)?;
      return Ok(buffer.to_vec());
    }
    let mut mocks = get_mocks().write().unwrap();
    Ok(mocks.files.entry(self.handle.path.clone()).or_default().to_vec())
  }

  #[napi]
  pub async fn text(&self) -> Result<String> {
    Ok(std::str::from_utf8(&self.nfs_bytes()?).unwrap().into())
  }
}

struct JsNfsFileArrayBuffer(JsNfsFile);

#[napi]
impl Task for JsNfsFileArrayBuffer {

  type Output = Vec<u8>;

  type JsValue = JsArrayBuffer;

  fn compute(&mut self) -> Result<Self::Output> {
    self.0.nfs_bytes()
  }

  fn resolve(&mut self, env: Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(env.create_arraybuffer_with_data(output)?.into_raw())
  }
}

#[napi]
struct JsNfsReadableStreamSource {
  content: Vec<u8>,
  count: usize,
  #[napi(readonly, ts_type="'bytes'")]
  pub type_: String
}

#[napi]
impl JsNfsReadableStreamSource {

  #[napi]
  pub fn pull(&mut self, env: Env, #[napi(ts_arg_type="ReadableByteStreamController")] controller: Unknown) -> Result<()> {
    let controller = controller.coerce_to_object()?;
    if self.count < self.content.len() {
      let enqueue = controller.get_named_property::<JsFunction>(FIELD_ENQUEUE)?;
      // let arg = env.create_uint32(self.content[self.count] as u32)?;
      // let _ = enqueue.call(Some(&controller), &[arg])?;
      // self.count += 1;
      let arg = env.create_arraybuffer_with_data(self.content.clone())?;
      let arg = arg.into_raw().into_typedarray(TypedArrayType::Uint8, self.content.len(), 0)?;
      let _ = enqueue.call(Some(&controller), &[arg]);
      self.count = self.content.len();
    } else {
      let close = controller.get_named_property::<JsFunction>(FIELD_CLOSE)?;
      let _ = close.call_without_args(Some(&controller))?;
    }
    Ok(())
  }
}

#[napi]
struct JsNfsWritableFileStream {
  handle: JsNfsHandle,
  position: Option<i64>,
  #[napi(readonly)]
  pub locked: bool
}

#[napi]
impl JsNfsWritableFileStream {

  fn parse_write_input(&self, input: Unknown) -> Result<JsNfsWritableFileStreamWriteOptions> {
    match input.get_type()? {
      ValueType::String => self.parse_string(input.coerce_to_string()?, None),
      ValueType::Object => self.parse_write_input_object(input.coerce_to_object()?),
      _ => Err(Error::new(Status::InvalidArg, "Writing unsupported type".into()))
    }
  }

  fn parse_write_input_object(&self, obj: Object) -> Result<JsNfsWritableFileStreamWriteOptions> {
    if obj.has_named_property(FIELD_TYPE)? {
      let type_ = obj.get_named_property::<Unknown>(FIELD_TYPE)?;
      if type_.get_type()? == ValueType::String {
        match type_.coerce_to_string()?.into_utf8()?.as_str()? {
          WRITE_TYPE_SEEK => return self.parse_seek_options(obj),
          WRITE_TYPE_TRUNCATE => return self.parse_truncate_options(obj),
          WRITE_TYPE_WRITE => return self.parse_write_options(obj),
          _ => ()
        };
      }
    }
    match () {
      _ if is_string_object(&obj)? => self.parse_string(obj.coerce_to_string()?, None),
      _ if is_blob(&obj)? => self.parse_blob(obj, None),
      _ if is_typed_array(&obj)? => self.parse_typed_array(obj, None),
      _ if is_data_view(&obj)? => self.parse_data_view(obj, None),
      _ if is_array_buffer(&obj)? => self.parse_array_buffer(obj, None),
      _ => Err(Error::new(Status::InvalidArg, "Writing unsupported type".into()))
    }
  }

  fn parse_seek_options(&self, obj: Object) -> Result<JsNfsWritableFileStreamWriteOptions> {
    if obj.has_named_property(FIELD_POSITION)? {
      let position = obj.get_named_property::<Unknown>(FIELD_POSITION)?;
      if position.get_type()? == ValueType::Number {
        return Ok(JsNfsWritableFileStreamWriteOptions{
          type_: WRITE_TYPE_SEEK.into(),
          data: None,
          position: Some(position.coerce_to_number()?.get_int64()?),
          size: None,
        });
      }
    }
    Err(Error::new(Status::InvalidArg, format!("Property position of type number is required when writing object with type={:?}", WRITE_TYPE_SEEK)))
  }

  fn parse_truncate_options(&self, obj: Object) -> Result<JsNfsWritableFileStreamWriteOptions> {
    if obj.has_named_property(FIELD_SIZE)? {
      let size = obj.get_named_property::<Unknown>(FIELD_SIZE)?;
      if size.get_type()? == ValueType::Number {
        return Ok(JsNfsWritableFileStreamWriteOptions{
          type_: WRITE_TYPE_TRUNCATE.into(),
          data: None,
          position: None,
          size: Some(size.coerce_to_number()?.get_int64()?),
        });
      }
    }
    Err(Error::new(Status::InvalidArg, format!("Property size of type number is required when writing object with type={:?}", WRITE_TYPE_TRUNCATE)))
  }

  fn parse_write_options(&self, obj: Object) -> Result<JsNfsWritableFileStreamWriteOptions> {
    let mut pos = None;
    if obj.has_named_property(FIELD_POSITION)? {
      let position = obj.get_named_property::<Unknown>(FIELD_POSITION)?;
      if position.get_type()? == ValueType::Number {
        pos = Some(position.coerce_to_number()?.get_int64()?);
      }
    }
    if obj.has_named_property(FIELD_DATA)? {
      return self.parse_wrapped_data(obj.get_named_property::<Unknown>(FIELD_DATA)?, pos);
    }
    Err(Error::new(Status::InvalidArg, format!("Property data of type object or string is required when writing object with type={:?}", WRITE_TYPE_WRITE)))
  }

  fn parse_wrapped_data(&self, data: Unknown, position: Option<i64>) -> Result<JsNfsWritableFileStreamWriteOptions> {
    match data.get_type()? {
      ValueType::String => self.parse_string(data.coerce_to_string()?, position),
      ValueType::Object => self.parse_wrapped_data_object(data.coerce_to_object()?, position),
      _ => Err(Error::new(Status::InvalidArg, "Writing unsupported data type".into())),
    }
  }

  fn parse_wrapped_data_object(&self, data: Object, position: Option<i64>) -> Result<JsNfsWritableFileStreamWriteOptions> {
    match () {
      _ if is_string_object(&data)? => self.parse_string(data.coerce_to_string()?, position),
      _ if is_blob(&data)? => self.parse_blob(data, position),
      _ if is_typed_array(&data)? => self.parse_typed_array(data, position),
      _ if is_data_view(&data)? => self.parse_data_view(data, position),
      _ if is_array_buffer(&data)? => self.parse_array_buffer(data, position),
      _ => Err(Error::new(Status::InvalidArg, "Writing unsupported data type".into()))
    }
  }

  fn parse_string(&self, string: JsString, position: Option<i64>) -> Result<JsNfsWritableFileStreamWriteOptions> {
    self.parsed_write_options(Some(string.into_utf8()?.as_str()?.as_bytes().to_owned()), position)
  }

  fn parse_blob(&self, _blob: Object, position: Option<i64>) -> Result<JsNfsWritableFileStreamWriteOptions> {
    self.parsed_write_options(None, position) // FIXME
  }

  fn parse_typed_array(&self, typed_array: Object, position: Option<i64>) -> Result<JsNfsWritableFileStreamWriteOptions> {
    // FIXME: what about length, byte_offset, and typedarray_type?
    self.parsed_write_options(Some(JsTypedArray::try_from(typed_array.into_unknown())?.into_value()?.arraybuffer.into_value()?.to_owned()), position)
  }

  fn parse_data_view(&self, data_view: Object, position: Option<i64>) -> Result<JsNfsWritableFileStreamWriteOptions> {
    // FIXME: what about length and byte_offset?
    self.parsed_write_options(Some(JsDataView::try_from(data_view.into_unknown())?.into_value()?.arraybuffer.into_value()?.to_owned()), position)
  }

  fn parse_array_buffer(&self, array_buffer: Object, position: Option<i64>) -> Result<JsNfsWritableFileStreamWriteOptions> {
    self.parsed_write_options(Some(JsArrayBuffer::try_from(array_buffer.into_unknown())?.into_value()?.to_owned()), position)
  }

  fn parsed_write_options(&self, data: Option<Vec<u8>>, position: Option<i64>) -> Result<JsNfsWritableFileStreamWriteOptions> {
    Ok(JsNfsWritableFileStreamWriteOptions{
      type_: WRITE_TYPE_WRITE.into(),
      data,
      position,
      size: None
    })
  }

  fn try_seek_and_write_data(&mut self, options: &JsNfsWritableFileStreamWriteOptions) -> Result<Undefined> {
    let old_position = self.position.clone();
    if let Some(position) = options.position {
      self.nfs_seek(position)?;
    }
    let res = self.try_write_data(options);
    if !res.is_ok() {
      self.position = old_position;
    }
    res
  }

  fn try_write_data(&mut self, options: &JsNfsWritableFileStreamWriteOptions) -> Result<Undefined> {
    if let Some(data) = &options.data {
      return self.nfs_write(data.as_slice());
    }
    Err(Error::new(Status::InvalidArg, format!("Property data of type object or string is required when writing object with type={:?}", WRITE_TYPE_WRITE)))
  }

  fn nfs_write(&mut self, bytes: &[u8]) -> Result<Undefined> {
    let post_write_pos = if let Some(nfs) = &self.handle.nfs {
      let mut my_nfs = nfs.to_owned();
      let nfs_file = my_nfs.open(Path::new(self.handle.path.as_str()), nix::fcntl::OFlag::O_SYNC)?;
      let offset = match self.position {
        None => nfs_file.fstat64()?.nfs_size,
        Some(pos) => pos as u64
      };
      let _ = nfs_file.pwrite(bytes, offset)?;
      (offset as i64) + (bytes.len() as i64)
    } else {
      let mut mocks = get_mocks().write().unwrap();
      let contents = mocks.files.entry(self.handle.path.clone()).or_default();
      let offset = match self.position {
        None => contents.len(),
        Some(pos) => pos as usize
      };
      if contents.len() >= offset + bytes.len() {
        contents.splice(offset..(offset + bytes.len()), bytes.iter().cloned());
      } else {
        contents.truncate(offset);
        contents.append(&mut bytes.to_vec());
      }
      (offset as i64) + (bytes.len() as i64)
    };
    self.position = Some(post_write_pos);
    Ok(())
  }

  #[napi(ts_return_type="Promise<void>")]
  pub fn write(&'static mut self, #[napi(ts_arg_type="ArrayBuffer | TypedArray | DataView | Blob | String | string | {type: 'write' | 'seek' | 'truncate', data?: ArrayBuffer | TypedArray | DataView | Blob | String | string, position?: number, size?: number}")] data: Unknown) -> Result<AsyncTask<JsNfsWritableFileStreamWrite>> {
    let options = self.parse_write_input(data)?;
    Ok(AsyncTask::new(JsNfsWritableFileStreamWrite{stream: self, options}))
  }

  fn try_seek(&mut self, options: &JsNfsWritableFileStreamWriteOptions) -> Result<Undefined> {
    if let Some(position) = options.position {
      return self.nfs_seek(position);
    }
    Err(Error::new(Status::InvalidArg, format!("Property position of type number is required when writing object with type={:?}", WRITE_TYPE_SEEK)))
  }

  fn nfs_seek(&mut self, position: i64) -> Result<Undefined> {
    if let Some(nfs) = &self.handle.nfs {
      let my_nfs = nfs.to_owned();
      let nfs_stat = my_nfs.stat64(Path::new(self.handle.path.as_str()))?;
      if position > nfs_stat.nfs_size as i64 {
        return Err(Error::new(Status::GenericFailure, "Seeking past size".into()));
      }
    } else {
      let mut mocks = get_mocks().write().unwrap();
      let contents = mocks.files.entry(self.handle.path.clone()).or_default();
      if position > contents.len() as i64 {
        return Err(Error::new(Status::GenericFailure, "Seeking past size".into()));
      }
    }
    self.position = Some(position);
    Ok(())
  }

  #[napi]
  pub async fn seek(&mut self, position: i64) -> Result<Undefined> {
    self.nfs_seek(position)
  }

  fn try_truncate(&mut self, options: &JsNfsWritableFileStreamWriteOptions) -> Result<Undefined> {
    if let Some(size) = options.size {
      return self.nfs_truncate(size);
    }
    Err(Error::new(Status::InvalidArg, format!("Property size of type number is required when writing object with type={:?}", WRITE_TYPE_TRUNCATE)))
  }

  fn nfs_truncate(&mut self, size: i64) -> Result<Undefined> {
    if let Some(nfs) = &self.handle.nfs {
      let my_nfs = nfs.to_owned();
      my_nfs.truncate(Path::new(self.handle.path.as_str()), size as u64)?;
    } else {
      let mut mocks = get_mocks().write().unwrap();
      let contents = mocks.files.entry(self.handle.path.clone()).or_default();
      contents.truncate(size as usize);
    }
    if let Some(position) = self.position {
      if position > size {
        self.position = Some(size);
      }
    }
    Ok(())
  }

  #[napi]
  pub async fn truncate(&mut self, size: i64) -> Result<Undefined> {
    self.nfs_truncate(size)
  }

  #[napi]
  pub async fn close(&self) -> Result<Undefined> {
    Ok(())
  }

  #[napi]
  pub async fn abort(&self, reason: String) -> Result<String> {
    Ok(reason)
  }

  #[napi]
  pub fn release_lock(&mut self) -> Result<Undefined> {
    self.locked = false;
    Ok(())
  }

  #[napi(ts_return_type="WritableStreamDefaultWriter")]
  pub fn get_writer(&'static mut self, env: Env) -> Result<Object> {
    if self.locked {
      return Err(Error::new(Status::GenericFailure, "Invalid state: WritableStream is locked".into()));
    }
    let global = env.get_global()?;
    let sink = JsNfsWritableStreamSink{stream: self, closed: false}.into_instance(env)?;
    let stream_constructor = global.get_named_property::<JsFunction>(JS_TYPE_WRITABLE_STREAM)?;
    let arg = stream_constructor.new_instance(&[sink])?;
    let constructor = global.get_named_property::<JsFunction>(JS_TYPE_WRITABLE_STREAM_DEFAULT_WRITER)?;
    Ok(constructor.new_instance(&[arg])?)
  }
}

struct JsNfsWritableFileStreamWriteOptions {
  type_: String,
  data: Option<Vec<u8>>,
  position: Option<i64>,
  size: Option<i64>
}

impl Default for JsNfsWritableFileStreamWriteOptions {

  fn default() -> Self {
    Self{type_: Default::default(), data: Default::default(), position: Default::default(), size: Default::default()}
  }
}

struct JsNfsWritableFileStreamWrite {
  stream: &'static mut JsNfsWritableFileStream,
  options: JsNfsWritableFileStreamWriteOptions,
}

#[napi]
impl Task for JsNfsWritableFileStreamWrite {

  type Output = ();

  type JsValue = ();

  fn compute(&mut self) -> Result<Self::Output> {
    match self.options.type_.as_str() {
      WRITE_TYPE_WRITE => self.stream.try_seek_and_write_data(&self.options),
      WRITE_TYPE_SEEK => self.stream.try_seek(&self.options),
      WRITE_TYPE_TRUNCATE => self.stream.try_truncate(&self.options),
      _ => Err(Error::new(Status::GenericFailure, format!("Unknown write type: {:?}", self.options.type_.as_str())))
    }
  }

  fn resolve(&mut self, _env: Env, _output: Self::Output) -> Result<Self::JsValue> {
    Ok(())
  }
}

#[napi]
struct JsNfsWritableStreamSink {
  stream: &'static mut JsNfsWritableFileStream,
  closed: bool
}

#[napi]
impl JsNfsWritableStreamSink {

  #[napi(ts_args_type="controller?: WritableStreamDefaultController")]
  pub async fn start(&'static mut self) -> Result<()> {
    self.stream.locked = true;
    Ok(())
  }

  #[napi]
  pub async fn abort(&'static mut self, reason: String) -> Result<String> {
    self.close_stream();
    Ok(reason)
  }

  fn close_stream(&mut self) {
    self.closed = true;
  }

  #[napi(ts_args_type="controller?: WritableStreamDefaultController")]
  pub async fn close(&'static mut self) -> Result<()> {
    if self.closed {
      return Err(Error::new(Status::GenericFailure, "Invalid state: WritableStream is closed".into()));
    }
    self.close_stream();
    Ok(())
  }

  #[napi(ts_return_type="Promise<void>")]
  pub fn write(&'static mut self, #[napi(ts_arg_type="any")] chunk: Unknown, #[napi(ts_arg_type="WritableStreamDefaultController")] _controller: Option<Unknown>) -> Result<AsyncTask<JsNfsWritableStreamWrite>> {
    if self.closed {
      return Err(Error::new(Status::GenericFailure, "Invalid state: WritableStream is closed".into()));
    }
    let options = self.stream.parse_write_input(chunk).unwrap_or_default();
    if options.type_ != WRITE_TYPE_WRITE {
      return Err(Error::new(Status::InvalidArg, "Invalid chunk".into()));
    }
    Ok(AsyncTask::new(JsNfsWritableStreamWrite{sink: self, chunk: options.data.unwrap_or_default()}))
  }
}

struct JsNfsWritableStreamWrite {
  sink: &'static mut JsNfsWritableStreamSink,
  chunk: Vec<u8>
}

#[napi]
impl Task for JsNfsWritableStreamWrite {

  type Output = ();

  type JsValue = ();

  fn compute(&mut self) -> Result<Self::Output> {
    self.sink.stream.nfs_write(self.chunk.as_slice())
  }

  fn resolve(&mut self, _env: Env, _output: Self::Output) -> Result<Self::JsValue> {
    Ok(())
  }
}

fn get_parent_path_and_name(path: &String) -> (String, String) {
  path.rsplit_once('/').map(|res| (res.0.to_string() + "/", res.1.to_string())).unwrap()
}

fn format_dir_path(parent_path: &String, name: &String) -> String {
  format!("{}{}/", parent_path, name)
}

fn format_file_path(parent_path: &String, name: &String) -> String {
  format!("{}{}", parent_path, name)
}

fn is_string_object(obj: &Object) -> Result<bool> {
  Ok(obj.has_named_property(FIELD_SUBSTRING)?
    && obj.get_named_property::<Unknown>(FIELD_SUBSTRING)?.get_type()? == ValueType::Function)
}

fn is_blob(obj: &Object) -> Result<bool> {
  Ok(obj.has_named_property(FIELD_TYPE)?
    && obj.get_named_property::<Unknown>(FIELD_TYPE)?.get_type()? == ValueType::String)
}

fn is_typed_array(obj: &Object) -> Result<bool> {
  Ok(obj.has_named_property(FIELD_LENGTH)?
    && obj.get_named_property::<Unknown>(FIELD_LENGTH)?.get_type()? == ValueType::Number)
}

fn is_data_view(obj: &Object) -> Result<bool> {
  Ok(obj.has_named_property(FIELD_BUFFER)?
    && obj.get_named_property::<Unknown>(FIELD_BUFFER)?.get_type()? == ValueType::Object)
}

fn is_array_buffer(obj: &Object) -> Result<bool> {
  Ok(obj.has_named_property(FIELD_BYTE_LENGTH)?
    && obj.get_named_property::<Unknown>(FIELD_BYTE_LENGTH)?.get_type()? == ValueType::Number)
}
