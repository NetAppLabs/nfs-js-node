#![deny(clippy::all)]

//use napi::bindgen_prelude::*;
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
