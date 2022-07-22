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
    write(data: ArrayBuffer | TypedArray | DataView | Blob | string | {type: "write" | "seek" | "truncate", data: ArrayBuffer | TypedArray | DataView | Blob | string, position: number, size: number}): Promise<void>;
    seek(position: number): Promise<void>;
    truncate(size: number): Promise<void>;
}

interface File extends Blob {
    readonly lastModified: number;
    readonly name: string;
    readonly webkitRelativePath: string;
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
const FIELD_CREATE: &str = "create";
const FIELD_RECURSIVE: &str = "recursive";
const FIELD_KEEP_EXISTING_DATA: &str = "keepExistingData";

const KIND_FILE: &str = "file";
const KIND_DIRECTORY: &str = "directory";

const PERM_READ: &str = "read";
const PERM_READWRITE: &str = "readwrite";

const PERM_STATE_GRANTED: &str = "granted";
const PERM_STATE_DENIED: &str = "denied";
const PERM_STATE_PROMPT: &str = "prompt";

#[napi(iterator)]
struct JsNfsDirectoryHandleEntries {
  entries: Vec<JsNfsHandle>,
  count: usize
}

impl Generator for JsNfsDirectoryHandleEntries {

  type Yield = Vec<Value>;

  type Next = Undefined;

  type Return = Undefined;

  fn next(&mut self, _: Option<Self::Next>) -> Option<Self::Yield> {
    if self.entries.len() <= self.count {
      return None;
    }
    let entry = &self.entries[self.count];
    let mut res: Vec<Value> = Vec::new();
    res.push(entry.name.clone().into());
    match entry.kind.as_str() {
      KIND_DIRECTORY => res.push(JsNfsDirectoryHandle::from(entry.clone()).into()),
      _ => res.push(JsNfsFileHandle::from(entry.clone()).into()),
    };
    self.count += 1;
    Some(res)
  }
}

#[napi(iterator)]
struct JsNfsDirectoryHandleKeys {
  entries: Vec<JsNfsHandle>,
  count: usize
}

impl Generator for JsNfsDirectoryHandleKeys {

  type Yield = String;

  type Next = Undefined;

  type Return = Undefined;

  fn next(&mut self, _: Option<Self::Next>) -> Option<Self::Yield> {
    if self.entries.len() <= self.count {
      return None;
    }
    let entry = &self.entries[self.count];
    let res = entry.name.clone();
    self.count += 1;
    Some(res)
  }
}

#[napi(iterator)]
struct JsNfsDirectoryHandleValues {
  entries: Vec<JsNfsHandle>,
  count: usize
}

impl Generator for JsNfsDirectoryHandleValues {

  type Yield = Either<JsNfsDirectoryHandle, JsNfsFileHandle>;

  type Next = Undefined;

  type Return = Undefined;

  fn next(&mut self, _: Option<Self::Next>) -> Option<Self::Yield> {
    if self.entries.len() <= self.count {
      return None;
    }
    let entry = &self.entries[self.count];
    let res: Either<JsNfsDirectoryHandle, JsNfsFileHandle> = match entry.kind.as_str() {
      KIND_DIRECTORY => napi::Either::A(JsNfsDirectoryHandle::from(entry.clone())),
      _ => napi::Either::B(JsNfsFileHandle::from(entry.clone()))
    };
    self.count += 1;
    Some(res)
  }
}

#[napi]
struct JsNfsWritableFileStream {
  handle: JsNfsHandle,
  position: Option<u32>,
  #[napi(readonly)]
  pub locked: bool
}

#[napi]
impl JsNfsWritableFileStream {

  #[napi]
  pub async fn write(&mut self, data: Value) -> napi::Result<Undefined> {
    if !data.is_string() {
      return Err(Error::new(Status::StringExpected, "Writing data other than strings is not implemented yet".to_string()));
    }
    let data = data.as_str().unwrap_or_default();
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
      let _ = nfs_file.pwrite(data.as_bytes(), offset)?;
      self.position = Some((offset as u32) + (data.as_bytes().len() as u32));
    }
    Ok(())
  }

  #[napi]
  pub async fn seek(&mut self, position: u32) -> napi::Result<Undefined> {
    if let Some(nfs) = &self.handle.nfs {
      let my_nfs = nfs.to_owned();
      let nfs_stat = my_nfs.stat64(Path::new(self.handle.path.as_str()))?;
      if position > nfs_stat.nfs_size as u32 {
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
  pub async fn truncate(&mut self, size: u32) -> napi::Result<Undefined> {
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
  pub async fn close(&self) -> napi::Result<Undefined> {
    Ok(())
  }

  #[napi]
  pub async fn abort(&self, reason: String) -> napi::Result<String> {
    Ok(reason)
  }

  #[napi(ts_return_type="WritableStreamDefaultWriter")]
  pub fn get_writer(&self) -> napi::Result<Value> {
    let mut obj: Map<String, Value> = Map::new();
    let _ = obj.insert("ready".to_string(), true.into());
    let _ = obj.insert("closed".to_string(), false.into());
    let _ = obj.insert("desiredSize".to_string(), 123.into());
    let res = Value::Object(obj);
    Ok(res)
  }
}

#[napi]
struct JsNfsHandlePermissionDescriptor {
  #[napi(ts_type="'read' | 'readwrite'")]
  pub mode: String
}

impl FromNapiValue for JsNfsHandlePermissionDescriptor {

  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let obj = from_napi_to_map(env, napi_val)?;
    let mode = obj["mode"].as_str().unwrap().to_string();
    let res = JsNfsHandlePermissionDescriptor{mode};
    Ok(res)
  }
}

#[napi]
struct JsNfsGetDirectoryOptions {
  pub create: bool
}

impl FromNapiValue for JsNfsGetDirectoryOptions {

  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let obj = from_napi_to_map(env, napi_val)?;
    let create = obj.get(FIELD_CREATE).and_then(|val| Some(val.as_bool().unwrap())).unwrap_or_default();
    let res = JsNfsGetDirectoryOptions{create};
    Ok(res)
  }
}

fn should_create_directory(options: Option<JsNfsGetDirectoryOptions>) -> bool {
  if let Some(opt) = options {
    return opt.create
  }
  false
}

#[napi]
struct JsNfsGetFileOptions {
  pub create: bool
}

impl FromNapiValue for JsNfsGetFileOptions {

  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let obj = from_napi_to_map(env, napi_val)?;
    let create = obj.get(FIELD_CREATE).and_then(|val| Some(val.as_bool().unwrap())).unwrap_or_default();
    let res = JsNfsGetFileOptions{create};
    Ok(res)
  }
}

fn should_create_file(options: Option<JsNfsGetFileOptions>) -> bool {
  if let Some(opt) = options {
    return opt.create
  }
  false
}

#[napi]
struct JsNfsRemoveOptions {
  pub recursive: bool
}

impl FromNapiValue for JsNfsRemoveOptions {

  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let obj = from_napi_to_map(env, napi_val)?;
    let recursive = obj.get(FIELD_RECURSIVE).and_then(|val| Some(val.as_bool().unwrap())).unwrap_or_default();
    let res = JsNfsRemoveOptions{recursive};
    Ok(res)
  }
}

fn should_remove_recursively(options: Option<JsNfsRemoveOptions>) -> bool {
  if let Some(opt) = options {
    return opt.recursive
  }
  false
}

#[napi]
struct JsNfsCreateWritableOptions {
  pub keep_existing_data: bool
}

impl FromNapiValue for JsNfsCreateWritableOptions {

  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let obj = from_napi_to_map(env, napi_val)?;
    let keep_existing_data = obj.get(FIELD_KEEP_EXISTING_DATA).and_then(|val| Some(val.as_bool().unwrap())).unwrap_or_default();
    let res = JsNfsCreateWritableOptions{keep_existing_data};
    Ok(res)
  }
}

fn should_keep_existing_data(options: Option<JsNfsCreateWritableOptions>) -> bool {
  if let Some(opt) = options {
    return opt.keep_existing_data
  }
  false
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

  #[napi]
  pub fn is_same_entry(&self, other: JsNfsHandle) -> napi::Result<bool> {
    let res = other.kind == self.kind && other.name == self.name;
    Ok(res)
  }

  #[napi]
  pub async fn query_permission(&self, perm: JsNfsHandlePermissionDescriptor) -> napi::Result<String> {
    if let Some(nfs) = &self.nfs {
      let my_nfs = nfs.to_owned();
      let mode = match perm.mode.as_str() {
        PERM_READWRITE => Mode::S_IRUSR | Mode::S_IWUSR,
        _ => Mode::S_IRUSR
      };
      if my_nfs.access(Path::new(self.name.as_str()), mode.bits().into()).is_ok() {
        return Ok(PERM_STATE_GRANTED.to_string());
      }
      return Ok(PERM_STATE_DENIED.to_string());
    }
    Ok(PERM_STATE_GRANTED.to_string())
  }

  #[napi]
  pub async fn request_permission(&self, perm: JsNfsHandlePermissionDescriptor) -> napi::Result<String> {
    if let Some(nfs) = &self.nfs {
      let my_nfs = nfs.to_owned();
      let mode = match perm.mode.as_str() {
        PERM_READWRITE => Mode::S_IRUSR | Mode::S_IWUSR,
        _ => Mode::S_IRUSR
      };
      if my_nfs.access(Path::new(self.name.as_str()), mode.bits().into()).is_ok() {
        return Ok(PERM_STATE_GRANTED.to_string());
      }
      return Ok(PERM_STATE_DENIED.to_string());
    }
    Ok(PERM_STATE_GRANTED.to_string())
  }
}

impl FromNapiValue for JsNfsHandle {

  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let obj = from_napi_to_map(env, napi_val)?;
    let kind = obj.get(FIELD_KIND).and_then(|val| Some(val.as_str().unwrap().to_string())).unwrap_or_default();
    let name = obj.get(FIELD_NAME).and_then(|val| Some(val.as_str().unwrap().to_string())).unwrap_or_default();
    let res = JsNfsHandle{nfs: None, path: name.clone(), kind: kind.clone(), name: name.clone()};
    Ok(res)
  }
}

#[napi]
struct JsNfsDirectoryHandle {
  handle: JsNfsHandle,
  #[napi(js_name="[Symbol.asyncIterator]", ts_type="JsNfsDirectoryHandle['entries']")]
  pub iter: Value,
  #[napi(readonly, ts_type="'directory'")]
  pub kind: String,
  #[napi(readonly)]
  pub name: String
}

#[napi]
impl JsNfsDirectoryHandle {

  pub fn with_initial_name(name: String) -> Self {
    JsNfsHandle{nfs: None, path: name.clone(), kind: KIND_DIRECTORY.to_string(), name: name.clone()}.into()
  }

  #[napi(constructor)]
  pub fn open(url: String) -> Self {
    JsNfsHandle::open(url).into()
  }

  #[napi]
  pub fn to_handle(&self) -> JsNfsHandle {
    self.handle.clone()
  }

  #[napi]
  pub fn is_same_entry(&self, other: JsNfsHandle) -> napi::Result<bool> {
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
            KIND_DIRECTORY => format!("{}{}/", self.handle.path.clone(), name.clone()),
            _ => format!("{}{}", self.handle.path.clone(), name.clone())
          };
          entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path, kind, name});
        }
      }
    } else {
      entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path: "3".to_string(), kind: KIND_FILE.to_string(), name: "3".to_string()});
      entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path: "annar".to_string(), kind: KIND_FILE.to_string(), name: "annar".to_string()});
      entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path: "first".to_string(), kind: KIND_DIRECTORY.to_string(), name: "first".to_string()});
      entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path: "..".to_string(), kind: KIND_DIRECTORY.to_string(), name: "..".to_string()});
      entries.push(JsNfsHandle{nfs: self.handle.nfs.clone(), path: ".".to_string(), kind: KIND_DIRECTORY.to_string(), name: ".".to_string()});
    }
    Ok(entries)
  }

  #[napi(iterator, ts_return_type="AsyncIterableIterator<[string, FileSystemDirectoryHandle | FileSystemFileHandle]>")]
  pub fn entries(&self) -> napi::Result<JsNfsDirectoryHandleEntries> {
    let res = JsNfsDirectoryHandleEntries{entries: self.nfs_entries()?, count: 0};
    Ok(res)
  }

  #[napi(iterator, ts_return_type="AsyncIterableIterator<string>")]
  pub fn keys(&self) -> napi::Result<JsNfsDirectoryHandleKeys> {
    let res = JsNfsDirectoryHandleKeys{entries: self.nfs_entries()?, count: 0};
    Ok(res)
  }

  #[napi(iterator, ts_return_type="AsyncIterableIterator<FileSystemDirectoryHandle | FileSystemFileHandle>")]
  pub fn values(&self) -> napi::Result<JsNfsDirectoryHandleValues> {
    let res = JsNfsDirectoryHandleValues{entries: self.nfs_entries()?, count: 0};
    Ok(res)
  }

  #[napi]
  pub async fn get_directory_handle(&self, name: String, options: Option<JsNfsGetDirectoryOptions>) -> napi::Result<JsNfsDirectoryHandle> {
    let create = should_create_directory(options);
    let entries = self.nfs_entries()?;
    for entry in entries {
      if entry.kind == KIND_DIRECTORY.to_string() && entry.name == name {
        return Ok(entry.into())
      }
    }
    if !create {
      return Err(Error::new(Status::GenericFailure, format!("Directory {:?} not found", name)));
    }
    let path = format!("{}{}/", self.handle.path.clone(), name.clone());
    if let Some(nfs) = &self.handle.nfs {
      let my_nfs = nfs.to_owned();
      let _ = my_nfs.mkdir(Path::new(path.trim_end_matches('/')))?;
    }
    let res = JsNfsHandle{nfs: self.handle.nfs.clone(), path, kind: KIND_DIRECTORY.to_string(), name: name.clone()};
    Ok(res.into())
  }

  #[napi]
  pub async fn get_file_handle(&self, name: String, options: Option<JsNfsGetFileOptions>) -> napi::Result<JsNfsFileHandle> {
    let create = should_create_file(options);
    let entries = self.nfs_entries()?;
    for entry in entries {
      if entry.kind == KIND_FILE.to_string() && entry.name == name {
        return Ok(entry.into())
      }
    }
    if !create {
      return Err(Error::new(Status::GenericFailure, format!("File {:?} not found", name)));
    }
    let path = format!("{}{}", self.handle.path.clone(), name.clone());
    if let Some(nfs) = &self.handle.nfs {
      let mut my_nfs = nfs.to_owned();
      let _ = my_nfs.create(Path::new(path.as_str()), OFlag::O_SYNC, Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IWGRP | Mode::S_IRGRP | Mode::S_IROTH | Mode::S_IWOTH)?;
    }
    let res = JsNfsHandle{nfs: self.handle.nfs.clone(), path, kind: KIND_FILE.to_string(), name: name.clone()};
    Ok(res.into())
  }

  fn nfs_remove(&self, entry: JsNfsHandle, recursive: bool) -> napi::Result<()> {
    let path = entry.path.clone();
    let kind = entry.kind.clone();
    let name = entry.name.clone();
    if let Some(nfs) = &self.handle.nfs {
      let subentries = JsNfsDirectoryHandle::from(entry).nfs_entries()?;
      if kind == KIND_DIRECTORY.to_string() && !recursive && subentries.len() > 2 {
        return Err(Error::new(Status::GenericFailure, format!("Directory {:?} is not empty", name)));
      }

      if kind == KIND_DIRECTORY.to_string() {
        for subentry in subentries {
          if subentry.name != "." && subentry.name != ".." {
            self.nfs_remove(subentry, recursive)?;
          }
        }

        nfs.rmdir(Path::new(path.trim_end_matches('/')))?;
      } else {
        nfs.unlink(Path::new(path.as_str()))?;
      }
    } else {
      if name == "first".to_string() && !recursive {
        return Err(Error::new(Status::GenericFailure, format!("Directory {:?} is not empty", name)));
      }
    }

    return Ok(())
  }

  #[napi]
  pub async fn remove_entry(&self, name: String, options: Option<JsNfsRemoveOptions>) -> napi::Result<()> {
    let recursive = should_remove_recursively(options);
    let entries = self.nfs_entries()?;
    for entry in entries {
      if entry.name == name {
        return self.nfs_remove(entry, recursive);
      }
    }
    if self.handle.nfs.is_none() && name != "unknown" {
      return Ok(());
    }
    Err(Error::new(Status::GenericFailure, format!("Entry {:?} not found", name)))
  }

  fn nfs_resolve(&self, subentries: Vec<JsNfsHandle>, possible_descendant: JsNfsHandle) -> napi::Result<Vec<String>> {
    for subentry in subentries {
      if subentry.is_same_entry(possible_descendant.clone()).unwrap() {
        let res = subentry.path.trim_matches('/').split('/').map(str::to_string).collect();
        return Ok(res);
      }

      if subentry.nfs.is_some() && subentry.kind == KIND_DIRECTORY && subentry.name != ".".to_string() && subentry.name != "..".to_string() {
        let subdir = JsNfsDirectoryHandle::from(subentry);
        let res = subdir.nfs_resolve(subdir.nfs_entries()?, possible_descendant.clone());
        if res.is_ok() {
          return res;
        }
      }
    }
    Err(Error::new(Status::GenericFailure, format!("Possible descendant {} {:?} not found", possible_descendant.kind.clone(), possible_descendant.name.clone())))
  }

  #[napi]
  pub async fn resolve(&self, possible_descendant: JsNfsHandle) -> napi::Result<Either<Vec<String>, Null>> {
    let res = self.nfs_resolve(self.nfs_entries()?, possible_descendant);
    if res.is_ok() {
      return Ok(napi::Either::A(res.unwrap()));
    }
    Ok(napi::Either::B(Null))
  }
}

impl FromNapiValue for JsNfsDirectoryHandle {

  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let obj = from_napi_to_map(env, napi_val)?;
    let _kind = obj.get(FIELD_KIND).and_then(|val| Some(val.as_str().unwrap().to_string())).unwrap_or(KIND_DIRECTORY.to_string());
    // TODO: check whether _kind matches KIND_DIRECTORY and, if not, return error?
    let name = obj.get(FIELD_NAME).and_then(|val| Some(val.as_str().unwrap().to_string())).unwrap_or_default();
    let res = JsNfsDirectoryHandle::with_initial_name(name.clone());
    Ok(res)
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
    JsNfsDirectoryHandle{iter: Value::Null, kind: handle.kind.clone(), name: handle.name.clone(), handle: handle.clone()}
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

  pub fn with_initial_name(name: String) -> Self {
    JsNfsHandle{nfs: None, path: name.clone(), kind: KIND_FILE.to_string(), name: name.clone()}.into()
  }

  #[napi]
  pub fn to_handle(&self) -> JsNfsHandle {
    self.handle.clone()
  }

  #[napi]
  pub fn is_same_entry(&self, other: JsNfsHandle) -> napi::Result<bool> {
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
      let nfs_stat = my_nfs.stat64(Path::new(self.handle.path.as_str()))?;
      let _type = "text/plain".to_string(); // FIXME
      let webkit_relative_path = ".".to_string(); // FIXME
      let res = JsNfsFile{handle: self.handle.clone(), size: nfs_stat.nfs_size as u32, _type, last_modified: nfs_stat.nfs_mtime as u32, name: self.name.clone(), webkit_relative_path};
      return Ok(res);
    }
    let res = JsNfsFile::with_initial_name(self.name.clone());
    Ok(res)
  }

  #[napi]
  pub async fn create_writable(&self, options: Option<JsNfsCreateWritableOptions>) -> napi::Result<JsNfsWritableFileStream> {
    let keep_existing_data = should_keep_existing_data(options);
    let position = match keep_existing_data {
      false => Some(0),
      _ => None
    };
    let res = JsNfsWritableFileStream{handle: self.handle.clone(), position, locked: keep_existing_data};
    Ok(res)
  }
}

impl FromNapiValue for JsNfsFileHandle {

  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let obj = from_napi_to_map(env, napi_val)?;
    let _kind = obj.get(FIELD_KIND).and_then(|val| Some(val.as_str().unwrap().to_string())).unwrap_or(KIND_FILE.to_string());
    // TODO: check whether _kind matches KIND_FILE and, if not, return error?
    let name = obj.get(FIELD_NAME).and_then(|val| Some(val.as_str().unwrap().to_string())).unwrap_or_default();
    let res = JsNfsFileHandle::with_initial_name(name.clone());
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
    JsNfsFileHandle{kind: handle.kind.clone(), name: handle.name.clone(), handle: handle.clone()}
  }
}

#[napi]
struct JsNfsFile {
  handle: JsNfsHandle,
  #[napi(readonly)]
  pub size: u32,
  #[napi(readonly)]
  pub _type: String,
  #[napi(readonly)]
  pub last_modified: u32,
  #[napi(readonly)]
  pub name: String,
  #[napi(readonly)]
  pub webkit_relative_path: String
}

#[napi]
impl JsNfsFile {

  pub fn with_initial_name(name: String) -> Self {
    let size: u32 = match name.as_str() {
      "writable-write-string" => 10,
      "writable-truncate" => 5,
      _ => 123
    };
    JsNfsFile{
      handle: JsNfsHandle{nfs: None, path: name.clone(), kind: KIND_FILE.to_string(), name: name.clone()},
      size,
      _type: "text/plain".to_string(),
      last_modified: 1658159058,
      name,
      webkit_relative_path: ".".to_string()
    }
  }

  fn to_blob(&self) -> JsNfsBlob {
    let content = self.nfs_text().unwrap();
    JsNfsBlob{size: content.len() as u32, _type: self._type.clone(), content}
  }

  #[napi(ts_return_type="Promise<ArrayBuffer>")]
  pub async fn array_buffer(&self) -> napi::Result<Value> {
    let res = self.to_blob().array_buffer().await?;
    Ok(res)
  }

  #[napi]
  pub fn slice(&self, start: Option<i32>, end: Option<i32>, content_type: Option<String>) -> JsNfsBlob {
    self.to_blob().slice(start, end, content_type)
  }

  #[napi(ts_return_type="ReadableStream<Uint8Array>")]
  pub fn stream(&self) -> napi::Result<Value> {
    self.to_blob().stream()
  }

  fn nfs_text(&self) -> napi::Result<String> {
    if let Some(nfs) = &self.handle.nfs {
      let mut my_nfs = nfs.to_owned();
      let nfs_file = my_nfs.open(Path::new(self.handle.path.as_str()), OFlag::O_SYNC)?;
      let nfs_stat = nfs_file.fstat64()?;
      let buffer = &mut vec![0u8; nfs_stat.nfs_size as usize];
      let _ = nfs_file.pread_into(nfs_stat.nfs_size, 0, buffer)?;
      let res = str::from_utf8(buffer).unwrap();
      return Ok(res.to_string());
    }
    let res = match self.name.as_str() {
      "writable-write-string" => "hello rust".to_string(),
      "writable-truncate" => "hello".to_string(),
      _ => "In order to make sure that this file is exactly 123 bytes in size, I have written this text while watching its chars count.".to_string()
    };
    Ok(res)
  }

  #[napi]
  pub async fn text(&self) -> napi::Result<String> {
    self.nfs_text()
  }
}

#[napi]
struct JsNfsBlob {
  content: String,
  #[napi(readonly)]
  pub size: u32,
  #[napi(readonly)]
  pub _type: String
}

#[napi]
impl JsNfsBlob {

  #[napi(ts_return_type="Promise<ArrayBuffer>")]
  pub async fn array_buffer(&self) -> napi::Result<Value> {
    let mut obj: Map<String, Value> = Map::new();
    let _ = obj.insert("byteLength".to_string(), self.size.into());
    let res = Value::Object(obj);
    Ok(res)
  }

  fn get_index_from_optional(&self, pos: Option<i32>, def: i32) -> usize {
    let mut pos = pos.unwrap_or(def);
    if pos < 0 {
      pos = pos + (self.content.len() as i32);
      if pos <= 0 {
        pos = 0;
      }
    }
    pos as usize
  }

  #[napi]
  pub fn slice(&self, start: Option<i32>, end: Option<i32>, content_type: Option<String>) -> JsNfsBlob {
    let e: usize = self.content.len();
    let start = self.get_index_from_optional(start, 0);
    let end = self.get_index_from_optional(end, e as i32);
    if start >= end {
      return JsNfsBlob{size: 0, _type: content_type.unwrap_or_default(), content: String::new()};
    }
    let mut content = self.content.clone();
    if start == 0 && end == e {
      return JsNfsBlob{size: content.len() as u32, _type: content_type.unwrap_or_default(), content};
    }
    unsafe { content = content.get_unchecked((start as usize)..(end as usize)).to_string() }
    JsNfsBlob{size: content.len() as u32, _type: content_type.unwrap_or_default(), content}
  }

  #[napi(ts_return_type="ReadableStream<Uint8Array>")]
  pub fn stream(&self) -> napi::Result<Value> {
    let mut obj: Map<String, Value> = Map::new();
    let _ = obj.insert("locked".to_string(), true.into());
    let res = Value::Object(obj);
    Ok(res)
  }

  #[napi]
  pub async fn text(&self) -> napi::Result<String> {
    let res = self.content.clone();
    Ok(res)
  }
}

unsafe fn from_napi_to_map(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Map::<String, Value>> {
  let ty = type_of!(env, napi_val)?;
  if ty != ValueType::Object {
    return Err(Error::new(Status::ObjectExpected, format!("Expect {:?}, got: {:?}", ValueType::Object, ty)))
  }

  let mut is_arr = false;
  if sys::napi_is_array(env, napi_val, &mut is_arr) != sys::Status::napi_ok {
    return Err(Error::new(Status::GenericFailure, "Failed to detect whether given js is an array".to_string()))
  }

  if is_arr {
    return Err(Error::new(Status::ObjectExpected, "Expect Object, got Array".to_string()))
  }

  Map::<String, Value>::from_napi_value(env, napi_val)
}
