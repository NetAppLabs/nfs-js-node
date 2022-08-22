import { JsNfsCreateWritableOptions, JsNfsDirectoryHandle, JsNfsFileHandle, JsNfsGetFileOptions, JsNfsWritableFileStream } from './index'

type NfsDirectoryHandle = JsNfsDirectoryHandle;

export function getRootHandle(nfsURL: string): NfsDirectoryHandle {
  let rootHandle = new JsNfsDirectoryHandle(nfsURL);
  let rootHandleEx = rootHandle as any;
  rootHandleEx[Symbol.asyncIterator] = async function *(): AsyncIterableIterator<[string, JsNfsDirectoryHandle | JsNfsFileHandle]> {
    for await (const [key, value] of rootHandleEx.entries()) {
      yield [key, value];
    }
  }
  rootHandleEx._getFileHandle = rootHandleEx.getFileHandle;
  rootHandleEx.getFileHandle = async (name: string, options?: JsNfsGetFileOptions): Promise<JsNfsFileHandle> => {
    return new Promise(async (resolve, reject) => {
      await rootHandleEx._getFileHandle(name, options)
        .then((fileHandle: JsNfsFileHandle) => {
          let fileHandleEx = fileHandle as any;
          fileHandleEx._createWritable = fileHandleEx.createWritable;
          fileHandleEx.createWritable = async (options?: JsNfsCreateWritableOptions): Promise<JsNfsWritableFileStream> => {
            return new Promise(async (res, rej) => {
              await fileHandleEx._createWritable(options)
                .then((stream: JsNfsWritableFileStream) => {
                  let streamEx = stream as any;
                  streamEx._write = streamEx.write;
                  streamEx.write = async (data: ArrayBuffer | TypedArray | DataView | Blob | String | string | {type: 'write' | 'seek' | 'truncate', data?: ArrayBuffer | TypedArray | DataView | Blob | String | string, position?: number, size?: number}): Promise<void> => {
                    return new Promise(async (r, j) => {
                      if (data instanceof Blob) {
                        data = await data.arrayBuffer();
                      } else if (data.type === "write" && data.data instanceof Blob) {
                        data.data = await data.data.arrayBuffer();
                      }

                      try {
                        await streamEx._write(data)
                        .then(() => r())
                        .catch((reason: any) => j(reason));
                      } catch (error) {
                        j(error);
                      }
                    });
                  };
                  streamEx._getWriter = streamEx.getWriter;
                  streamEx.getWriter = (): WritableStreamDefaultWriter => {
                    let writer = streamEx._getWriter();
                    let writerEx = writer as any;
                    writerEx._releaseLock = writerEx.releaseLock;
                    writerEx.releaseLock = () => {
                      writerEx._releaseLock();
                      streamEx.releaseLock();
                    };
                    return writer;
                  };
                  res(streamEx);
                })
                .catch((reason: any) => rej(reason));
            });
          };
          resolve(fileHandleEx);
        })
        .catch((reason: any) => reject(reason))
    });
  };
  return rootHandle;
}
