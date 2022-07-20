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
        entry.push("first".into());
        entry.push(JsNfsDirectoryHandle::with_initial_name("first".to_string()).into());
        Some(entry)
      },
      2 => {
        let mut entry: Vec<Value> = Vec::new();
        entry.push("annar".into());
        entry.push(JsNfsFileHandle::with_initial_name("annar".to_string()).into());
        Some(entry)
      },
      3 => {
        let mut entry: Vec<Value> = Vec::new();
        entry.push("3".into());
        entry.push(JsNfsFileHandle::with_initial_name("3".to_string()).into());
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
      1 => Some("first".to_string()),
      2 => Some("annar".to_string()),
      3 => Some("3".to_string()),
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
      1 => Some(napi::Either::A(JsNfsDirectoryHandle::with_initial_name("first".to_string()))),
      2 => Some(napi::Either::B(JsNfsFileHandle::with_initial_name("annar".to_string()))),
      3 => Some(napi::Either::B(JsNfsFileHandle::with_initial_name("3".to_string()))),
      _ => None
    }
  }
}

#[napi]
struct JsNfsWritableFileStream {
  #[napi(readonly)]
  pub locked: bool
}

#[napi]
impl JsNfsWritableFileStream {

  pub fn new() -> Self {
    Self::with_initial_locked(false)
  }

  pub fn with_initial_locked(locked: bool) -> Self {
    JsNfsWritableFileStream{locked}
  }

  #[napi]
  pub async fn write(&self, data: Value) -> napi::Result<Undefined> {
    Ok(())
  }

  #[napi]
  pub async fn seek(&self, position: u32) -> napi::Result<Undefined> {
    Ok(())
  }

  #[napi]
  pub async fn truncate(&self, size: u32) -> napi::Result<Undefined> {
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
    obj.insert("ready".to_string(), true.into());
    obj.insert("closed".to_string(), false.into());
    obj.insert("desiredSize".to_string(), 123.into());
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
    let create = obj.get(FIELD_CREATE).and_then(|val| Some(val.as_bool().unwrap()));
    let res = JsNfsGetDirectoryOptions{create: create.unwrap_or_default()};
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
    let create = obj.get(FIELD_CREATE).and_then(|val| Some(val.as_bool().unwrap()));
    let res = JsNfsGetFileOptions{create: create.unwrap_or_default()};
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
    let recursive = obj.get(FIELD_RECURSIVE).and_then(|val| Some(val.as_bool().unwrap()));
    let res = JsNfsRemoveOptions{recursive: recursive.unwrap_or_default()};
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
    let keep_existing_data = obj.get(FIELD_KEEP_EXISTING_DATA).and_then(|val| Some(val.as_bool().unwrap()));
    let res = JsNfsCreateWritableOptions{keep_existing_data: keep_existing_data.unwrap_or_default()};
    Ok(res)
  }
}

fn should_keep_existing_data(options: Option<JsNfsCreateWritableOptions>) -> bool {
  if let Some(opt) = options {
    return opt.keep_existing_data
  }
  false
}

#[napi]
struct JsNfsHandle {
  #[napi(readonly, ts_type="'directory' | 'file'")]
  pub kind: String,
  #[napi(readonly)]
  pub name: String
}

#[napi]
impl JsNfsHandle {

  pub fn new(kind: String) -> Self {
    Self::with_initial_values(kind, String::new())
  }

  pub fn with_initial_values(kind: String, name: String) -> Self {
    JsNfsHandle{kind, name}
  }

  #[napi]
  pub fn is_same_entry(&self, other: JsNfsHandle) -> napi::Result<bool> {
    let res = other.kind == self.kind && other.name == self.name;
    Ok(res)
  }

  #[napi]
  pub async fn query_permission(&self, perm: JsNfsHandlePermissionDescriptor) -> napi::Result<String> {
    match perm.mode.as_str() {
      PERM_READ => Ok(PERM_STATE_GRANTED.to_string()),
      PERM_READWRITE => Ok(PERM_STATE_DENIED.to_string()),
      _ => Ok(PERM_STATE_PROMPT.to_string())
    }
  }

  #[napi]
  pub async fn request_permission(&self, perm: JsNfsHandlePermissionDescriptor) -> napi::Result<String> {
    match perm.mode.as_str() {
      PERM_READ => Ok(PERM_STATE_PROMPT.to_string()),
      PERM_READWRITE => Ok(PERM_STATE_GRANTED.to_string()),
      _ => Ok(PERM_STATE_DENIED.to_string())
    }
  }
}

impl FromNapiValue for JsNfsHandle {

  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let obj = from_napi_to_map(env, napi_val)?;
    let kind = obj.get(FIELD_KIND).and_then(|val| Some(val.as_str().unwrap().to_string()));
    let name = obj.get(FIELD_NAME).and_then(|val| Some(val.as_str().unwrap().to_string()));
    let res = JsNfsHandle::with_initial_values(kind.unwrap_or_default(), name.unwrap_or_default());
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

  pub fn new() -> Self {
    Self::with_initial_name(String::new())
  }

  pub fn with_initial_name(name: String) -> Self {
    JsNfsDirectoryHandle{handle: JsNfsHandle::with_initial_values(KIND_DIRECTORY.to_string(), name.clone()), iter: Value::Null, kind: KIND_DIRECTORY.to_string(), name}
  }

  #[napi(constructor)]
  pub fn open(url: String) -> Self {
    // TODO: use url param
    Self::with_initial_name(url)
  }

  #[napi]
  pub fn to_handle(&self) -> JsNfsHandle {
    JsNfsHandle::with_initial_values(self.kind.clone(), self.name.clone())
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

  #[napi(iterator, ts_return_type="AsyncIterableIterator<[string, FileSystemDirectoryHandle | FileSystemFileHandle]>")]
  pub fn entries(&self) -> napi::Result<JsNfsDirectoryHandleEntries> {
    let res = JsNfsDirectoryHandleEntries{count: 0};
    Ok(res)
  }

  #[napi(iterator, ts_return_type="AsyncIterableIterator<string>")]
  pub fn keys(&self) -> napi::Result<JsNfsDirectoryHandleKeys> {
    let res = JsNfsDirectoryHandleKeys{count: 0};
    Ok(res)
  }

  #[napi(iterator, ts_return_type="AsyncIterableIterator<FileSystemDirectoryHandle | FileSystemFileHandle>")]
  pub fn values(&self) -> napi::Result<JsNfsDirectoryHandleValues> {
    let res = JsNfsDirectoryHandleValues{count: 0};
    Ok(res)
  }

  #[napi]
  pub async fn get_directory_handle(&self, name: String, options: Option<JsNfsGetDirectoryOptions>) -> napi::Result<JsNfsDirectoryHandle> {
    let create = should_create_directory(options);
    if name.ne("first") && !create {
      return Err(Error::new(Status::GenericFailure, format!("Directory {:?} not found", name)));
    }
    let res = JsNfsDirectoryHandle::with_initial_name(name);
    Ok(res)
  }

  #[napi]
  pub async fn get_file_handle(&self, name: String, options: Option<JsNfsGetFileOptions>) -> napi::Result<JsNfsFileHandle> {
    let create = should_create_file(options);
    if name.ne("annar") && name.ne("3") && !create {
      return Err(Error::new(Status::GenericFailure, format!("File {:?} not found", name)));
    }
    let res = JsNfsFileHandle::with_initial_name(name);
    Ok(res)
  }

  #[napi]
  pub async fn remove_entry(&self, name: String, options: Option<JsNfsRemoveOptions>) -> napi::Result<()> {
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

  #[napi]
  pub async fn resolve(&self, possible_descendant: JsNfsHandle) -> napi::Result<Either<Vec<String>, Null>> {
    let first = JsNfsHandle{kind: KIND_DIRECTORY.to_string(), name: "first".to_string()};
    let annar = JsNfsHandle{kind: KIND_FILE.to_string(), name: "annar".to_string()};
    let three = JsNfsHandle{kind: KIND_FILE.to_string(), name: "3".to_string()};
    match possible_descendant {
      first => {
        let mut res: Vec<String> = Vec::new();
        res.push(first.name.clone());
        Ok(napi::Either::A(res))
      },
      annar => {
        let mut res: Vec<String> = Vec::new();
        res.push(annar.name.clone());
        Ok(napi::Either::A(res))
      },
      three => {
        let mut res: Vec<String> = Vec::new();
        res.push(three.name.clone());
        Ok(napi::Either::A(res))
      },
      _ => Ok(napi::Either::B(Null))
    }
  }
}

impl FromNapiValue for JsNfsDirectoryHandle {

  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let obj = from_napi_to_map(env, napi_val)?;
    let kind = obj.get(FIELD_KIND).and_then(|val| Some(val.as_str().unwrap().to_string()));
    // TODO: check whether kind matches KIND_DIRECTORY and, if not, return error?
    let name = obj.get(FIELD_NAME).and_then(|val| Some(val.as_str().unwrap().to_string()));
    let res = JsNfsDirectoryHandle::with_initial_name(name.unwrap_or_default());
    Ok(res)
  }
}

impl Into<Value> for JsNfsDirectoryHandle {

  fn into(self) -> Value {
    let mut obj: Map<String, Value> = Map::new();
    obj.insert(FIELD_KIND.to_string(), KIND_DIRECTORY.into());
    obj.insert(FIELD_NAME.to_string(), self.name.into());
    Value::Object(obj)
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

  pub fn new() -> Self {
    Self::with_initial_name(String::new())
  }

  pub fn with_initial_name(name: String) -> Self {
    JsNfsFileHandle{handle: JsNfsHandle::with_initial_values(KIND_FILE.to_string(), name.clone()), kind: KIND_FILE.to_string(), name}
  }

  #[napi]
  pub fn to_handle(&self) -> JsNfsHandle {
    JsNfsHandle::with_initial_values(self.kind.clone(), self.name.clone())
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
    let res = JsNfsFile::with_initial_name(self.name.clone());
    Ok(res)
  }

  #[napi]
  pub async fn create_writable(&self, options: Option<JsNfsCreateWritableOptions>) -> napi::Result<JsNfsWritableFileStream> {
    let keep_existing_data = should_keep_existing_data(options);
    let res = JsNfsWritableFileStream::with_initial_locked(keep_existing_data);
    Ok(res)
  }
}

impl FromNapiValue for JsNfsFileHandle {

  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let obj = from_napi_to_map(env, napi_val)?;
    let kind = obj.get(FIELD_KIND).and_then(|val| Some(val.as_str().unwrap().to_string()));
    // TODO: check whether kind matches KIND_FILE and, if not, return error?
    let name = obj.get(FIELD_NAME).and_then(|val| Some(val.as_str().unwrap().to_string()));
    let res = JsNfsFileHandle::with_initial_name(name.unwrap_or_default());
    Ok(res)
  }
}

impl Into<Value> for JsNfsFileHandle {

  fn into(self) -> Value {
    let mut obj: Map<String, Value> = Map::new();
    obj.insert(FIELD_KIND.to_string(), KIND_FILE.into());
    obj.insert(FIELD_NAME.to_string(), self.name.into());
    Value::Object(obj)
  }
}

#[napi]
struct JsNfsFile {
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
    JsNfsFile{
      size: 123,
      _type: "text/plain".to_string(),
      last_modified: 1658159058,
      name: name,
      webkit_relative_path: ".".to_string()
    }
  }

  #[napi(ts_return_type="Promise<ArrayBuffer>")]
  pub async fn array_buffer(&self) -> napi::Result<Value> {
    let res = JsNfsBlob{size: self.size, _type: self._type.clone()}.array_buffer().await?;
    Ok(res)
  }

  #[napi]
  pub fn slice(&self, start: Option<i32>, end: Option<i32>, content_type: Option<String>) -> JsNfsBlob {
    JsNfsBlob{size: self.size, _type: self._type.clone()}.slice(start, end, content_type)
  }

  #[napi(ts_return_type="ReadableStream<Uint8Array>")]
  pub fn stream(&self) -> napi::Result<Value> {
    JsNfsBlob{size: self.size, _type: self._type.clone()}.stream()
  }

  #[napi]
  pub async fn text(&self) -> napi::Result<String> {
    let res = JsNfsBlob{size: self.size, _type: self._type.clone()}.text().await?;
    Ok(res)
  }
}

#[napi]
struct JsNfsBlob {
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
    obj.insert("byteLength".to_string(), self.size.into());
    let res = Value::Object(obj);
    Ok(res)
  }

  #[napi]
  pub fn slice(&self, start: Option<i32>, end: Option<i32>, content_type: Option<String>) -> JsNfsBlob {
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
    JsNfsBlob{size, _type: content_type.unwrap_or_default()}
  }

  #[napi(ts_return_type="ReadableStream<Uint8Array>")]
  pub fn stream(&self) -> napi::Result<Value> {
    let mut obj: Map<String, Value> = Map::new();
    obj.insert("locked".to_string(), true.into());
    let res = Value::Object(obj);
    Ok(res)
  }

  #[napi]
  pub async fn text(&self) -> napi::Result<String> {
    let res = String::new();
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
