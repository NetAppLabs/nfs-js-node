"use strict";
/**
 * Copyright 2025 NetApp Inc. All Rights Reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * SPDX-License-Identifier: Apache-2.0
 */
var _a;
Object.defineProperty(exports, "__esModule", { value: true });
exports.NfsWritableFileStream = exports.NfsFileHandle = exports.NfsDirectoryHandle = exports.NfsHandle = void 0;
const index_1 = require("./index");
class NfsHandle {
    constructor(_jsh) {
        this._jsh = _jsh;
        this.kind = _jsh.kind;
        this.name = _jsh.name;
        this.isFile = _jsh.kind == 'file';
        this.isDirectory = _jsh.kind == 'directory';
    }
    isSameEntry(other) {
        return new Promise(async (resolve, reject) => {
            try {
                resolve(this._jsh.isSameEntry(other._jsh || other));
            }
            catch (reason) {
                reject(reason);
            }
        });
    }
    async queryPermission(perm) {
        return this._jsh.queryPermission(perm);
    }
    async requestPermission(perm) {
        return this._jsh.requestPermission(perm);
    }
}
exports.NfsHandle = NfsHandle;
class NfsDirectoryHandle extends NfsHandle {
    constructor(param) {
        const [url, toWrap] = typeof param === 'string' ? [param] : ['', param];
        const _js = toWrap || new index_1.JsNfsDirectoryHandle(url);
        super(_js.toHandle());
        this[_a] = this.entries;
        this[Symbol.asyncIterator] = this.entries;
        this._js = _js;
        this.kind = 'directory';
        this.isFile = false;
        this.isDirectory = true;
        this.getFile = this.getFileHandle;
        this.getDirectory = this.getDirectoryHandle;
        this.getEntries = this.values;
    }
    async *entries() {
        for await (const [key, value] of this._js.entries()) {
            yield [key, value instanceof index_1.JsNfsDirectoryHandle ? new NfsDirectoryHandle(value) : new NfsFileHandle(value)];
        }
    }
    async *keys() {
        for await (const key of this._js.keys()) {
            yield key;
        }
    }
    async *values() {
        for await (const value of this._js.values()) {
            yield value instanceof index_1.JsNfsDirectoryHandle ? new NfsDirectoryHandle(value) : new NfsFileHandle(value);
        }
    }
    async getDirectoryHandle(name, options) {
        return new Promise(async (resolve, reject) => {
            await this._js.getDirectoryHandle(name, options)
                .then((handle) => resolve(new NfsDirectoryHandle(handle)))
                .catch((reason) => {
                if (reason.message == 'The path supplied exists, but was not an entry of requested type.') {
                    reason.name = 'TypeMismatchError';
                }
                reject(reason);
            });
        });
    }
    async getFileHandle(name, options) {
        return new Promise(async (resolve, reject) => {
            await this._js.getFileHandle(name, options)
                .then((handle) => resolve(new NfsFileHandle(handle)))
                .catch((reason) => {
                if (reason.message == 'The path supplied exists, but was not an entry of requested type.') {
                    reason.name = 'TypeMismatchError';
                }
                reject(reason);
            });
        });
    }
    async removeEntry(name, options) {
        return this._js.removeEntry(name, options);
    }
    async resolve(possibleDescendant) {
        return this._js.resolve(possibleDescendant._jsh || possibleDescendant);
    }
}
exports.NfsDirectoryHandle = NfsDirectoryHandle;
_a = Symbol.asyncIterator;
class NfsFileHandle extends NfsHandle {
    constructor(_js) {
        super(_js.toHandle());
        this._js = _js;
        this.kind = 'file';
        this.isFile = true;
        this.isDirectory = false;
    }
    async getFile() {
        return this._js.getFile();
    }
    async createSyncAccessHandle() {
        throw new Error("createSyncAccessHandle not implemented");
    }
    async createWritable(options) {
        return new Promise(async (resolve, reject) => {
            await this._js.createWritable(options)
                .then((stream) => resolve(new NfsWritableFileStream(stream)))
                .catch((reason) => reject(reason));
        });
    }
}
exports.NfsFileHandle = NfsFileHandle;
class NfsWritableFileStream {
    constructor(_js) {
        this._js = _js;
        this.locked = _js.locked;
    }
    async write(data) {
        return new Promise(async (resolve, reject) => {
            if (data instanceof Blob) {
                data = await data.arrayBuffer();
            }
            else {
                const dat = data;
                if (dat.type === 'write' && dat.data instanceof Blob) {
                    dat.data = await dat.data.arrayBuffer();
                }
            }
            try {
                await this._js.write(data)
                    .then(() => resolve())
                    .catch((reason) => reject(reason));
            }
            catch (reason) {
                reject(reason);
            }
        });
    }
    async seek(position) {
        return this._js.seek(position);
    }
    async truncate(size) {
        return this._js.truncate(size);
    }
    async close() {
        return this._js.close();
    }
    async abort(reason) {
        return new Promise(async (resolve, reject) => {
            await this._js.abort(reason)
                .then((_reason) => resolve())
                .catch((reason) => reject(reason));
        });
    }
    getWriter() {
        const writer = this._js.getWriter();
        this.locked = true;
        writer._releaseLock = writer.releaseLock;
        writer.releaseLock = () => {
            writer._releaseLock();
            this._js.releaseLock();
            this.locked = false;
        };
        return writer;
    }
}
exports.NfsWritableFileStream = NfsWritableFileStream;
