#![deny(clippy::all)]

use std::str::FromStr;

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
    isSameEntry(other: FileSystemHandle): Promise<boolean>;
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
  #[napi(js_name="readonly locked")]
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
  #[napi(js_name="readonly kind", ts_type="'directory' | 'file'")]
  pub kind: String,
  #[napi(js_name="readonly name")]
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
    let res = JsNfsHandle::with_initial_values(kind.unwrap_or_else(|| String::new()), name.unwrap_or_else(|| String::new()));
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
    Self::with_initial_name(String::new())
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

  pub fn entries(&self) -> napi::Result<Value> {
    let res = Value::from_str("{'value': undefined, 'done': true}")?;
    Ok(res)
  }

  pub fn keys(&self) -> napi::Result<Value> {
    let res = Value::from_str("{'value': undefined, 'done': true}")?;
    Ok(res)
  }

  pub fn values(&self) -> napi::Result<Value> {
    let res = Value::from_str("{'value': undefined, 'done': true}")?;
    Ok(res)
  }

  pub async fn get_directory_handle(&self, name: String, options: Option<NFSGetDirectoryOptions>) -> napi::Result<NFSDirectoryHandle> {
    let res = NFSDirectoryHandle::new();
    Ok(res)
  }

  pub async fn get_file_handle(&self, name: String, options: Option<NFSGetFileOptions>) -> napi::Result<NFSFileHandle> {
    let res = NFSFileHandle::new();
    Ok(res)
  }

  pub async fn remove_entry(&self, name: String, options: Option<NFSRemoveOptions>) -> napi::Result<()> {
    Ok(())
  }

  pub async fn resolve(&self, possible_descendant: NFSHandle) -> napi::Result<Either<Vec<String>, Null>> {
    let res: Vec<String>  = Vec::new();
    Ok(napi::Either::A(res))
  }
}

#[derive(Clone)]
#[napi]
struct JsNfsDirectoryHandle {
  inner: NFSDirectoryHandle,
  #[napi(js_name="readonly kind", ts_type="'directory'")]
  pub kind: String,
  #[napi(js_name="readonly name")]
  pub name: String
}

#[napi]
impl JsNfsDirectoryHandle {

  pub fn with_initial_name(name: String) -> Self {
    JsNfsDirectoryHandle{inner: NFSDirectoryHandle::with_initial_name(name.clone()), kind: KIND_DIRECTORY.to_string(), name}
  }

  #[napi(constructor)]
  pub fn new(url: String) -> Self {
    let inner = NFSDirectoryHandle::open(url);
    JsNfsDirectoryHandle{kind: KIND_DIRECTORY.to_string(), name: inner.name.clone(), inner}
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

  #[napi(ts_return_type="AsyncIterableIterator<[string, FileSystemDirectoryHandle | FileSystemFileHandle]>")]
  pub fn entries(&self) -> napi::Result<Value> {
    self.inner.entries()
  }

  #[napi(ts_return_type="AsyncIterableIterator<string>")]
  pub fn keys(&self) -> napi::Result<Value> {
    self.inner.keys()
  }

  #[napi(ts_return_type="AsyncIterableIterator<FileSystemDirectoryHandle | FileSystemFileHandle>")]
  pub fn values(&self) -> napi::Result<Value> {
    self.inner.values()
  }

  #[napi]
  pub async fn get_directory_handle(&self, name: String, options: Option<JsNfsGetDirectoryOptions>) -> napi::Result<JsNfsDirectoryHandle> {
    let res = self.inner.get_directory_handle(name.clone(), options.and_then(|opt| Some(opt.inner))).await?;
    let ret = JsNfsDirectoryHandle{kind: res.kind.clone(), name: res.name.clone(), inner: res};
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

  #[napi(js_name="[Symbol.asyncIterator]", ts_return_type="AsyncIterableIterator<[string, JsNfsDirectoryHandle | JsNfsFileHandle]>")] // XXX: ts_return_type="JsNfsDirectoryHandle['entries']"
  pub fn iter(&self) -> Result<Value> {
    // let res = self.inner.entries();
    let res = Value::from_str("{'value': undefined, 'done': true}")?;
    Ok(res)
  }
}

impl FromNapiValue for JsNfsDirectoryHandle {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let obj = from_napi_to_map(env, napi_val)?;
    let kind = obj.get("kind").and_then(|val| Some(val.as_str().unwrap().to_string()));
    // TODO: check whether kind matches KIND_DIRECTORY and, if not, return error?
    let name = obj.get("name").and_then(|val| Some(val.as_str().unwrap().to_string()));
    let res = JsNfsDirectoryHandle::with_initial_name(name.unwrap_or_else(|| String::new()));
    Ok(res)
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
    let res = NFSFile::new();
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
  #[napi(js_name="readonly kind", ts_type="'file'")]
  pub kind: String,
  #[napi(js_name="readonly name")]
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
    let res = JsNfsFile::new();
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
    let res = JsNfsFileHandle::with_initial_name(name.unwrap_or_else(|| String::new()));
    Ok(res)
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

  pub fn new() -> Self {
    NFSFile{
      size: 0,
      _type: String::new(),
      last_modified: 0,
      name: String::new(),
      webkit_relative_path: String::new()
    }
  }

  pub async fn array_buffer(&self) -> napi::Result<Value> {
    let res = Value::Null;
    Ok(res)
  }

  pub fn slice(&self, start: Option<u32>, end: Option<u32>, content_type: Option<String>) -> NFSBlob {
    NFSBlob{size: 0, _type: String::new()}
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
  #[napi(js_name="readonly size")]
  pub size: u32,
  #[napi(js_name="readonly type")]
  pub _type: String,
  #[napi(js_name="readonly lastModified")]
  pub last_modified: u32,
  #[napi(js_name="readonly name")]
  pub name: String,
  #[napi(js_name="readonly webkitRelativePath")]
  pub webkit_relative_path: String
}

#[napi]
impl JsNfsFile {

  pub fn new() -> Self {
    JsNfsFile{
      inner: NFSFile::new(),
      size: 0,
      _type: String::new(),
      last_modified: 0,
      name: String::new(),
      webkit_relative_path: String::new()
    }
  }

  #[napi(ts_return_type="Promise<ArrayBuffer>")]
  pub async fn array_buffer(&self) -> napi::Result<Value> {
    let res = self.inner.array_buffer().await?;
    Ok(res)
  }

  #[napi]
  pub fn slice(&self, start: Option<u32>, end: Option<u32>, content_type: Option<String>) -> JsNfsBlob {
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

  pub fn slice(&self, start: Option<u32>, end: Option<u32>, content_type: Option<String>) -> NFSBlob {
    NFSBlob{size: 0, _type: String::new()}
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
  #[napi(js_name="readonly size")]
  pub size: u32,
  #[napi(js_name="readonly type")]
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
  pub fn slice(&self, start: Option<u32>, end: Option<u32>, content_type: Option<String>) -> JsNfsBlob {
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

  let obj = Value::Object(Map::<String, Value>::from_napi_value(env, napi_val)?);
  let res = obj.as_object().unwrap();
  Ok(res.clone())
}
