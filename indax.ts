import {
  JsNfsHandlePermissionDescriptor,
  JsNfsGetDirectoryOptions,
  JsNfsGetFileOptions,
  JsNfsRemoveOptions,
  JsNfsCreateWritableOptions,
  JsNfsHandle,
  JsNfsDirectoryHandle,
  JsNfsFileHandle,
  JsNfsWritableFileStream,
} from './index';

type NfsHandlePermissionDescriptor = JsNfsHandlePermissionDescriptor;
type NfsCreateWritableOptions = JsNfsCreateWritableOptions;

type TypedArray = Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array | BigInt64Array | BigUint64Array;

export class NfsHandle implements FileSystemHandle {
  private _jsh: JsNfsHandle
  readonly kind: FileSystemHandleKind
  readonly name: string
  constructor(_jsh: JsNfsHandle) {
    this._jsh = _jsh;
    this.kind = _jsh.kind;
    this.name = _jsh.name;
  }
  isSameEntry(other: FileSystemHandle): Promise<boolean> {
    return new Promise(async (resolve, reject) => {
      try {
        resolve(this._jsh.isSameEntry((other as any)._jsh || other));
      } catch(reason) {
        reject(reason);
      }
    });
  }
  async queryPermission(perm: NfsHandlePermissionDescriptor): Promise<PermissionState> {
    return this._jsh.queryPermission(perm) as Promise<PermissionState>;
  }
  async requestPermission(perm: NfsHandlePermissionDescriptor): Promise<PermissionState> {
    return this._jsh.requestPermission(perm) as Promise<PermissionState>;
  }
}

export class NfsDirectoryHandle extends NfsHandle implements FileSystemDirectoryHandle {
  [Symbol.asyncIterator]: NfsDirectoryHandle['entries']
  readonly kind: 'directory'
  readonly isFile: false
  readonly isDirectory: true
  private _js: JsNfsDirectoryHandle
  constructor(url: string);
  constructor(toWrap: JsNfsDirectoryHandle);
  constructor(param: string | JsNfsDirectoryHandle) {
    const [url, toWrap] = typeof param === 'string' ? [param] : ['', param];
    const _js = toWrap || new JsNfsDirectoryHandle(url);
    super(_js.toHandle());
    this[Symbol.asyncIterator] = this.entries;
    this._js = _js;
    this.kind = 'directory';
    this.isFile = false;
    this.isDirectory = true;
    this.getFile = this.getFileHandle;
    this.getDirectory = this.getDirectoryHandle;
    this.getEntries = this.values;
  }
  async *entries(): AsyncIterableIterator<[string, FileSystemDirectoryHandle | FileSystemFileHandle]> {
    for await (const [key, value] of this._js.entries()) {
      yield [key, value instanceof JsNfsDirectoryHandle ? new NfsDirectoryHandle(value) as FileSystemDirectoryHandle : new NfsFileHandle(value) as FileSystemFileHandle];
    }
  }
  async *keys(): AsyncIterableIterator<string> {
    for await (const key of this._js.keys()) {
      yield key;
    }
  }
  async *values(): AsyncIterableIterator<FileSystemDirectoryHandle | FileSystemFileHandle> {
    for await (const value of this._js.values()) {
      yield value instanceof JsNfsDirectoryHandle ? new NfsDirectoryHandle(value) as FileSystemDirectoryHandle : new NfsFileHandle(value) as FileSystemFileHandle;
    }
  }
  async getDirectoryHandle(name: string, options?: FileSystemGetDirectoryOptions): Promise<FileSystemDirectoryHandle> {
    return new Promise(async (resolve, reject) => {
      await this._js.getDirectoryHandle(name, options as JsNfsGetDirectoryOptions)
        .then((handle) => resolve(new NfsDirectoryHandle(handle) as FileSystemDirectoryHandle))
        .catch((reason) => reject(reason));
    });
  }
  async getFileHandle(name: string, options?: FileSystemGetFileOptions): Promise<FileSystemFileHandle> {
    return new Promise(async (resolve, reject) => {
      await this._js.getFileHandle(name, options as JsNfsGetFileOptions)
        .then((handle) => resolve(new NfsFileHandle(handle) as FileSystemFileHandle))
        .catch((reason) => reject(reason));
    });
  }
  async removeEntry(name: string, options?: FileSystemRemoveOptions): Promise<void> {
    return this._js.removeEntry(name, options as JsNfsRemoveOptions);
  }
  async resolve(possibleDescendant: FileSystemHandle): Promise<Array<string> | null> {
    return this._js.resolve((possibleDescendant as any)._jsh || possibleDescendant);
  }

  /**
   * @deprecated Old property just for Chromium <=85. Use `.getFileHandle()` in the new API.
   */
  getFile: NfsDirectoryHandle['getFileHandle'];
  /**
  * @deprecated Old property just for Chromium <=85. Use `.getDirectoryHandle()` in the new API.
  */
  getDirectory: NfsDirectoryHandle['getDirectoryHandle'];
  /**
  * @deprecated Old property just for Chromium <=85. Use `.keys()`, `.values()`, `.entries()`, or the directory itself as an async iterable in the new API.
  */
  getEntries: NfsDirectoryHandle['values'];
 }

export class NfsFileHandle extends NfsHandle implements FileSystemFileHandle {
  readonly kind: 'file'
  readonly isFile: true
  readonly isDirectory: false
  private _js: JsNfsFileHandle
  constructor(_js: JsNfsFileHandle) {
    super(_js.toHandle());
    this._js = _js;
    this.kind = 'file';
    this.isFile = true;
    this.isDirectory = false;
  }
  async getFile(): Promise<File> {
    return this._js.getFile();
  }
  async createWritable(options?: NfsCreateWritableOptions): Promise<NfsWritableFileStream> {
    return new Promise(async (resolve, reject) => {
      await this._js.createWritable(options)
        .then((stream) => resolve(new NfsWritableFileStream(stream)))
        .catch((reason) => reject(reason));
    });
  }
}

interface NfsWritableFileStreamLock { locked: boolean }
export class NfsWritableFileStream implements NfsWritableFileStreamLock {
  private _js: JsNfsWritableFileStream
  readonly locked: boolean
  constructor(_js: JsNfsWritableFileStream) {
    this._js = _js;
    this.locked = _js.locked;
  }
  async write(data: ArrayBuffer | TypedArray | DataView | Blob | String | string | {type: 'write' | 'seek' | 'truncate', data?: ArrayBuffer | TypedArray | DataView | Blob | String | string, position?: number, size?: number}): Promise<void> {
    return new Promise(async (resolve, reject) => {
      if (data instanceof Blob) {
        data = await data.arrayBuffer();
      } else {
        const dat = data as any;
        if (dat.type === 'write' && dat.data instanceof Blob) {
          dat.data = await dat.data.arrayBuffer();
        }
      }

      try {
        await this._js.write(data)
          .then(() => resolve())
          .catch((reason) => reject(reason));
      } catch(reason) {
        reject(reason);
      }
    });
  }
  async seek(position: number): Promise<void> {
    return this._js.seek(position);
  }
  async truncate(size: number): Promise<void> {
    return this._js.truncate(size);
  }
  async close(): Promise<void> {
    return this._js.close();
  }
  async abort(reason: string): Promise<string> {
    return this._js.abort(reason);
  }
  getWriter(): WritableStreamDefaultWriter {
    const writer = this._js.getWriter();
    (<NfsWritableFileStreamLock>this).locked = true;
    (<WritableStreamDefaultWriterEx>writer)._releaseLock = writer.releaseLock;
    writer.releaseLock = () => {
      (<WritableStreamDefaultWriterEx>writer)._releaseLock();
      this._js.releaseLock();
      (<NfsWritableFileStreamLock>this).locked = false;
    };
    return writer;
  }
}

interface WritableStreamDefaultWriterEx extends WritableStreamDefaultWriter {
  _releaseLock: () => void
}
