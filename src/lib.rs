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

#[napi]
struct ArrayBuffer {}

#[napi]
struct AsyncIterableIteratorDirectoryEntry {
  pub key: String,
  pub value: JsNfsDirectoryHandle
}

#[napi]
struct AsyncIterableIteratorFileEntry {
  pub key: String,
  pub value: JsNfsFileHandle
}

#[napi]
struct AsyncIterableIteratorEntries {}

#[napi]
impl AsyncIterableIteratorEntries {

  pub fn new() -> Self {
    AsyncIterableIteratorEntries{}
  }

  #[napi]
  pub async fn next(&self) -> napi::Result<Vec<Either<AsyncIterableIteratorDirectoryEntry, AsyncIterableIteratorFileEntry>>> {
    let res: Vec<Either<AsyncIterableIteratorDirectoryEntry, AsyncIterableIteratorFileEntry>> = Vec::new();
    Ok(res)
  }
}

#[napi]
struct AsyncIterableIteratorKeys {}

#[napi]
impl AsyncIterableIteratorKeys {

  pub fn new() -> Self {
    AsyncIterableIteratorKeys{}
  }

  #[napi]
  pub async fn next(&self) -> napi::Result<String> {
    let res = String::new();
    Ok(res)
  }
}

#[napi]
struct AsyncIterableIteratorValues {}

#[napi]
impl AsyncIterableIteratorValues {

  pub fn new() -> Self {
    AsyncIterableIteratorValues{}
  }

  #[napi]
  pub async fn next(&self) -> napi::Result<Either<JsNfsDirectoryHandle, JsNfsFileHandle>> {
    let res = JsNfsDirectoryHandle::new(String::new());
    Ok(napi::Either::A(res))
  }
}

#[napi]
struct ReadableStreamUint8Array {
  pub locked: bool
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
  pub mode: String
}

impl FromNapiValue for JsNfsHandlePermissionDescriptor {

  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let obj = from_napi_to_map(env, napi_val)?;
    let mode = obj["mode"].as_str().unwrap().to_string();
    // TODO: check whether mode matches either PERM_READ or PERM_READWRITE and, if not, return error?
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

  /// Class method
  pub fn is_same_entry(&self, other: NFSHandle) -> napi::Result<bool> {
    let res = false;
    Ok(res)
  }

  pub async fn query_permission(&self, perm: NFSHandlePermissionDescriptor) -> napi::Result<String> {
    let res = String::new();
    Ok(res)
  }

  pub async fn request_permission(&self, perm: NFSHandlePermissionDescriptor) -> napi::Result<String> {
    let res = String::new();
    Ok(res)
  }
}

#[napi]
struct JsNfsHandle {
  inner: NFSHandle,
  pub kind: String,
  pub name: String
}

#[napi]
impl JsNfsHandle {

  pub fn with_initial_values(kind: String, name: String) -> Self {
    JsNfsHandle{inner: NFSHandle::with_initial_values(kind.clone(), name.clone()), kind, name}
  }

  /// Class method
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
    let kind = obj["kind"].as_str().unwrap().to_string();
    let name = obj["name"].as_str().unwrap().to_string();
    let res = JsNfsHandle::with_initial_values(kind, name);
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

  /// Class method
  pub fn is_same_entry(&self, other: NFSHandle) -> napi::Result<bool> {
    let res = false;
    Ok(res)
  }

  pub async fn query_permission(&self, perm: NFSHandlePermissionDescriptor) -> napi::Result<String> {
    let res = String::new();
    Ok(res)
  }

  pub async fn request_permission(&self, perm: NFSHandlePermissionDescriptor) -> napi::Result<String> {
    let res = String::new();
    Ok(res)
  }

  pub fn entries(&self) -> napi::Result<AsyncIterableIteratorEntries> {
    let res = AsyncIterableIteratorEntries{};
    Ok(res)
  }

  pub fn keys(&self) -> napi::Result<AsyncIterableIteratorKeys> {
    let res = AsyncIterableIteratorKeys{};
    Ok(res)
  }

  pub fn values(&self) -> napi::Result<AsyncIterableIteratorValues> {
    let res = AsyncIterableIteratorValues{};
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
  pub kind: String,
  pub name: String
}

#[napi]
impl JsNfsDirectoryHandle {

  pub fn with_initial_name(name: String) -> Self {
    JsNfsDirectoryHandle{inner: NFSDirectoryHandle::with_initial_name(name.clone()), kind: KIND_DIRECTORY.to_string(), name}
  }

  #[napi(constructor)]
  pub fn new(url: String) -> Self {
    JsNfsDirectoryHandle{inner: NFSDirectoryHandle::with_initial_name(String::new()), kind: KIND_DIRECTORY.to_string(), name: String::new()}
  }


  /// Class method
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
  pub fn entries(&self) -> napi::Result<AsyncIterableIteratorEntries> {
    self.inner.entries()
  }

  #[napi]
  pub fn keys(&self) -> napi::Result<AsyncIterableIteratorKeys> {
    self.inner.keys()
  }

  #[napi]
  pub fn values(&self) -> napi::Result<AsyncIterableIteratorValues> {
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
}

impl FromNapiValue for JsNfsDirectoryHandle {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let obj = from_napi_to_map(env, napi_val)?;
    let kind = obj["kind"].as_str().unwrap().to_string();
    // TODO: check whether kind matches KIND_DIRECTORY and, if not, return error?
    let name = obj["name"].as_str().unwrap().to_string();
    let res = JsNfsDirectoryHandle::with_initial_name(name);
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

  /// Class method
  pub fn is_same_entry(&self, other: NFSHandle) -> napi::Result<bool> {
    let res = false;
    Ok(res)
  }

  pub async fn query_permission(&self, perm: NFSHandlePermissionDescriptor) -> napi::Result<String> {
    let res = String::new();
    Ok(res)
  }

  pub async fn request_permission(&self, perm: NFSHandlePermissionDescriptor) -> napi::Result<String> {
    let res = String::new();
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
  pub kind: String,
  pub name: String
}

#[napi]
impl JsNfsFileHandle {

  pub fn with_initial_name(name: String) -> Self {
    JsNfsFileHandle{inner: NFSFileHandle::with_initial_name(name.clone()), kind: KIND_FILE.to_string(), name}
  }

  /// Class method
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
    let kind = obj["kind"].as_str().unwrap().to_string();
    // TODO: check whether kind matches KIND_FILE and, if not, return error?
    let name = obj["name"].as_str().unwrap().to_string();
    let res = JsNfsFileHandle::with_initial_name(name);
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

  /// Class method
  pub async fn array_buffer(&self) -> napi::Result<ArrayBuffer> {
    let res = ArrayBuffer{};
    Ok(res)
  }

  pub fn slice(&self, start: Option<u32>, end: Option<u32>, content_type: Option<String>) -> NFSBlob {
    NFSBlob{size: 0, _type: String::new()}
  }

  pub fn stream(&self) -> ReadableStreamUint8Array {
    ReadableStreamUint8Array{locked: false}
  }

  pub async fn text(&self) -> napi::Result<String> {
    let res = String::new();
    Ok(res)
  }
}

#[napi]
struct JsNfsFile {
  inner: NFSFile,
  pub size: u32,
  pub _type: String,
  pub last_modified: u32,
  pub name: String,
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

  /// Class method
  #[napi]
  pub async fn array_buffer(&self) -> napi::Result<ArrayBuffer> {
    let res = self.inner.array_buffer().await?;
    Ok(res)
  }

  #[napi]
  pub fn slice(&self, start: Option<u32>, end: Option<u32>, content_type: Option<String>) -> JsNfsBlob {
    let res = self.inner.slice(start, end, content_type);
    JsNfsBlob{_type: res._type.clone(), size: res.size, inner: res}
  }

  #[napi]
  pub fn stream(&self) -> ReadableStreamUint8Array {
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

  /// Class method
  pub async fn array_buffer(&self) -> napi::Result<ArrayBuffer> {
    let res = ArrayBuffer{};
    Ok(res)
  }

  pub fn slice(&self, start: Option<u32>, end: Option<u32>, content_type: Option<String>) -> NFSBlob {
    NFSBlob{size: 0, _type: String::new()}
  }

  pub fn stream(&self) -> ReadableStreamUint8Array {
    ReadableStreamUint8Array{locked: false}
  }

  pub async fn text(&self) -> napi::Result<String> {
    let res = String::new();
    Ok(res)
  }
}

#[napi]
struct JsNfsBlob {
  inner: NFSBlob,
  pub size: u32,
  pub _type: String
}

#[napi]
impl JsNfsBlob {

  /// Class method
  #[napi]
  pub async fn array_buffer(&self) -> napi::Result<ArrayBuffer> {
    let res = self.inner.array_buffer().await?;
    Ok(res)
  }

  #[napi]
  pub fn slice(&self, start: Option<u32>, end: Option<u32>, content_type: Option<String>) -> JsNfsBlob {
    let res = self.inner.slice(start, end, content_type);
    JsNfsBlob{_type: res._type.clone(), size: res.size, inner: res}
  }

  #[napi]
  pub fn stream(&self) -> ReadableStreamUint8Array {
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
