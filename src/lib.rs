#![deny(clippy::all)]

use std::collections::HashMap;

use napi::bindgen_prelude::*;
use napi_derive::napi;

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

type FileSystemHandleKind = "directory" | "file";

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

interface FileSystemFileHandle extends FileSystemHandle {
    readonly kind: "file";
    getFile(): Promise<File>;
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

#[napi]
#[derive(PartialEq)]
pub enum JsNfsHandleKind {
  file,
  directory
}

#[napi]
#[derive(PartialEq)]
pub enum JsNfsHandlePermissionMode {
  read,
  readwrite
}

pub struct NFSPermissionDescriptor {
  mode: JsNfsHandlePermissionMode
}

impl NFSPermissionDescriptor {

  pub fn with_initial_mode(mode: JsNfsHandlePermissionMode) -> Self {
    NFSPermissionDescriptor{mode}
  }
}

#[napi]
struct JsNfsPermissionDescriptor {
  inner: NFSPermissionDescriptor,
  pub mode: JsNfsHandlePermissionMode
}

#[napi]
impl JsNfsPermissionDescriptor {

  #[napi(factory)]
  pub fn with_initial_mode(mode: JsNfsHandlePermissionMode) -> Self {
    JsNfsPermissionDescriptor{inner: NFSPermissionDescriptor::with_initial_mode(mode), mode}
  }
}

struct NFSGetDirectoryOptions {
  create: Option<bool>
}

impl NFSGetDirectoryOptions {

  pub fn new() -> Self {
    NFSGetDirectoryOptions{create: None}
  }

  pub fn with_initial_create(create: bool) -> Self {
    NFSGetDirectoryOptions{create: Some(create)}
  }
}

#[napi]
struct JsNfsGetDirectoryOptions {
  inner: NFSGetDirectoryOptions,
  pub create: Option<bool>
}

#[napi]
impl JsNfsGetDirectoryOptions {

  #[napi(factory)]
  pub fn with_initial_create(create: bool) -> Self {
    JsNfsGetDirectoryOptions{inner: NFSGetDirectoryOptions::with_initial_create(create), create: Some(create)}
  }

  #[napi(constructor)]
  pub fn new() -> Self {
    JsNfsGetDirectoryOptions{inner: NFSGetDirectoryOptions::new(), create: None}
  }
}

struct NFSGetFileOptions {
  create: Option<bool>
}

impl NFSGetFileOptions {

  pub fn new() -> Self {
    NFSGetFileOptions{create: None}
  }

  pub fn with_initial_create(create: bool) -> Self {
    NFSGetFileOptions{create: Some(create)}
  }
}

#[napi]
struct JsNfsGetFileOptions {
  inner: NFSGetFileOptions,
  pub create: Option<bool>
}

#[napi]
impl JsNfsGetFileOptions {

  #[napi(factory)]
  pub fn with_initial_create(create: bool) -> Self {
    JsNfsGetFileOptions{inner: NFSGetFileOptions::with_initial_create(create), create: Some(create)}
  }

  #[napi(constructor)]
  pub fn new() -> Self {
    JsNfsGetFileOptions{inner: NFSGetFileOptions::new(), create: None}
  }
}

struct NFSHandle {
  kind: JsNfsHandleKind,
  name: String
}

impl NFSHandle {

  pub fn new(kind: JsNfsHandleKind) -> Self {
    Self::with_initial_values(kind, String::new())
  }

  pub fn with_initial_values(kind: JsNfsHandleKind, name: String) -> Self {
    NFSHandle{kind, name}
  }

  /// Class method
  pub fn is_same_entry(&self, other: NFSHandle) -> napi::Result<bool> {
    let res = false;
    Ok(res)
  }

  pub async fn query_permission(&self, perm: NFSPermissionDescriptor) -> napi::Result<String> {
    let res = String::new();
    Ok(res)
  }

  pub async fn request_permission(&self, perm: NFSPermissionDescriptor) -> napi::Result<String> {
    let res = String::new();
    Ok(res)
  }
}

#[napi]
struct JsNfsHandle {
  inner: NFSHandle,
  pub kind: JsNfsHandleKind,
  pub name: String
}

#[napi]
impl JsNfsHandle {

  #[napi(factory)]
  pub fn with_initial_values(kind: JsNfsHandleKind, name: String) -> Self {
    JsNfsHandle{inner: NFSHandle::with_initial_values(kind, name.clone()), kind, name}
  }

  #[napi(constructor)]
  pub fn new() -> Self {
    JsNfsHandle{inner: NFSHandle::new(JsNfsHandleKind::file), kind: JsNfsHandleKind::file, name: String::new()}
  }

  /// Class method
  #[napi]
  pub fn is_same_entry(&self, other: ClassInstance<JsNfsHandle>) -> napi::Result<bool> {
    let res = self.kind == JsNfsHandleKind::directory;
    // TODO
    // self.inner.is_same_entry(other.inner)
    Ok(res)
  }

  // TODO
  // #[napi]
  // pub async fn query_permission(&self, perm: JsObject) -> napi::Result<JsString> {
  //   // let p: &mut JsNfsPermissionDescriptor = FromNapiValue::from_unknown(perm.into_unknown())?;
  //   // let res = self.inner.query_permission(p.inner).await?;
  //   let res = JsString::from_unknown(perm.into_unknown())?;
  //   Ok(res)
  // }

  // TODO
  // #[napi]
  // pub async fn request_permission(&self, perm: ClassInstance<JsNfsPermissionDescriptor>) -> napi::Result<String> {
  //   let res = self.inner.request_permission(perm.inner).await?;
  //   Ok(res)
  // }
}

struct NFSDirectoryHandle {
  handle: NFSHandle,
  kind: JsNfsHandleKind,
  name: String
}

impl NFSDirectoryHandle {

  pub fn new() -> Self {
    NFSDirectoryHandle{handle: NFSHandle::new(JsNfsHandleKind::directory), kind: JsNfsHandleKind::directory, name: String::new()}
  }

  pub fn with_initial_name(name: String) -> Self {
    NFSDirectoryHandle{handle: NFSHandle::with_initial_values(JsNfsHandleKind::directory, name.clone()), kind: JsNfsHandleKind::directory, name}
  }

  /// Class method
  pub async fn entries(&self) -> napi::Result<HashMap<String, String>> {
    let res: HashMap<String, String> = HashMap::new();
    Ok(res)
  }

  pub async fn keys(&self) -> napi::Result<Vec<String>> {
    let res: Vec<String>  = Vec::new();
    Ok(res)
  }

  pub async fn values(&self) -> napi::Result<Vec<String>> {
    let res: Vec<String>  = Vec::new();
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

  pub async fn remove_entry(&self, name: String) -> napi::Result<()> {
    Ok(())
  }

  pub async fn resolve(&self, possible_descendant: NFSHandle) -> napi::Result<Vec<String>> {
    let res: Vec<String>  = Vec::new();
    Ok(res)
  }
}

#[napi]
struct JsNfsDirectoryHandle {
  inner: NFSDirectoryHandle,
  pub kind: JsNfsHandleKind,
  pub name: String
}

#[napi]
impl JsNfsDirectoryHandle {

  #[napi(factory)]
  pub fn with_initial_name(name: String) -> Self {
    JsNfsDirectoryHandle{inner: NFSDirectoryHandle::with_initial_name(name.clone()), kind: JsNfsHandleKind::directory, name}
  }

  #[napi(constructor)]
  pub fn new() -> Self {
    JsNfsDirectoryHandle{inner: NFSDirectoryHandle::new(), kind: JsNfsHandleKind::directory, name: String::new()}
  }


  /// Class method
  #[napi]
  pub async fn entries(&self) -> napi::Result<HashMap<String, String>> {
    let ret = self.inner.entries().await?;
    Ok(ret)
  }

  #[napi]
  pub async fn keys(&self) -> napi::Result<Vec<String>> {
    let ret = self.inner.keys().await?;
    Ok(ret)
  }

  #[napi]
  pub async fn values(&self) -> napi::Result<Vec<String>> {
    let ret = self.inner.values().await?;
    Ok(ret)
  }

  #[napi]
  pub async fn get_directory_handle(&self, name: String) -> napi::Result<JsNfsDirectoryHandle> {
    let res = self.inner.get_directory_handle(name.clone(), None).await?;
    let ret = JsNfsDirectoryHandle{inner: res, kind: JsNfsHandleKind::directory, name}; // XXX: correct to use `name` param for new directory handle name?
    Ok(ret)
  }

  // TODO
  // #[napi]
  // pub async fn get_directory_handle(&self, name: String, options: Option<JsNfsGetDirectoryOptions>) -> napi::Result<JsNfsDirectoryHandle> {
  //   let res = self.inner.get_directory_handle(name, options.and_then(|opt| Some(opt.inner))).await?;
  //   let ret = JsNfsDirectoryHandle{inner: res};
  //   Ok(ret)
  // }

  #[napi]
  pub async fn get_file_handle(&self, name: String) -> napi::Result<JsNfsFileHandle> {
    let res = self.inner.get_file_handle(name.clone(), None).await?;
    let ret = JsNfsFileHandle{inner: res, kind: JsNfsHandleKind::file, name}; // XXX: correct to use `name` param for new file handle name?
    Ok(ret)
  }

  // TODO
  // #[napi]
  // pub async fn get_file_handle(&self, name: String, options: Option<JsNfsGetFileOptions>) -> napi::Result<JsNfsFileHandle> {
  //   let res = self.inner.get_file_handle(name, options.and_then(|opt| Some(opt.inner))).await?;
  //   let ret = JsNfsFileHandle{inner: res};
  //   Ok(ret)
  // }

  #[napi]
  pub async fn remove_entry(&self, name: String) -> napi::Result<()> {
    let ret = self.inner.remove_entry(name).await?;
    Ok(ret)
  }

  // TODO
  // #[napi]
  // pub async fn resolve(&self, possible_descendant: JsNfsHandle) -> napi::Result<Vec<String>> {
  //   let ret = self.inner.resolve(possible_descendant.inner).await?;
  //   Ok(ret)
  // }
}

struct NFSFileHandle {
  handle: NFSHandle,
  kind: JsNfsHandleKind,
  name: String
}

impl NFSFileHandle {

  pub fn new() -> Self {
    NFSFileHandle{handle: NFSHandle::new(JsNfsHandleKind::file), kind: JsNfsHandleKind::file, name: String::new()}
  }

  pub fn with_initial_name(name: String) -> Self {
    NFSFileHandle{handle: NFSHandle::with_initial_values(JsNfsHandleKind::file, name.clone()), kind: JsNfsHandleKind::file, name}
  }

  /// Class method
  pub async fn get_file(&self) -> napi::Result<NFSFile> {
    let res = NFSFile::new();
    Ok(res)
  }
}

#[napi]
struct JsNfsFileHandle {
  inner: NFSFileHandle,
  pub kind: JsNfsHandleKind,
  pub name: String
}

#[napi]
impl JsNfsFileHandle {

  #[napi(factory)]
  pub fn with_initial_name(name: String) -> Self {
    JsNfsFileHandle{inner: NFSFileHandle::with_initial_name(name.clone()), kind: JsNfsHandleKind::file, name}
  }

  #[napi(constructor)]
  pub fn new() -> Self {
    JsNfsFileHandle{inner: NFSFileHandle::new(), kind: JsNfsHandleKind::file, name: String::new()}
  }

  /// Class method
  #[napi]
  pub async fn get_file(&self) -> napi::Result<JsNfsFile> {
    let ret = self.inner.get_file().await?;
    let res = JsNfsFile::new();
    Ok(res)
  }
}

struct NFSFile {
  size: u32,
  r#type: String,
  last_modified: u32,
  name: String,
  webkit_relative_path: String
}

impl NFSFile {

  pub fn new() -> Self {
    NFSFile{
      size: 0,
      r#type: String::new(),
      last_modified: 0,
      name: String::new(),
      webkit_relative_path: String::new()
    }
  }

  pub fn with_initial_values(size: u32, r#type: String, last_modified: u32, name: String, webkit_relative_path: String) -> Self {
    NFSFile{size, r#type, last_modified, name, webkit_relative_path}
  }

  /// Class method
  // TODO
  // pub async fn array_buffer(&self) -> napi::Result<JsArrayBuffer> {
  //   todo!()
  // }

  pub fn slice(&self, start: Option<u32>, end: Option<u32>, content_type: Option<String>) -> NFSBlob {
    NFSBlob{ size: 0, r#type: String::new()}
  }

  // TODO
  // pub fn stream(&self) -> ReadableStream<Uint8Array> {
  //   todo!()
  // }

  pub async fn text(&self) -> napi::Result<String> {
    let res = String::new();
    Ok(res)
  }
}

#[napi]
struct JsNfsFile {
  inner: NFSFile,
  pub size: u32,
  pub Type: String, // XXX: can't name it `type`
  pub last_modified: u32,
  pub name: String,
  pub webkit_relative_path: String
}

#[napi]
impl JsNfsFile {

  // XXX: does it make sense to generate factory for JsNfsFile?
  // #[napi(factory)]
  pub fn with_initial_values(size: u32, Type: String, last_modified: u32, name: String, webkit_relative_path: String) -> Self {
    JsNfsFile{
      inner: NFSFile::with_initial_values(size, Type.clone(), last_modified, name.clone(), webkit_relative_path.clone()),
      size,
      Type,
      last_modified,
      name,
      webkit_relative_path
    }
  }

  // XXX: does it make sense to generate constructor for JsNfsFile?
  // #[napi(constructor)]
  pub fn new() -> Self {
    JsNfsFile{
      inner: NFSFile::new(),
      size: 0,
      Type: String::new(),
      last_modified: 0,
      name: String::new(),
      webkit_relative_path: String::new()
    }
  }

  /// Class method
  // TODO
  // #[napi]
  // pub async fn array_buffer(&self) -> napi::Result<JsArrayBuffer> {
  //   todo!()
  // }

  #[napi]
  pub fn slice(&self, start: Option<u32>, end: Option<u32>, content_type: Option<String>) -> JsNfsBlob {
    let res = self.inner.slice(start, end, content_type);
    JsNfsBlob{Type: res.r#type.clone(), size: res.size, inner: res}
  }

  // TODO
  // #[napi]
  // pub fn stream(&self) -> ReadableStream<Uint8Array> {
  //   todo!()
  // }

  #[napi]
  pub async fn text(&self) -> napi::Result<String> {
    let res = self.inner.text().await?;
    Ok(res)
  }
}

struct NFSBlob {
  size: u32,
  r#type: String
}

impl NFSBlob {

  pub fn new() -> Self {
    NFSBlob{
      size: 0,
      r#type: String::new()
    }
  }

  pub fn with_initial_values(size: u32, r#type: String) -> Self {
    NFSBlob{size, r#type}
  }

  /// Class method
  // TODO
  // pub async fn array_buffer(&self) -> napi::Result<JsArrayBuffer> {
  //   todo!()
  // }

  pub fn slice(&self, start: Option<u32>, end: Option<u32>, content_type: Option<String>) -> NFSBlob {
    NFSBlob{size: 0, r#type: String::new()}
  }

  // TODO
  // pub fn stream(&self) -> ReadableStream<Uint8Array> {
  //   todo!()
  // }

  pub async fn text(&self) -> napi::Result<String> {
    let res = String::new();
    Ok(res)
  }
}

#[napi]
struct JsNfsBlob {
  inner: NFSBlob,
  pub size: u32,
  pub Type: String // XXX: can't name it `type`
}

#[napi]
impl JsNfsBlob {

  // XXX: does it make sense to generate factory for JsNfsBlob?
  // #[napi(factory)]
  pub fn with_initial_values(size: u32, Type: String) -> Self {
    JsNfsBlob{
      inner: NFSBlob::with_initial_values(size, Type.clone()),
      size,
      Type
    }
  }

  // XXX: does it make sense to generate constructor for JsNfsBlob?
  // #[napi(constructor)]
  pub fn new() -> Self {
    JsNfsBlob{
      inner: NFSBlob::new(),
      size: 0,
      Type: String::new()
    }
  }

  /// Class method
  // TODO
  // #[napi]
  // pub async fn array_buffer(&self) -> napi::Result<JsArrayBuffer> {
  //   todo!()
  // }

  #[napi]
  pub fn slice(&self, start: Option<u32>, end: Option<u32>, content_type: Option<String>) -> JsNfsBlob {
    let res = self.inner.slice(start, end, content_type);
    JsNfsBlob{Type: res.r#type.clone(), size: res.size, inner: res}
  }

  // TODO
  // #[napi]
  // pub fn stream(&self) -> ReadableStream<Uint8Array> {
  //   todo!()
  // }

  #[napi]
  pub async fn text(&self) -> napi::Result<String> {
    let res = self.inner.text().await?;
    Ok(res)
  }
}
