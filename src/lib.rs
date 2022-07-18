#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::{Value, Map};

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
    create?: boolean;
}

interface FileSystemGetFileOptions {
    create?: boolean;
}

interface FileSystemRemoveOptions {
    recursive?: boolean;
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
    keepExistingData?: boolean;
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

const KIND_FILE: &str = "file";
const KIND_DIRECTORY: &str = "directory";

const PERM_READ: &str = "read";
const PERM_READWRITE: &str = "readwrite";

const PERM_STATE_GRANTED: &str = "granted";
const PERM_STATE_DENIED: &str = "denied";
const PERM_STATE_PROMPT: &str = "prompt";

#[napi(iterator)]
struct JsNfsDirectoryHandleEntries {
  count: usize
}

impl Generator for JsNfsDirectoryHandleEntries {

  type Yield = Vec<Value>;

  type Next = Undefined;

  type Return = Undefined;

  fn next(&mut self, value: Option<Self::Next>) -> Option<Self::Yield> {
    self.count += 1;
    match self.count {
      1 => {
        let mut entry: Vec<Value> = Vec::new();
        entry.push(String::from("first").into());
        entry.push(JsNfsDirectoryHandle::with_initial_name(String::from("first")).into());
        Some(entry)
      },
      2 => {
        let mut entry: Vec<Value> = Vec::new();
        entry.push(String::from("annar").into());
        entry.push(JsNfsFileHandle::with_initial_name(String::from("annar")).into());
        Some(entry)
      },
      3 => {
        let mut entry: Vec<Value> = Vec::new();
        entry.push(String::from("3").into());
        entry.push(JsNfsFileHandle::with_initial_name(String::from("3")).into());
        Some(entry)
      },
      _ => None
    }
  }
}

#[napi(iterator)]
struct JsNfsDirectoryHandleKeys {
  count: usize
}

impl Generator for JsNfsDirectoryHandleKeys {

  type Yield = String;

  type Next = Undefined;

  type Return = Undefined;

  fn next(&mut self, value: Option<Self::Next>) -> Option<Self::Yield> {
    self.count += 1;
    match self.count {
      1 => Some(String::from("first")),
      2 => Some(String::from("annar")),
      3 => Some(String::from("3")),
      _ => None
    }
  }
}

#[napi(iterator)]
struct JsNfsDirectoryHandleValues {
  count: usize
}

impl Generator for JsNfsDirectoryHandleValues {

  type Yield = Either<JsNfsDirectoryHandle, JsNfsFileHandle>;

  type Next = Undefined;

  type Return = Undefined;

  fn next(&mut self, value: Option<Self::Next>) -> Option<Self::Yield> {
    self.count += 1;
    match self.count {
      1 => Some(napi::Either::A(JsNfsDirectoryHandle::with_initial_name(String::from("first")))),
      2 => Some(napi::Either::B(JsNfsFileHandle::with_initial_name(String::from("annar")))),
      3 => Some(napi::Either::B(JsNfsFileHandle::with_initial_name(String::from("3")))),
      _ => None
    }
  }
}

struct NFSWritableFileStream {
  locked: bool
}

impl NFSWritableFileStream {

  pub fn new() -> Self {
    Self::with_initial_locked(false)
  }

  pub fn with_initial_locked(locked: bool) -> Self {
    NFSWritableFileStream{locked}
  }

  pub async fn write(&self, data: Value) -> napi::Result<Undefined> {
    Ok(())
  }

  pub async fn close(&self) -> napi::Result<Undefined> {
    Ok(())
  }
}

#[napi]
struct JsNfsWritableFileStream {
  inner: NFSWritableFileStream,
  #[napi(readonly)]
  pub locked: bool
}

#[napi]
impl JsNfsWritableFileStream {

  pub fn with_initial_locked(locked: bool) -> Self {
    JsNfsWritableFileStream{inner: NFSWritableFileStream::with_initial_locked(locked), locked}
  }

  pub fn new() -> Self {
    Self::with_initial_locked(false)
  }

  #[napi]
  pub async fn write(&self, data: Value) -> napi::Result<Undefined> {
    let res = self.inner.write(data).await?;
    Ok(res)
  }

  #[napi]
  pub async fn close(&self) -> napi::Result<Undefined> {
    let res = self.inner.close().await?;
    Ok(res)
  }
}

struct NFSHandlePermissionDescriptor {
  mode: String
}

#[napi]
struct JsNfsHandlePermissionDescriptor {
  inner: NFSHandlePermissionDescriptor,
  #[napi(ts_type="'read' | 'readwrite'")]
  pub mode: String
}

impl FromNapiValue for JsNfsHandlePermissionDescriptor {

  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let obj = from_napi_to_map(env, napi_val)?;
    let mode = obj["mode"].as_str().unwrap().to_string();
    let res = JsNfsHandlePermissionDescriptor{inner: NFSHandlePermissionDescriptor{mode: mode.clone()}, mode};
    Ok(res)
  }
}

struct NFSGetDirectoryOptions {
  create: Option<bool>
}

fn should_create_directory(options: Option<NFSGetDirectoryOptions>) -> bool {
  if let Some(opt) = options {
    if let Some(create) = opt.create {
      return create
    }
  }
  false
}

#[napi]
struct JsNfsGetDirectoryOptions {
  inner: NFSGetDirectoryOptions,
  pub create: Option<bool>
}

impl FromNapiValue for JsNfsGetDirectoryOptions {

  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let obj = from_napi_to_map(env, napi_val)?;
    let create = obj["create"].as_bool();
    let res = JsNfsGetDirectoryOptions{inner: NFSGetDirectoryOptions{create}, create};
    Ok(res)
  }
}

struct NFSGetFileOptions {
  create: Option<bool>
}

fn should_create_file(options: Option<NFSGetFileOptions>) -> bool {
  if let Some(opt) = options {
    if let Some(create) = opt.create {
      return create
    }
  }
  false
}

#[napi]
struct JsNfsGetFileOptions {
  inner: NFSGetFileOptions,
  pub create: Option<bool>
}

impl FromNapiValue for JsNfsGetFileOptions {

  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let obj = from_napi_to_map(env, napi_val)?;
    let create = obj["create"].as_bool();
    let res = JsNfsGetFileOptions{inner: NFSGetFileOptions{create}, create};
    Ok(res)
  }
}

struct NFSRemoveOptions {
  recursive: Option<bool>
}

fn should_remove_recursively(options: Option<NFSRemoveOptions>) -> bool {
  if let Some(opt) = options {
    if let Some(recursive) = opt.recursive {
      return recursive
    }
  }
  false
}

#[napi]
struct JsNfsRemoveOptions {
  inner: NFSRemoveOptions,
  pub recursive: Option<bool>
}

impl FromNapiValue for JsNfsRemoveOptions {

  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let obj = from_napi_to_map(env, napi_val)?;
    let recursive = obj["recursive"].as_bool();
    let res = JsNfsRemoveOptions{inner: NFSRemoveOptions{recursive}, recursive};
    Ok(res)
  }
}

struct NFSCreateWritableOptions {
  keep_existing_data: Option<bool>
}

#[napi]
struct JsNfsCreateWritableOptions {
  inner: NFSCreateWritableOptions,
  pub keep_existing_data: Option<bool>
}

impl FromNapiValue for JsNfsCreateWritableOptions {

  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let obj = from_napi_to_map(env, napi_val)?;
    let keep_existing_data = obj["keepExistingData"].as_bool();
    let res = JsNfsCreateWritableOptions{inner: NFSCreateWritableOptions{keep_existing_data}, keep_existing_data};
    Ok(res)
  }
}

#[derive(Clone)]
struct NFSHandle {
  kind: String,
  name: String
}

impl NFSHandle {

  pub fn new(kind: String) -> Self {
    Self::with_initial_values(kind, String::new())
  }

  pub fn with_initial_values(kind: String, name: String) -> Self {
    NFSHandle{kind, name}
  }

  pub fn is_same_entry(&self, other: NFSHandle) -> napi::Result<bool> {
    let res = other.kind == self.kind && other.name == self.name;
    Ok(res)
  }

  pub async fn query_permission(&self, perm: NFSHandlePermissionDescriptor) -> napi::Result<String> {
    match perm.mode.as_str() {
      PERM_READ => Ok(PERM_STATE_GRANTED.to_string()),
      PERM_READWRITE => Ok(PERM_STATE_DENIED.to_string()),
      _ => Ok(PERM_STATE_PROMPT.to_string())
    }
  }

  pub async fn request_permission(&self, perm: NFSHandlePermissionDescriptor) -> napi::Result<String> {
    match perm.mode.as_str() {
      PERM_READ => Ok(PERM_STATE_PROMPT.to_string()),
      PERM_READWRITE => Ok(PERM_STATE_GRANTED.to_string()),
      _ => Ok(PERM_STATE_DENIED.to_string())
    }
  }
}

#[napi]
struct JsNfsHandle {
  inner: NFSHandle,
  #[napi(readonly, ts_type="'directory' | 'file'")]
  pub kind: String,
  #[napi(readonly)]
  pub name: String
}

#[napi]
impl JsNfsHandle {

  pub fn with_initial_values(kind: String, name: String) -> Self {
    JsNfsHandle{inner: NFSHandle::with_initial_values(kind.clone(), name.clone()), kind, name}
  }

  #[napi]
  pub fn is_same_entry(&self, other: JsNfsHandle) -> napi::Result<bool> {
    self.inner.is_same_entry(other.inner)
  }

  #[napi]
  pub async fn query_permission(&self, perm: JsNfsHandlePermissionDescriptor) -> napi::Result<String> {
    let res = self.inner.query_permission(perm.inner).await?;
    Ok(res)
  }

  #[napi]
  pub async fn request_permission(&self, perm: JsNfsHandlePermissionDescriptor) -> napi::Result<String> {
    let res = self.inner.request_permission(perm.inner).await?;
    Ok(res)
  }
}

impl FromNapiValue for JsNfsHandle {

  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let obj = from_napi_to_map(env, napi_val)?;
    let kind = obj.get("kind").and_then(|val| Some(val.as_str().unwrap().to_string()));
    let name = obj.get("name").and_then(|val| Some(val.as_str().unwrap().to_string()));
    let res = JsNfsHandle::with_initial_values(kind.unwrap_or(String::new()), name.unwrap_or(String::new()));
    Ok(res)
  }
}

#[derive(Clone)]
struct NFSDirectoryHandle {
  handle: NFSHandle,
  kind: String,
  name: String
}

impl NFSDirectoryHandle {

  pub fn new() -> Self {
    Self::with_initial_name(String::new())
  }

  pub fn open(url: String) -> Self {
    // TODO: use url param
    Self::with_initial_name(url)
  }

  pub fn with_initial_name(name: String) -> Self {
    NFSDirectoryHandle{handle: NFSHandle::with_initial_values(KIND_DIRECTORY.to_string(), name.clone()), kind: KIND_DIRECTORY.to_string(), name}
  }

  pub fn is_same_entry(&self, other: NFSHandle) -> napi::Result<bool> {
    self.handle.is_same_entry(other)
  }

  pub async fn query_permission(&self, perm: NFSHandlePermissionDescriptor) -> napi::Result<String> {
    let res = self.handle.query_permission(perm).await?;
    Ok(res)
  }

  pub async fn request_permission(&self, perm: NFSHandlePermissionDescriptor) -> napi::Result<String> {
    let res = self.handle.request_permission(perm).await?;
    Ok(res)
  }

  pub fn entries(&self) -> napi::Result<JsNfsDirectoryHandleEntries> {
    let res = JsNfsDirectoryHandleEntries{count: 0};
    Ok(res)
  }

  pub fn keys(&self) -> napi::Result<JsNfsDirectoryHandleKeys> {
    let res = JsNfsDirectoryHandleKeys{count: 0};
    Ok(res)
  }

  pub fn values(&self) -> napi::Result<JsNfsDirectoryHandleValues> {
    let res = JsNfsDirectoryHandleValues{count: 0};
    Ok(res)
  }

  pub async fn get_directory_handle(&self, name: String, options: Option<NFSGetDirectoryOptions>) -> napi::Result<NFSDirectoryHandle> {
    let create = should_create_directory(options);
    if name.ne("first") && !create {
      return Err(Error::new(Status::GenericFailure, format!("Directory {:?} not found", name)));
    }
    let res = NFSDirectoryHandle::with_initial_name(name);
    Ok(res)
  }

  pub async fn get_file_handle(&self, name: String, options: Option<NFSGetFileOptions>) -> napi::Result<NFSFileHandle> {
    let create = should_create_file(options);
    if name.ne("annar") && name.ne("3") && !create {
      return Err(Error::new(Status::GenericFailure, format!("File {:?} not found", name)));
    }
    let res = NFSFileHandle::with_initial_name(name);
    Ok(res)
  }

  pub async fn remove_entry(&self, name: String, options: Option<NFSRemoveOptions>) -> napi::Result<()> {
    let recursive = should_remove_recursively(options);
    match name.as_str() {
      "first" => {
        if !recursive {
          return Err(Error::new(Status::GenericFailure, format!("Directory {:?} is not empty", name)));
        }
        Ok(())
      },
      "annar" => Ok(()),
      "3" => Ok(()),
      _ => Err(Error::new(Status::GenericFailure, format!("Entry {:?} not found", name)))
    }
  }

  pub async fn resolve(&self, possible_descendant: NFSHandle) -> napi::Result<Either<Vec<String>, Null>> {
    let first = NFSHandle{kind: KIND_DIRECTORY.to_string(), name: String::from("first")};
    let annar = NFSHandle{kind: KIND_FILE.to_string(), name: String::from("annar")};
    let three = NFSHandle{kind: KIND_FILE.to_string(), name: String::from("3")};
    match possible_descendant {
      first => {
        let mut res: Vec<String> = Vec::new();
        res.push(first.name);
        Ok(napi::Either::A(res))
      },
      annar => {
        let mut res: Vec<String> = Vec::new();
        res.push(annar.name);
        Ok(napi::Either::A(res))
      },
      three => {
        let mut res: Vec<String> = Vec::new();
        res.push(three.name);
        Ok(napi::Either::A(res))
      },
      _ => Ok(napi::Either::B(Null))
    }
  }
}

#[derive(Clone)]
#[napi]
struct JsNfsDirectoryHandle {
  inner: NFSDirectoryHandle,
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
    JsNfsDirectoryHandle{inner: NFSDirectoryHandle::with_initial_name(name.clone()), iter: Value::Null, kind: KIND_DIRECTORY.to_string(), name}
  }

  #[napi(constructor)]
  pub fn new(url: String) -> Self {
    let inner = NFSDirectoryHandle::open(url);
    JsNfsDirectoryHandle{iter: Value::Null, kind: KIND_DIRECTORY.to_string(), name: inner.name.clone(), inner}
  }

  #[napi]
  pub fn to_handle(&self) -> JsNfsHandle {
    let inner = NFSHandle::with_initial_values(self.kind.clone(), self.name.clone());
    JsNfsHandle{kind: inner.kind.clone(), name: inner.name.clone(), inner: inner}
  }

  #[napi]
  pub fn is_same_entry(&self, other: JsNfsHandle) -> napi::Result<bool> {
    self.inner.is_same_entry(other.inner)
  }

  #[napi]
  pub async fn query_permission(&self, perm: JsNfsHandlePermissionDescriptor) -> napi::Result<String> {
    let res = self.inner.query_permission(perm.inner).await?;
    Ok(res)
  }

  #[napi]
  pub async fn request_permission(&self, perm: JsNfsHandlePermissionDescriptor) -> napi::Result<String> {
    let res = self.inner.request_permission(perm.inner).await?;
    Ok(res)
  }

  #[napi(iterator, ts_return_type="AsyncIterableIterator<[string, FileSystemDirectoryHandle | FileSystemFileHandle]>")]
  pub fn entries(&self) -> napi::Result<JsNfsDirectoryHandleEntries> {
    self.inner.entries()
  }

  #[napi(iterator, ts_return_type="AsyncIterableIterator<string>")]
  pub fn keys(&self) -> napi::Result<JsNfsDirectoryHandleKeys> {
    self.inner.keys()
  }

  #[napi(iterator, ts_return_type="AsyncIterableIterator<FileSystemDirectoryHandle | FileSystemFileHandle>")]
  pub fn values(&self) -> napi::Result<JsNfsDirectoryHandleValues> {
    self.inner.values()
  }

  #[napi]
  pub async fn get_directory_handle(&self, name: String, options: Option<JsNfsGetDirectoryOptions>) -> napi::Result<JsNfsDirectoryHandle> {
    let res = self.inner.get_directory_handle(name.clone(), options.and_then(|opt| Some(opt.inner))).await?;
    let ret = JsNfsDirectoryHandle{iter: Value::Null, kind: res.kind.clone(), name: res.name.clone(), inner: res};
    Ok(ret)
  }

  #[napi]
  pub async fn get_file_handle(&self, name: String, options: Option<JsNfsGetFileOptions>) -> napi::Result<JsNfsFileHandle> {
    let res = self.inner.get_file_handle(name.clone(), options.and_then(|opt| Some(opt.inner))).await?;
    let ret = JsNfsFileHandle{kind: res.kind.clone(), name: res.name.clone(), inner: res};
    Ok(ret)
  }

  #[napi]
  pub async fn remove_entry(&self, name: String, options: Option<JsNfsRemoveOptions>) -> napi::Result<()> {
    let ret = self.inner.remove_entry(name, options.and_then(|opt| Some(opt.inner))).await?;
    Ok(ret)
  }

  #[napi]
  pub async fn resolve(&self, possible_descendant: JsNfsHandle) -> napi::Result<Either<Vec<String>, Null>> {
    let ret = self.inner.resolve(possible_descendant.inner).await?;
    Ok(ret)
  }
}

impl FromNapiValue for JsNfsDirectoryHandle {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let obj = from_napi_to_map(env, napi_val)?;
    let kind = obj.get("kind").and_then(|val| Some(val.as_str().unwrap().to_string()));
    // TODO: check whether kind matches KIND_DIRECTORY and, if not, return error?
    let name = obj.get("name").and_then(|val| Some(val.as_str().unwrap().to_string()));
    let res = JsNfsDirectoryHandle::with_initial_name(name.unwrap_or(String::new()));
    Ok(res)
  }
}

impl Into<Value> for JsNfsDirectoryHandle {

  fn into(self) -> Value {
    let mut obj: Map<String, Value> = Map::new();
    obj.insert(String::from("kind"), KIND_DIRECTORY.into());
    obj.insert(String::from("name"), self.name.into());
    Value::Object(obj)
  }
}

#[derive(Clone)]
struct NFSFileHandle {
  handle: NFSHandle,
  kind: String,
  name: String
}

impl NFSFileHandle {

  pub fn new() -> Self {
    Self::with_initial_name(String::new())
  }

  pub fn with_initial_name(name: String) -> Self {
    NFSFileHandle{handle: NFSHandle::with_initial_values(KIND_FILE.to_string(), name.clone()), kind: KIND_FILE.to_string(), name}
  }

  pub fn is_same_entry(&self, other: NFSHandle) -> napi::Result<bool> {
    self.handle.is_same_entry(other)
  }

  pub async fn query_permission(&self, perm: NFSHandlePermissionDescriptor) -> napi::Result<String> {
    let res = self.handle.query_permission(perm).await?;
    Ok(res)
  }

  pub async fn request_permission(&self, perm: NFSHandlePermissionDescriptor) -> napi::Result<String> {
    let res = self.handle.request_permission(perm).await?;
    Ok(res)
  }

  pub async fn get_file(&self) -> napi::Result<NFSFile> {
    let res = NFSFile::with_initial_name(self.name.clone());
    Ok(res)
  }

  pub async fn create_writable(&self, options: Option<JsNfsCreateWritableOptions>) -> napi::Result<JsNfsWritableFileStream> {
    let res = JsNfsWritableFileStream::new();
    Ok(res)
  }
}

#[derive(Clone)]
#[napi]
struct JsNfsFileHandle {
  inner: NFSFileHandle,
  #[napi(readonly, ts_type="'file'")]
  pub kind: String,
  #[napi(readonly)]
  pub name: String
}

#[napi]
impl JsNfsFileHandle {

  pub fn with_initial_name(name: String) -> Self {
    JsNfsFileHandle{inner: NFSFileHandle::with_initial_name(name.clone()), kind: KIND_FILE.to_string(), name}
  }

  #[napi]
  pub fn to_handle(&self) -> JsNfsHandle {
    let inner = NFSHandle::with_initial_values(self.kind.clone(), self.name.clone());
    JsNfsHandle{kind: inner.kind.clone(), name: inner.name.clone(), inner: inner}
  }

  #[napi]
  pub fn is_same_entry(&self, other: JsNfsHandle) -> napi::Result<bool> {
    self.inner.is_same_entry(other.inner)
  }

  #[napi]
  pub async fn query_permission(&self, perm: JsNfsHandlePermissionDescriptor) -> napi::Result<String> {
    let res = self.inner.query_permission(perm.inner).await?;
    Ok(res)
  }

  #[napi]
  pub async fn request_permission(&self, perm: JsNfsHandlePermissionDescriptor) -> napi::Result<String> {
    let res = self.inner.request_permission(perm.inner).await?;
    Ok(res)
  }

  #[napi]
  pub async fn get_file(&self) -> napi::Result<JsNfsFile> {
    let ret = self.inner.get_file().await?;
    let res = JsNfsFile{size: ret.size, _type: ret._type.clone(), last_modified: ret.last_modified, name: ret.name.clone(), webkit_relative_path: ret.webkit_relative_path.clone(), inner: ret};
    Ok(res)
  }

  #[napi]
  pub async fn create_writable(&self, options: Option<JsNfsCreateWritableOptions>) -> napi::Result<JsNfsWritableFileStream> {
    let res = self.inner.create_writable(options).await?;
    Ok(res)
  }
}

impl FromNapiValue for JsNfsFileHandle {

  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let obj = from_napi_to_map(env, napi_val)?;
    let kind = obj.get("kind").and_then(|val| Some(val.as_str().unwrap().to_string()));
    // TODO: check whether kind matches KIND_FILE and, if not, return error?
    let name = obj.get("name").and_then(|val| Some(val.as_str().unwrap().to_string()));
    let res = JsNfsFileHandle::with_initial_name(name.unwrap_or(String::new()));
    Ok(res)
  }
}

impl Into<Value> for JsNfsFileHandle {

  fn into(self) -> Value {
    let mut obj: Map<String, Value> = Map::new();
    obj.insert(String::from("kind"), KIND_FILE.into());
    obj.insert(String::from("name"), self.name.into());
    Value::Object(obj)
  }
}

struct NFSFile {
  size: u32,
  _type: String,
  last_modified: u32,
  name: String,
  webkit_relative_path: String
}

impl NFSFile {

  pub fn with_initial_name(name: String) -> Self {
    NFSFile{
      size: 123,
      _type: String::from("text/plain"),
      last_modified: 1658159058,
      name: name,
      webkit_relative_path: String::from(".")
    }
  }

  pub async fn array_buffer(&self) -> napi::Result<Value> {
    let res = Value::Null;
    Ok(res)
  }

  pub fn slice(&self, start: Option<i32>, end: Option<i32>, content_type: Option<String>) -> NFSBlob {
    NFSBlob{size: self.size, _type: self._type.clone()}.slice(start, end, content_type)
  }

  pub fn stream(&self) -> napi::Result<Value> {
    let res = Value::Null;
    Ok(res)
  }

  pub async fn text(&self) -> napi::Result<String> {
    let res = String::new();
    Ok(res)
  }
}

#[napi]
struct JsNfsFile {
  inner: NFSFile,
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
    let inner = NFSFile::with_initial_name(name);
    JsNfsFile{
      size: inner.size,
      _type: inner._type.clone(),
      last_modified: inner.last_modified,
      name: inner.name.clone(),
      webkit_relative_path: inner.webkit_relative_path.clone(),
      inner
    }
  }

  #[napi(ts_return_type="Promise<ArrayBuffer>")]
  pub async fn array_buffer(&self) -> napi::Result<Value> {
    let res = self.inner.array_buffer().await?;
    Ok(res)
  }

  #[napi]
  pub fn slice(&self, start: Option<i32>, end: Option<i32>, content_type: Option<String>) -> JsNfsBlob {
    let res = self.inner.slice(start, end, content_type);
    JsNfsBlob{_type: res._type.clone(), size: res.size, inner: res}
  }

  #[napi(ts_return_type="ReadableStream<Uint8Array>")]
  pub fn stream(&self) -> napi::Result<Value> {
    self.inner.stream()
  }

  #[napi]
  pub async fn text(&self) -> napi::Result<String> {
    let res = self.inner.text().await?;
    Ok(res)
  }
}

struct NFSBlob {
  size: u32,
  _type: String
}

impl NFSBlob {

  pub async fn array_buffer(&self) -> napi::Result<Value> {
    let res = Value::Null;
    Ok(res)
  }

  pub fn slice(&self, start: Option<i32>, end: Option<i32>, content_type: Option<String>) -> NFSBlob {
    let mut size = self.size;
    if let Some(e) = end {
      if (e.abs() as u32) <= size {
        size = e.abs() as u32;
      }
    }
    if let Some(s) = start {
      if (s.abs() as u32) <= size {
        size = size - (s.abs() as u32);
      }
    }
    NFSBlob{size, _type: content_type.unwrap_or_default()}
  }

  pub fn stream(&self) -> napi::Result<Value> {
    let res = Value::Null;
    Ok(res)
  }

  pub async fn text(&self) -> napi::Result<String> {
    let res = String::new();
    Ok(res)
  }
}

#[napi]
struct JsNfsBlob {
  inner: NFSBlob,
  #[napi(readonly)]
  pub size: u32,
  #[napi(readonly)]
  pub _type: String
}

#[napi]
impl JsNfsBlob {

  #[napi(ts_return_type="Promise<ArrayBuffer>")]
  pub async fn array_buffer(&self) -> napi::Result<Value> {
    let res = self.inner.array_buffer().await?;
    Ok(res)
  }

  #[napi]
  pub fn slice(&self, start: Option<i32>, end: Option<i32>, content_type: Option<String>) -> JsNfsBlob {
    let res = self.inner.slice(start, end, content_type);
    JsNfsBlob{_type: res._type.clone(), size: res.size, inner: res}
  }

  #[napi(ts_return_type="ReadableStream<Uint8Array>")]
  pub fn stream(&self) -> napi::Result<Value> {
    self.inner.stream()
  }

  #[napi]
  pub async fn text(&self) -> napi::Result<String> {
    let res = self.inner.text().await?;
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
    return Err(Error::new(Status::GenericFailure, String::from("Failed to detect whether given js is an array")))
  }

  if is_arr {
    return Err(Error::new(Status::ObjectExpected, String::from("Expect Object, got Array")))
  }

  Map::<String, Value>::from_napi_value(env, napi_val)
}
