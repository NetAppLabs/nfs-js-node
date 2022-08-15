#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::{Value, Map};
use std::{env, path::Path, str};
use nix::{fcntl::OFlag, sys::stat::Mode};
use libnfs::Nfs;

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
const FIELD_LENGTH: &str = "length";
const FIELD_BUFFER: &str = "buffer";
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

#[napi(iterator)]
struct JsNfsDirectoryHandleEntries {
  #[napi(js_name="[Symbol.asyncIterator]", ts_type="AsyncIterableIterator<[string, JsNfsDirectoryHandle | JsNfsFileHandle]>")]
  pub _sym: bool, // unused fake member, just to so that generated JsNfsDirectoryHandleEntries class specifies `[Symbol.asyncIterator]: AsyncIterableIterator<[string, JsNfsDirectoryHandle | JsNfsFileHandle]>`
  entries: Vec<JsNfsHandle>,
  count: usize
}

impl Generator for JsNfsDirectoryHandleEntries {

  type Yield = Vec<Value>;

  type Next = ();

  type Return = ();

  fn next(&mut self, _: Option<Self::Next>) -> Option<Self::Yield> {
    if self.entries.len() <= self.count {
      return None;
    }
    let entry = &self.entries[self.count];
    let mut res: Vec<Value> = Vec::new();
    res.push(entry.name.to_owned().into());
    match entry.kind.as_str() {
      KIND_DIRECTORY => res.push(JsNfsDirectoryHandle::from(entry.to_owned()).into()),
      _ => res.push(JsNfsFileHandle::from(entry.to_owned()).into()),
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
    let res: Either<JsNfsDirectoryHandle, JsNfsFileHandle> = match entry.kind.as_str() {
      KIND_DIRECTORY => napi::Either::A(JsNfsDirectoryHandle::from(entry.to_owned())),
      _ => napi::Either::B(JsNfsFileHandle::from(entry.to_owned()))
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
    JsNfsGetDirectoryOptions{create: false}
  }
}

#[napi(object)]
struct JsNfsGetFileOptions {
  pub create: bool
}

impl Default for JsNfsGetFileOptions {

  fn default() -> Self {
    JsNfsGetFileOptions{create: false}
  }
}

#[napi(object)]
struct JsNfsRemoveOptions {
  pub recursive: bool
}

impl Default for JsNfsRemoveOptions {

  fn default() -> Self {
    JsNfsRemoveOptions{recursive: false}
  }
}

#[napi(object)]
struct JsNfsCreateWritableOptions {
  pub keep_existing_data: bool
}

impl Default for JsNfsCreateWritableOptions {

  fn default() -> Self {
    JsNfsCreateWritableOptions{keep_existing_data: false}
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

  pub fn open(url: String) -> Self {
    if env::var("TEST_USING_NFS").is_ok() {
      let mut nfs = Nfs::new().unwrap();
      let _ = nfs.parse_url_mount(url.as_str()).unwrap();
      let name = "/";
      return JsNfsHandle{nfs: Some(nfs), path: name.to_string(), kind: KIND_DIRECTORY.to_string(), name: name.to_string()};
    }
    JsNfsHandle{nfs: None, path: "/".to_string(), kind: KIND_DIRECTORY.to_string(), name: "/".to_string()}
  }

  fn is_same(&self, other: &JsNfsHandle) -> bool {
    other.kind == self.kind && other.name == self.name && (other.path.is_empty() || self.path.is_empty() || other.path == self.path)
  }

  #[napi]
  pub fn is_same_entry(&self, #[napi(ts_arg_type="JsNfsHandle")] other: Object) -> napi::Result<bool> {
    let res = self.is_same(&other.into());
    Ok(res)
  }

  #[napi]
  pub async fn query_permission(&self, perm: JsNfsHandlePermissionDescriptor) -> napi::Result<String> {
    if let Some(nfs) = &self.nfs {
      let my_nfs = nfs.to_owned();
      let nfs_stat = my_nfs.stat64(Path::new(self.path.as_str()))?;
      let perm_u64 = perm.to_u64(self.kind.as_str());
      if nfs_stat.nfs_mode & perm_u64 == perm_u64 {
        return Ok(PERM_STATE_GRANTED.to_string());
      }
    }
    if self.nfs.is_none() && ((self.name != "3" && self.name != "quatre") || perm.mode != PERM_READWRITE.to_string()) {
      return Ok(PERM_STATE_GRANTED.to_string());
    }
    Ok(PERM_STATE_DENIED.to_string())
  }

  #[napi]
  pub async fn request_permission(&self, perm: JsNfsHandlePermissionDescriptor) -> napi::Result<String> {
    if let Some(nfs) = &self.nfs {
      let my_nfs = nfs.to_owned();
      let nfs_stat = my_nfs.stat64(Path::new(self.path.as_str()))?;
      let perm_u64 = perm.to_u64(self.kind.as_str());
      if nfs_stat.nfs_mode & perm_u64 == perm_u64 {
        return Ok(PERM_STATE_GRANTED.to_string());
      }
      let mode = perm.to_mode(self.kind.as_str()).union(Mode::from_bits_truncate(nfs_stat.nfs_mode as u16));
      if !my_nfs.lchmod(Path::new(self.name.as_str()), mode).is_ok() {
        return Ok(PERM_STATE_DENIED.to_string());
      }
    }
    let res = self.query_permission(perm).await?;
    Ok(res)
  }
}

impl From<Object> for JsNfsHandle {

  fn from(obj: Object) -> Self {
    let kind = obj.get::<&str, &str>(FIELD_KIND).unwrap().unwrap_or_default().to_string();
    let name = obj.get::<&str, &str>(FIELD_NAME).unwrap().unwrap_or_default().to_string();
    let path = obj.get::<&str, &str>(FIELD_PATH).unwrap().unwrap_or_default().to_string();
    JsNfsHandle{nfs: None, path, kind, name}
  }
}

#[napi(iterator)]
struct JsNfsDirectoryHandle {
  handle: JsNfsHandle,
  iter: Option<JsNfsDirectoryHandleEntries>,
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
  pub fn to_handle(&self) -> napi::Result<JsNfsHandle> {
    Ok(self.handle.clone())
  }

  #[napi]
  pub fn is_same_entry(&self, #[napi(ts_arg_type="JsNfsHandle")] other: Object) -> napi::Result<bool> {
    self.handle.is_same_entry(other)
  }

  #[napi]
  pub async fn query_permission(&self, perm: JsNfsHandlePermissionDescriptor) -> napi::Result<String> {
    let res = self.handle.query_permission(perm).await?;
    Ok(res)
  }

  #[napi]
  pub async fn request_permission(&self, perm: JsNfsHandlePermissionDescriptor) -> napi::Result<String> {
    let res = self.handle.request_permission(perm).await?;
    Ok(res)
  }

  fn nfs_entries(&self) -> napi::Result<Vec<JsNfsHandle>> {
    let mut entries: Vec<JsNfsHandle> = Vec::new();
    if let Some(nfs) = &self.handle.nfs {
      let mut my_nfs = nfs.to_owned();
      let dir = my_nfs.opendir(Path::new(self.handle.path.as_str()))?;
      for entry in dir {
        if let Some(e) = entry.ok() {
          let kind = match e.d_type {
            libnfs::EntryType::Directory => KIND_DIRECTORY.to_string(),
            _ => KIND_FILE.to_string()
          };
          let name = e.path.into_os_string().into_string().unwrap();
          let path = match kind.as_str() {
            KIND_DIRECTORY => format!("{}{}/", self.handle.path, name),
            _ => format!("{}{}", self.handle.path, name)
          };
          entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path, kind, name});
        }
      }
    } else {
      match self.name.as_str() {
        "/" => {
          entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path: "/3".to_string(), kind: KIND_FILE.to_string(), name: "3".to_string()});
          entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path: "/annar".to_string(), kind: KIND_FILE.to_string(), name: "annar".to_string()});
          entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path: "/quatre/".to_string(), kind: KIND_DIRECTORY.to_string(), name: "quatre".to_string()});
          entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path: "/first/".to_string(), kind: KIND_DIRECTORY.to_string(), name: "first".to_string()});
          entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path: "/../".to_string(), kind: KIND_DIRECTORY.to_string(), name: "..".to_string()});
          entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path: "/./".to_string(), kind: KIND_DIRECTORY.to_string(), name: ".".to_string()});
        },
        "first" => {
          entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path: "/first/comment".to_string(), kind: KIND_FILE.to_string(), name: "comment".to_string()});
          entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path: "/first/../".to_string(), kind: KIND_DIRECTORY.to_string(), name: "..".to_string()});
          entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path: "/first/./".to_string(), kind: KIND_DIRECTORY.to_string(), name: ".".to_string()});
        },
        "quatre" => {
          entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path: "/quatre/points".to_string(), kind: KIND_FILE.to_string(), name: "points".to_string()});
          entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path: "/quatre/../".to_string(), kind: KIND_DIRECTORY.to_string(), name: "..".to_string()});
          entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path: "/quatre/./".to_string(), kind: KIND_DIRECTORY.to_string(), name: ".".to_string()});
        },
        _ => ()
      };
    }
    Ok(entries)
  }

  #[napi(iterator, ts_return_type="AsyncIterableIterator<[string, JsNfsDirectoryHandle | JsNfsFileHandle]>")]
  pub fn entries(&self) -> napi::Result<JsNfsDirectoryHandleEntries> {
    let res = JsNfsDirectoryHandleEntries{entries: self.nfs_entries()?, count: 0, _sym: false};
    Ok(res)
  }

  #[napi(iterator, ts_return_type="AsyncIterableIterator<string>")]
  pub fn keys(&self) -> napi::Result<JsNfsDirectoryHandleKeys> {
    let res = JsNfsDirectoryHandleKeys{entries: self.nfs_entries()?, count: 0, _sym: false};
    Ok(res)
  }

  #[napi(iterator, ts_return_type="AsyncIterableIterator<JsNfsDirectoryHandle | JsNfsFileHandle>")]
  pub fn values(&self) -> napi::Result<JsNfsDirectoryHandleValues> {
    let res = JsNfsDirectoryHandleValues{entries: self.nfs_entries()?, count: 0, _sym: false};
    Ok(res)
  }

  #[napi]
  pub async fn get_directory_handle(&self, name: String, #[napi(ts_arg_type="JsNfsGetDirectoryOptions")] options: Option<JsNfsGetDirectoryOptions>) -> napi::Result<JsNfsDirectoryHandle> {
    for entry in self.nfs_entries()? {
      if entry.kind == KIND_DIRECTORY.to_string() && entry.name == name {
        return Ok(entry.into())
      }
    }
    if !options.unwrap_or_default().create {
      return Err(Error::new(Status::GenericFailure, format!("Directory {:?} not found", name)));
    }
    let path = format!("{}{}/", self.handle.path, name);
    if let Some(nfs) = &self.handle.nfs {
      let my_nfs = nfs.to_owned();
      let _ = my_nfs.mkdir(Path::new(path.trim_end_matches('/')))?;
    }
    let res = JsNfsHandle{nfs: self.handle.nfs.clone(), path, kind: KIND_DIRECTORY.to_string(), name};
    Ok(res.into())
  }

  #[napi]
  pub async fn get_file_handle(&self, name: String, #[napi(ts_arg_type="JsNfsGetFileOptions")] options: Option<JsNfsGetFileOptions>) -> napi::Result<JsNfsFileHandle> {
    for entry in self.nfs_entries()? {
      if entry.kind == KIND_FILE.to_string() && entry.name == name {
        return Ok(entry.into())
      }
    }
    if !options.unwrap_or_default().create {
      return Err(Error::new(Status::GenericFailure, format!("File {:?} not found", name)));
    }
    let path = format!("{}{}", self.handle.path, name);
    if let Some(nfs) = &self.handle.nfs {
      let mut my_nfs = nfs.to_owned();
      let _ = my_nfs.create(Path::new(path.as_str()), OFlag::O_SYNC, Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IWGRP | Mode::S_IROTH | Mode::S_IWOTH)?;
    }
    let res = JsNfsHandle{nfs: self.handle.nfs.clone(), path, kind: KIND_FILE.to_string(), name};
    Ok(res.into())
  }

  fn nfs_remove(&self, entry: &JsNfsHandle, recursive: bool) -> napi::Result<()> {
    if let Some(nfs) = &self.handle.nfs {
      let subentries = match entry.kind.as_str() {
        KIND_DIRECTORY => JsNfsDirectoryHandle::from(entry.to_owned()).nfs_entries()?,
        _ => Vec::new(),
      };
      if entry.kind == KIND_DIRECTORY.to_string() && !recursive && subentries.len() > 2 {
        return Err(Error::new(Status::GenericFailure, format!("Directory {:?} is not empty", entry.name)));
      }

      if entry.kind == KIND_DIRECTORY.to_string() {
        for subentry in subentries {
          if subentry.name != "." && subentry.name != ".." {
            self.nfs_remove(&subentry, recursive)?;
          }
        }

        nfs.rmdir(Path::new(entry.path.trim_end_matches('/')))?;
      } else {
        nfs.unlink(Path::new(entry.path.as_str()))?;
      }
    } else {
      if entry.name == "first".to_string() && !recursive {
        return Err(Error::new(Status::GenericFailure, format!("Directory {:?} is not empty", entry.name)));
      }
    }

    return Ok(())
  }

  #[napi]
  pub async fn remove_entry(&self, name: String, #[napi(ts_arg_type="JsNfsRemoveOptions")] options: Option<JsNfsRemoveOptions>) -> napi::Result<()> {
    for entry in self.nfs_entries()? {
      if entry.name == name {
        return self.nfs_remove(&entry, options.unwrap_or_default().recursive);
      }
    }
    if self.handle.nfs.is_none() && name != "unknown" {
      return Ok(());
    }
    Err(Error::new(Status::GenericFailure, format!("Entry {:?} not found", name)))
  }

  fn nfs_resolve(&self, subentries: Vec<JsNfsHandle>, possible_descendant: &JsNfsHandle) -> napi::Result<Vec<String>> {
    for subentry in subentries {
      if subentry.is_same(possible_descendant) {
        let res = subentry.path.trim_matches('/').split('/').map(str::to_string).collect();
        return Ok(res);
      }

      if subentry.kind == KIND_DIRECTORY && subentry.name != ".".to_string() && subentry.name != "..".to_string() {
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
  pub fn resolve(&self, #[napi(ts_arg_type="JsNfsHandle")] possible_descendant: Object) -> AsyncTask<JsNfsDirectoryHandleResolve> {
    AsyncTask::new(JsNfsDirectoryHandleResolve{handle: JsNfsDirectoryHandle{handle: self.handle.clone(), kind: self.kind.clone(), name: self.name.clone(), iter: None, _sym: false}, possible_descendant: possible_descendant.into()})
  }
}

impl Generator for JsNfsDirectoryHandle {

  type Yield = Vec<Value>;

  type Next = ();

  type Return = ();

  fn next(&mut self, _: Option<Self::Next>) -> Option<Self::Yield> {
    if self.iter.is_none() {
      self.iter = Some(self.entries().unwrap());
    }
    let res = self.iter.as_mut().unwrap().next(None);
    if res.is_none() {
      self.iter = None;
    }
    res
  }
}

impl Into<Value> for JsNfsDirectoryHandle {

  fn into(self) -> Value {
    let mut obj: Map<String, Value> = Map::new();
    let _ = obj.insert(FIELD_KIND.to_string(), KIND_DIRECTORY.into());
    let _ = obj.insert(FIELD_NAME.to_string(), self.name.into());
    Value::Object(obj)
  }
}

impl From<JsNfsHandle> for JsNfsDirectoryHandle {

  fn from(handle: JsNfsHandle) -> Self {
    JsNfsDirectoryHandle{kind: handle.kind.clone(), name: handle.name.clone(), handle, iter: None, _sym: false}
  }
}

struct JsNfsDirectoryHandleResolve {
  handle: JsNfsDirectoryHandle,
  possible_descendant: JsNfsHandle
}

#[napi]
impl napi::Task for JsNfsDirectoryHandleResolve {

  type Output = Either<Vec<String>, Null>;

  type JsValue = Either<Vec<String>, Null>;

  fn compute(&mut self) -> Result<Self::Output> {
    let res = self.handle.nfs_resolve(self.handle.nfs_entries()?, &self.possible_descendant);
    if res.is_ok() {
      return Ok(napi::Either::A(res.unwrap()));
    }
    Ok(napi::Either::B(Null))
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
  pub fn to_handle(&self) -> napi::Result<JsNfsHandle> {
    Ok(self.handle.clone())
  }

  #[napi]
  pub fn is_same_entry(&self, #[napi(ts_arg_type="JsNfsHandle")] other: Object) -> napi::Result<bool> {
    self.handle.is_same_entry(other)
  }

  #[napi]
  pub async fn query_permission(&self, perm: JsNfsHandlePermissionDescriptor) -> napi::Result<String> {
    let res = self.handle.query_permission(perm).await?;
    Ok(res)
  }

  #[napi]
  pub async fn request_permission(&self, perm: JsNfsHandlePermissionDescriptor) -> napi::Result<String> {
    let res = self.handle.request_permission(perm).await?;
    Ok(res)
  }

  #[napi]
  pub async fn get_file(&self) -> napi::Result<JsNfsFile> {
    if let Some(nfs) = &self.handle.nfs {
      let my_nfs = nfs.to_owned();
      let path = Path::new(self.handle.path.as_str());
      let nfs_stat = my_nfs.stat64(path)?;
      let type_ = mime_guess::from_path(path).first_raw().unwrap_or("unknown").to_string();
      let res = JsNfsFile{handle: self.handle.clone(), size: nfs_stat.nfs_size as i64, type_, last_modified: (nfs_stat.nfs_mtime * 1000) as i64, name: self.name.clone()};
      return Ok(res);
    }
    let res = JsNfsFile::with_initial_name(self.name.clone());
    Ok(res)
  }

  #[napi]
  pub async fn create_writable(&self, #[napi(ts_arg_type="JsNfsCreateWritableOptions")] options: Option<JsNfsCreateWritableOptions>) -> napi::Result<JsNfsWritableFileStream> {
    let position = match options.unwrap_or_default().keep_existing_data {
      false => Some(0),
      _ => None
    };
    let res = JsNfsWritableFileStream{handle: self.handle.clone(), position, locked: false};
    Ok(res)
  }
}

impl Into<Value> for JsNfsFileHandle {

  fn into(self) -> Value {
    let mut obj: Map<String, Value> = Map::new();
    let _ = obj.insert(FIELD_KIND.to_string(), KIND_FILE.into());
    let _ = obj.insert(FIELD_NAME.to_string(), self.name.into());
    Value::Object(obj)
  }
}

impl From<JsNfsHandle> for JsNfsFileHandle {

  fn from(handle: JsNfsHandle) -> Self {
    JsNfsFileHandle{kind: handle.kind.clone(), name: handle.name.clone(), handle}
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

  pub fn with_initial_name(name: String) -> Self {
    let size: i64 = match name.as_str() {
      "writable-write-string-after-truncate-via-write" => 22,
      "writable-write-string-after-truncate" => 22,
      "writable-seek-past-size-and-write-string-via-write" => 11,
      "writable-seek-and-write-string-object-via-write" => 11,
      "writable-seek-and-write-string-via-write" => 11,
      "writable-write-string-after-seek-via-write" => 11,
      "writable-write-string-after-seek" => 11,
      "writable-append-string-via-struct" => 27,
      "writable-append-string" => 27,
      "writable-write-strings-via-struct" => 41,
      "writable-write-strings" => 41,
      "writable-write-string-via-struct" => 23,
      "writable-write-string" => 23,
      "writable-truncate-via-write" => 5,
      "writable-truncate" => 5,
      _ => 123
    };
    let type_ = mime_guess::from_path(Path::new(name.as_str())).first_raw().unwrap_or("unknown").to_string();
    JsNfsFile{
      handle: JsNfsHandle{nfs: None, path: name.clone(), kind: KIND_FILE.to_string(), name: name.clone()},
      size,
      type_,
      last_modified: 1658159058723,
      name
    }
  }

  fn to_blob(&self) -> JsNfsBlob {
    let content = self.nfs_bytes().unwrap();
    JsNfsBlob{size: content.len() as i64, type_: self.type_.clone(), content}
  }

  #[napi(ts_return_type="Promise<ArrayBuffer>")]
  pub fn array_buffer(&self) -> AsyncTask<JsNfsFileArrayBuffer> {
    AsyncTask::new(JsNfsFileArrayBuffer(JsNfsFile{handle: self.handle.clone(), size: self.size, type_: self.type_.clone(), last_modified: self.last_modified, name: self.name.clone()}))
  }

  #[napi]
  pub fn slice(&self, #[napi(ts_arg_type="number")] start: Option<i64>, #[napi(ts_arg_type="number")] end: Option<i64>, #[napi(ts_arg_type="string")] content_type: Option<String>) -> napi::Result<JsNfsBlob> {
    self.to_blob().slice(start, end, content_type)
  }

  #[napi(ts_return_type="ReadableStream<Uint8Array>")]
  pub fn stream(&self) -> napi::Result<Value> {
    self.to_blob().stream()
  }

  fn nfs_bytes(&self) -> napi::Result<Vec<u8>> {
    if let Some(nfs) = &self.handle.nfs {
      let mut my_nfs = nfs.to_owned();
      let nfs_file = my_nfs.open(Path::new(self.handle.path.as_str()), OFlag::O_SYNC)?;
      let nfs_stat = nfs_file.fstat64()?;
      let buffer = &mut vec![0u8; nfs_stat.nfs_size as usize];
      let _ = nfs_file.pread_into(nfs_stat.nfs_size, 0, buffer)?;
      return Ok(buffer.to_vec());
    }
    let res = match self.name.as_str() {
      "writable-write-string-after-truncate-via-write" => "hellbound troublemaker".as_bytes(),
      "writable-write-string-after-truncate" => "hellbound troublemaker".as_bytes(),
      "writable-seek-past-size-and-write-string-via-write" => "hello there".as_bytes(),
      "writable-seek-and-write-string-object-via-write" => "hello world".as_bytes(),
      "writable-seek-and-write-string-via-write" => "hello there".as_bytes(),
      "writable-write-string-after-seek-via-write" => "hello there".as_bytes(),
      "writable-write-string-after-seek" => "hello there".as_bytes(),
      "writable-append-string-via-struct" => "salutations from javascript".as_bytes(),
      "writable-append-string" => "salutations from javascript".as_bytes(),
      "writable-write-strings-via-struct" => "hello rust, how are you on this fine day?".as_bytes(),
      "writable-write-strings" => "hello rust, how are you on this fine day?".as_bytes(),
      "writable-write-string-via-struct" => "happy days, all is well".as_bytes(),
      "writable-write-string" => "happy days, all is well".as_bytes(),
      "writable-write-array-buffer-via-struct" => &[0x42,0xff,0xff,0xff,0xff,0xff,0xff,0xf0,0xfe,0x15,0xcd,0x5b,0x07,0xd4,0x31,0x7f],
      "writable-write-array-buffer" => &[0x00,0x00,0x80,0x00,0x31,0xd4,0x07,0x42,0xff,0xff,0xff,0xff,0xff,0xff,0xf0,0xfe,0x15,0xcd,0x5b,0x07,0xd4,0x31,0x7f],
      "writable-write-typed-array-via-struct" => &[0,0,0,0,1,0,0,0,0,0,0,0,2,0,0,0,0,0,0,0,0,0,0,0,3,0,0,0],
      "writable-write-typed-array" => &[0,0,1,0,0,0,2,0,0,0,0,0,3,0,0,0,0,0,0,0,4,0,5,0],
      "writable-write-data-view-via-struct" => &[0x00,0x00,0x80,0x00,0x31,0xd4,0x07,0x42,0xff,0xff,0xff,0xff,0xff,0xff,0xf0,0xfe,0x15,0xcd,0x5b,0x07,0xd4,0x31,0x7f],
      "writable-write-data-view" => &[0x42,0xff,0xff,0xff,0xff,0xff,0xff,0xf0,0xfe,0x15,0xcd,0x5b,0x07,0xd4,0x31,0x7f],
      "writable-truncate-via-write" => "hello".as_bytes(),
      "writable-truncate" => "hello".as_bytes(),
      _ => "In order to make sure that this file is exactly 123 bytes in size, I have written this text while watching its chars count.".as_bytes()
    };
    Ok(res.to_vec())
  }

  #[napi]
  pub async fn text(&self) -> napi::Result<String> {
    let res = str::from_utf8(&self.nfs_bytes()?).unwrap().to_string();
    Ok(res)
  }
}

struct JsNfsFileArrayBuffer(JsNfsFile);

#[napi]
impl napi::Task for JsNfsFileArrayBuffer {

  type Output = Vec<u8>;

  type JsValue = napi::JsArrayBuffer;

  fn compute(&mut self) -> Result<Self::Output> {
    self.0.nfs_bytes()
  }

  fn resolve(&mut self, env: Env, output: Self::Output) -> Result<Self::JsValue> {
    let res = env.create_arraybuffer_with_data(output)?.into_raw();
    Ok(res)
  }
}

#[napi]
struct JsNfsBlob {
  content: Vec<u8>,
  #[napi(readonly)]
  pub size: i64,
  #[napi(readonly)]
  pub type_: String
}

#[napi]
impl JsNfsBlob {

  #[napi(ts_return_type="Promise<ArrayBuffer>")]
  pub fn array_buffer(&self) -> AsyncTask<JsNfsBlobArrayBuffer> {
    AsyncTask::new(JsNfsBlobArrayBuffer(JsNfsBlob{content: self.content.clone(), size: self.size, type_: self.type_.clone()}))
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

  #[napi]
  pub fn slice(&self, #[napi(ts_arg_type="number")] start: Option<i64>, #[napi(ts_arg_type="number")] end: Option<i64>, #[napi(ts_arg_type="string")] content_type: Option<String>) -> napi::Result<JsNfsBlob> {
    let len = self.content.len() as i64;
    let start = self.get_index_from_optional(start, len, 0);
    let end = self.get_index_from_optional(end, len, len);
    let content = self.content.get(start..end).unwrap_or_default().to_vec();
    Ok(JsNfsBlob{size: content.len() as i64, type_: content_type.unwrap_or_default(), content})
  }

  #[napi(ts_return_type="ReadableStream<Uint8Array>")]
  pub fn stream(&self) -> napi::Result<Value> { // TODO
    let mut obj: Map<String, Value> = Map::new();
    let _ = obj.insert("locked".to_string(), true.into());
    let res = Value::Object(obj);
    Ok(res)
  }

  #[napi]
  pub async fn text(&self) -> napi::Result<String> {
    let res = str::from_utf8(&self.content).unwrap().to_string();
    Ok(res)
  }
}

struct JsNfsBlobArrayBuffer(JsNfsBlob);

#[napi]
impl napi::Task for JsNfsBlobArrayBuffer {

  type Output = Vec<u8>;

  type JsValue = napi::JsArrayBuffer;

  fn compute(&mut self) -> Result<Self::Output> {
    Ok(self.0.content.clone())
  }

  fn resolve(&mut self, env: Env, output: Self::Output) -> Result<Self::JsValue> {
    let res = env.create_arraybuffer_with_data(output)?.into_raw();
    Ok(res)
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

  fn parse_write_input(&self, input: Unknown) -> napi::Result<JsNfsWritableFileStreamWriteOptions> {
    match input.get_type()? {
      ValueType::String => self.parse_string(input.coerce_to_string()?, None),
      ValueType::Object => self.parse_write_input_object(input.coerce_to_object()?),
      _ => Err(Error::new(Status::InvalidArg, "Writing unsupported type".to_string()))
    }
  }

  fn parse_write_input_object(&self, obj: Object) -> napi::Result<JsNfsWritableFileStreamWriteOptions> {
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
    if is_string_object(&obj) {
      return self.parse_string(obj.coerce_to_string()?, None);
    } else if is_blob(&obj) {
      return self.parse_blob(obj, None);
    } else if is_typed_array(&obj) {
      return self.parse_typed_array(obj, None);
    } else if is_data_view(&obj) {
      return self.parse_data_view(obj, None);
    } else if is_array_buffer(&obj) {
      return self.parse_array_buffer(obj, None);
    }
    Err(Error::new(Status::InvalidArg, "Writing unsupported type".to_string()))
  }

  fn parse_seek_options(&self, obj: Object) -> napi::Result<JsNfsWritableFileStreamWriteOptions> {
    if obj.has_named_property(FIELD_POSITION)? {
      let position = obj.get_named_property::<Unknown>(FIELD_POSITION)?;
      if position.get_type()? == ValueType::Number {
        return Ok(JsNfsWritableFileStreamWriteOptions{
          type_: WRITE_TYPE_SEEK.to_string(),
          data: None,
          position: Some(position.coerce_to_number()?.get_int64()?),
          size: None,
        });
      }
    }
    Err(Error::new(Status::InvalidArg, format!("Property position of type number is required when writing object with type={:?}", WRITE_TYPE_SEEK.to_string())))
  }

  fn parse_truncate_options(&self, obj: Object) -> napi::Result<JsNfsWritableFileStreamWriteOptions> {
    if obj.has_named_property(FIELD_SIZE)? {
      let size = obj.get_named_property::<Unknown>(FIELD_SIZE)?;
      if size.get_type()? == ValueType::Number {
        return Ok(JsNfsWritableFileStreamWriteOptions{
          type_: WRITE_TYPE_TRUNCATE.to_string(),
          data: None,
          position: None,
          size: Some(size.coerce_to_number()?.get_int64()?),
        });
      }
    }
    Err(Error::new(Status::InvalidArg, format!("Property size of type number is required when writing object with type={:?}", WRITE_TYPE_TRUNCATE.to_string())))
  }

  fn parse_write_options(&self, obj: Object) -> napi::Result<JsNfsWritableFileStreamWriteOptions> {
    let mut pos: Option<i64> = None;
    if obj.has_named_property(FIELD_POSITION)? {
      let position = obj.get_named_property::<Unknown>(FIELD_POSITION)?;
      if position.get_type()? == ValueType::Number {
        pos = Some(position.coerce_to_number()?.get_int64()?);
      }
    }
    if obj.has_named_property(FIELD_DATA)? {
      return self.parse_wrapped_data(obj.get_named_property::<Unknown>(FIELD_DATA)?, pos);
    }
    Err(Error::new(Status::InvalidArg, format!("Property data of type object or string is required when writing object with type={:?}", WRITE_TYPE_WRITE.to_string())))
  }

  fn parse_wrapped_data(&self, data: Unknown, position: Option<i64>) -> napi::Result<JsNfsWritableFileStreamWriteOptions> {
    match data.get_type()? {
      ValueType::String => self.parse_string(data.coerce_to_string()?, position),
      ValueType::Object => self.parse_wrapped_data_object(data.coerce_to_object()?, position),
      _ => Err(Error::new(Status::InvalidArg, "Writing unsupported data type".to_string())),
    }
  }

  fn parse_wrapped_data_object(&self, data: Object, position: Option<i64>) -> napi::Result<JsNfsWritableFileStreamWriteOptions> {
    if is_string_object(&data) {
      return self.parse_string(data.coerce_to_string()?, position);
    } else if is_blob(&data) {
      return self.parse_blob(data, position);
    } else if is_typed_array(&data) {
      return self.parse_typed_array(data, position);
    } else if is_data_view(&data) {
      return self.parse_data_view(data, position);
    } else if is_array_buffer(&data) {
      return self.parse_array_buffer(data, position);
    }
    Err(Error::new(Status::InvalidArg, "Writing unsupported data type".to_string()))
  }

  fn parse_string(&self, string: napi::JsString, position: Option<i64>) -> napi::Result<JsNfsWritableFileStreamWriteOptions> {
    self.parsed_write_options(Some(string.into_utf8()?.as_str()?.as_bytes().to_owned()), position)
  }

  fn parse_blob(&self, _blob: Object, position: Option<i64>) -> napi::Result<JsNfsWritableFileStreamWriteOptions> {
    self.parsed_write_options(None, position) // FIXME
  }

  fn parse_typed_array(&self, typed_array: Object, position: Option<i64>) -> napi::Result<JsNfsWritableFileStreamWriteOptions> {
    // FIXME: what about length, byte_offset, and typedarray_type?
    self.parsed_write_options(Some(napi::JsTypedArray::try_from(typed_array.into_unknown())?.into_value()?.arraybuffer.into_value()?.to_owned()), position)
  }

  fn parse_data_view(&self, data_view: Object, position: Option<i64>) -> napi::Result<JsNfsWritableFileStreamWriteOptions> {
    // FIXME: what about length and byte_offset?
    self.parsed_write_options(Some(napi::JsDataView::try_from(data_view.into_unknown())?.into_value()?.arraybuffer.into_value()?.to_owned()), position)
  }

  fn parse_array_buffer(&self, array_buffer: Object, position: Option<i64>) -> napi::Result<JsNfsWritableFileStreamWriteOptions> {
    self.parsed_write_options(Some(napi::JsArrayBuffer::try_from(array_buffer.into_unknown())?.into_value()?.to_owned()), position)
  }

  fn parsed_write_options(&self, data: Option<Vec<u8>>, position: Option<i64>) -> napi::Result<JsNfsWritableFileStreamWriteOptions> {
    Ok(JsNfsWritableFileStreamWriteOptions{
      type_: WRITE_TYPE_WRITE.to_string(),
      data,
      position,
      size: None
    })
  }

  fn try_seek_and_write_data(&mut self, options: &JsNfsWritableFileStreamWriteOptions) -> napi::Result<Undefined> {
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

  fn try_write_data(&mut self, options: &JsNfsWritableFileStreamWriteOptions) -> napi::Result<Undefined> {
    if let Some(data) = &options.data {
      return self.nfs_write(data.as_slice());
    }
    Err(Error::new(Status::InvalidArg, format!("Property data of type object or string is required when writing object with type={:?}", WRITE_TYPE_WRITE.to_string())))
  }

  fn nfs_write(&mut self, bytes: &[u8]) -> napi::Result<Undefined> {
    if let Some(nfs) = &self.handle.nfs {
      let mut my_nfs = nfs.to_owned();
      let nfs_file = my_nfs.open(Path::new(self.handle.path.as_str()), OFlag::O_SYNC)?;
      let offset = match self.position {
        None => {
          let nfs_stat = nfs_file.fstat64()?;
          nfs_stat.nfs_size
        },
        _ => self.position.unwrap() as u64
      };
      let _ = nfs_file.pwrite(bytes, offset)?;
      self.position = Some((offset as i64) + (bytes.len() as i64));
    }
    Ok(())
  }

  #[napi(ts_return_type="Promise<void>")]
  pub fn write(&'static mut self, #[napi(ts_arg_type="ArrayBuffer | TypedArray | DataView | Blob | String | string | {type: 'write' | 'seek' | 'truncate', data?: ArrayBuffer | TypedArray | DataView | Blob | String | string, position?: number, size?: number}")] data: Unknown) -> napi::Result<AsyncTask<JsNfsWritableFileStreamWrite>> {
    let options = self.parse_write_input(data)?;
    Ok(AsyncTask::new(JsNfsWritableFileStreamWrite{stream: self, options}))
  }

  fn try_seek(&mut self, options: &JsNfsWritableFileStreamWriteOptions) -> napi::Result<Undefined> {
    if let Some(position) = options.position {
      return self.nfs_seek(position);
    }
    Err(Error::new(Status::InvalidArg, format!("Property position of type number is required when writing object with type={:?}", WRITE_TYPE_SEEK.to_string())))
  }

  fn nfs_seek(&mut self, position: i64) -> napi::Result<Undefined> {
    if let Some(nfs) = &self.handle.nfs {
      let my_nfs = nfs.to_owned();
      let nfs_stat = my_nfs.stat64(Path::new(self.handle.path.as_str()))?;
      if position > nfs_stat.nfs_size as i64 {
        return Err(Error::new(Status::GenericFailure, "Seeking past size".to_string()));
      }
    } else {
      if position > 123 {
        return Err(Error::new(Status::GenericFailure, "Seeking past size".to_string()));
      }
    }
    self.position = Some(position);
    Ok(())
  }

  #[napi]
  pub async fn seek(&mut self, position: i64) -> napi::Result<Undefined> {
    self.nfs_seek(position)
  }

  fn try_truncate(&mut self, options: &JsNfsWritableFileStreamWriteOptions) -> napi::Result<Undefined> {
    if let Some(size) = options.size {
      return self.nfs_truncate(size);
    }
    Err(Error::new(Status::InvalidArg, format!("Property size of type number is required when writing object with type={:?}", WRITE_TYPE_TRUNCATE.to_string())))
  }

  fn nfs_truncate(&mut self, size: i64) -> napi::Result<Undefined> {
    if let Some(nfs) = &self.handle.nfs {
      let my_nfs = nfs.to_owned();
      my_nfs.truncate(Path::new(self.handle.path.as_str()), size as u64)?;
    }
    if let Some(position) = self.position {
      if position > size {
        self.position = Some(size);
      }
    }
    Ok(())
  }

  #[napi]
  pub async fn truncate(&mut self, size: i64) -> napi::Result<Undefined> {
    self.nfs_truncate(size)
  }

  #[napi]
  pub async fn close(&self) -> napi::Result<Undefined> {
    Ok(())
  }

  #[napi]
  pub async fn abort(&self, reason: String) -> napi::Result<String> {
    Ok(reason)
  }

  #[napi(ts_return_type="WritableStreamDefaultWriter")]
  pub fn get_writer(&mut self) -> napi::Result<Value> { // TODO
    if self.locked {
      return Err(Error::new(Status::GenericFailure, "Writable file stream locked by another writer".to_string()));
    }
    let mut obj: Map<String, Value> = Map::new();
    let _ = obj.insert("ready".to_string(), true.into());
    let _ = obj.insert("closed".to_string(), false.into());
    let _ = obj.insert("desiredSize".to_string(), 123.into());
    let res = Value::Object(obj);
    self.locked = true;
    Ok(res)
  }
}

struct JsNfsWritableFileStreamWriteOptions {
  type_: String,
  data: Option<Vec<u8>>,
  position: Option<i64>,
  size: Option<i64>
}

struct JsNfsWritableFileStreamWrite {
  stream: &'static mut JsNfsWritableFileStream,
  options: JsNfsWritableFileStreamWriteOptions,
}

#[napi]
impl napi::Task for JsNfsWritableFileStreamWrite {

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

fn is_string_object(obj: &Object) -> bool {
  if obj.has_named_property(FIELD_SUBSTRING).unwrap() {
    let substring = obj.get_named_property::<Unknown>(FIELD_SUBSTRING).unwrap();
    return substring.get_type().unwrap() == ValueType::Function;
  }
  false
}

fn is_blob(obj: &Object) -> bool {
  if obj.has_named_property(FIELD_TYPE).unwrap() {
    let type_ = obj.get_named_property::<Unknown>(FIELD_TYPE).unwrap();
    return type_.get_type().unwrap() == ValueType::String;
  }
  false
}

fn is_typed_array(obj: &Object) -> bool {
  if obj.has_named_property(FIELD_LENGTH).unwrap() {
    let length = obj.get_named_property::<Unknown>(FIELD_LENGTH).unwrap();
    return length.get_type().unwrap() == ValueType::Number;
  }
  false
}

fn is_data_view(obj: &Object) -> bool {
  if obj.has_named_property(FIELD_BUFFER).unwrap() {
    let buffer = obj.get_named_property::<Unknown>(FIELD_BUFFER).unwrap();
    return buffer.get_type().unwrap() == ValueType::Object;
  }
  false
}

fn is_array_buffer(obj: &Object) -> bool {
  if obj.has_named_property(FIELD_BYTE_LENGTH).unwrap() {
    let byte_length = obj.get_named_property::<Unknown>(FIELD_BYTE_LENGTH).unwrap();
    return byte_length.get_type().unwrap() == ValueType::Number;
  }
  false
}
