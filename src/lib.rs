#![deny(clippy::all)]

//use napi::bindgen_prelude::*;
use napi_derive::napi;

/*

interface FileSystemHandle {
    readonly kind: FileSystemHandleKind;
    readonly name: string;
    isSameEntry(other: FileSystemHandle): Promise<boolean>;
}

interface FileSystemDirectoryHandle extends FileSystemHandle {
    readonly kind: "directory";
    getDirectoryHandle(name: string, options?: FileSystemGetDirectoryOptions): Promise<FileSystemDirectoryHandle>;
    getFileHandle(name: string, options?: FileSystemGetFileOptions): Promise<FileSystemFileHandle>;
    removeEntry(name: string, options?: FileSystemRemoveOptions): Promise<void>;
    resolve(possibleDescendant: FileSystemHandle): Promise<string[] | null>;
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

struct NFSDirectoryHandle {}

impl NFSDirectoryHandle {

  pub fn new() -> Self {
    return NFSDirectoryHandle{};
  }

  /// Class method
  pub async fn get_directory_handle(&self, name: String) -> napi::Result<NFSDirectoryHandle> {
    let res = NFSDirectoryHandle{};
    return Ok(res);
  }

  pub fn remove_entry(&self, name: String) -> napi::Result<()> {
    return Ok(());
  }
}

#[napi(js_name = "NFSDirectoryHandle")]
struct JsNFSDirectoryHandle {
  inner: NFSDirectoryHandle,
}

#[napi]
impl JsNFSDirectoryHandle {

  #[napi(constructor)]
  pub fn new() -> Self {
    JsNFSDirectoryHandle { inner: NFSDirectoryHandle::new() }
  }


  /// Class method
  #[napi]
  pub async fn get_directory_handle(&self, name: String) -> napi::Result<JsNFSDirectoryHandle> {
    let res = self.inner.get_directory_handle(name).await?;
    let ret = JsNFSDirectoryHandle { inner: res };
    return Ok(ret);
  }

  #[napi]
  pub fn remove_entry(&self, name: String) -> napi::Result<()> {
    self.inner.remove_entry(name)
  }
}
