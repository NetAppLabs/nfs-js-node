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
} from "./index.d";

type NfsHandlePermissionDescriptor = JsNfsHandlePermissionDescriptor;
type NfsGetDirectoryOptions = JsNfsGetDirectoryOptions;
type NfsGetFileOptions = JsNfsGetFileOptions;
type NfsRemoveOptions = JsNfsRemoveOptions;
type NfsCreateWritableOptions = JsNfsCreateWritableOptions;

// type TypedArray = Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array | BigInt64Array | BigUint64Array;
type TypedArray = Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array; // FIXME: BigInt64Array and BigUint64Array need ES2020

export class NfsHandle {
  private _jsh: JsNfsHandle
  readonly kind: 'directory' | 'file'
  readonly name: string
  constructor(_jsh: JsNfsHandle) {
    this._jsh = _jsh;
    this.kind = _jsh.kind;
    this.name = _jsh.name;
  }
  isSameEntry(other: NfsHandle): boolean {
    return this._jsh.isSameEntry(other._jsh);
  }
  async queryPermission(perm: NfsHandlePermissionDescriptor): Promise<string> {
    return this._jsh.queryPermission(perm);
  }
  async requestPermission(perm: NfsHandlePermissionDescriptor): Promise<string> {
    return this._jsh.requestPermission(perm);
  }
}

export class NfsDirectoryHandle extends NfsHandle {
  [Symbol.asyncIterator]: NfsDirectoryHandle['entries']
  private _js: JsNfsDirectoryHandle
  constructor(url?: string, toWrap?: JsNfsDirectoryHandle) {
    const _js = toWrap || new JsNfsDirectoryHandle(url || '');
    super(_js.toHandle());
    this._js = _js;
  }
  async *entries(): AsyncIterableIterator<[string, NfsDirectoryHandle | NfsFileHandle]> {
    for await (const [key, value] of this._js.entries()) {
      yield [key, value instanceof JsNfsDirectoryHandle ? new NfsDirectoryHandle(undefined, value) : new NfsFileHandle(value)];
    }
  }
  async *keys(): AsyncIterableIterator<string> {
    for await (const key of this._js.keys()) {
      yield key;
    }
  }
  async *values(): AsyncIterableIterator<NfsDirectoryHandle | NfsFileHandle> {
    for await (const value of this._js.values()) {
      yield value instanceof JsNfsDirectoryHandle ? new NfsDirectoryHandle(undefined, value) : new NfsFileHandle(value);
    }
  }
  async getDirectoryHandle(name: string, options?: NfsGetDirectoryOptions): Promise<NfsDirectoryHandle> {
    return new Promise(async (resolve, reject) => {
      await this._js.getDirectoryHandle(name, options)
        .then((handle) => resolve(new NfsDirectoryHandle(undefined, handle)))
        .catch((reason) => reject(reason));
    });
  }
  async getFileHandle(name: string, options?: NfsGetFileOptions): Promise<NfsFileHandle> {
    return new Promise(async (resolve, reject) => {
      await this._js.getFileHandle(name, options)
        .then((handle) => resolve(new NfsFileHandle(handle)))
        .catch((reason) => reject(reason));
    });
  }
  async removeEntry(name: string, options?: NfsRemoveOptions): Promise<void> {
    return this._js.removeEntry(name, options);
  }
  async resolve(possibleDescendant: NfsHandle): Promise<Array<string> | null> {
    return this._js.resolve(possibleDescendant);
  }
}

export class NfsFileHandle extends NfsHandle {
  private _js: JsNfsFileHandle
  constructor(_js: JsNfsFileHandle) {
    super(_js.toHandle());
    this._js = _js;
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

export class NfsWriteOptions {
  type: 'write' | 'seek' | 'truncate'
  data?: ArrayBuffer | TypedArray | DataView | Blob | String | string
  position?: number
  size?: number
};

interface NfsWritableFileStreamLock { locked: boolean }
export class NfsWritableFileStream implements NfsWritableFileStreamLock {
  private _js: JsNfsWritableFileStream
  readonly locked: boolean
  constructor(_js: JsNfsWritableFileStream) {
    this._js = _js;
    this.locked = _js.locked;
  }
  async write(data: ArrayBuffer | TypedArray | DataView | Blob | String | string | NfsWriteOptions): Promise<void> {
    return new Promise(async (resolve, reject) => {
      if (data instanceof Blob) {
        data = await data.arrayBuffer();
      } else if (data instanceof NfsWriteOptions && data.type === "write" && data.data instanceof Blob) {
        data.data = await data.data.arrayBuffer();
      }

      await this._js.write(data)
        .then(() => resolve())
        .catch((reason) => reject(reason));
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
    return writer;
  }
}
