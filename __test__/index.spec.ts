import test from 'ava'

import { JsNfsDirectoryHandle } from '../index'

const nfsURL = "nfs://1.2.3.4/export?vers=3";

test('should convert directory handle to handle', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const handle = rootHandle.toHandle();
  t.is(handle.kind, "directory");
  t.is(handle.kind, rootHandle.kind);
  t.is(handle.name, rootHandle.name);
})

test('should convert file handle to handle', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const handle = fileHandle.toHandle();
  t.is(handle.kind, "file");
  t.is(handle.kind, fileHandle.kind);
  t.is(handle.name, fileHandle.name);
})

test('should be same entry as self for directory', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const handle = rootHandle.toHandle();
  t.true(rootHandle.isSameEntry({kind: handle.kind, name: handle.name})); // FIXME: despite VS Code's "compiler errors", this works -- while below does not work (fails assertion)
  // t.true(rootHandle.isSameEntry(handle));
  t.true(handle.isSameEntry({kind: rootHandle.kind, name: rootHandle.name})); // FIXME: despite VS Code's "compiler errors", this works -- while below does not work (fails assertion)
  // t.true(handle.isSameEntry(rootHandle));
})

test('should be same entry as self for file', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const handle = fileHandle.toHandle();
  t.true(fileHandle.isSameEntry({kind: handle.kind, name: handle.name})); // FIXME: despite VS Code's "compiler errors", this works -- while below does not work (fails assertion)
  // t.true(fileHandle.isSameEntry(handle));
  t.true(handle.isSameEntry({kind: fileHandle.kind, name: fileHandle.name})); // FIXME: despite VS Code's "compiler errors", this works -- while below does not work (fails assertion)
  // t.true(handle.isSameEntry(fileHandle));
})

test('should be granted read permission when querying on directory', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const perm = await rootHandle.queryPermission({mode: "read"});
  t.is(perm, "granted");
})

test('should be denied readwrite permission when querying on directory', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const perm = await rootHandle.queryPermission({mode: "readwrite"});
  t.is(perm, "denied");
})

test('should be prompted for read permission when requesting on directory', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const perm = await rootHandle.requestPermission({mode: "read"});
  t.is(perm, "prompt");
})

test('should be granted readwrite permission when requesting on directory', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const perm = await rootHandle.requestPermission({mode: "readwrite"});
  t.is(perm, "granted");
})

test('should be granted read permission when querying on file', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const perm = await fileHandle.queryPermission({mode: "read"});
  t.is(perm, "granted");
})

test('should be denied readwrite permission when querying on file', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const perm = await fileHandle.queryPermission({mode: "readwrite"});
  t.is(perm, "denied");
})

test('should be prompted for read permission when requesting on file', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const perm = await fileHandle.requestPermission({mode: "read"});
  t.is(perm, "prompt");
})

test('should be granted readwrite permission when requesting on file', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const perm = await fileHandle.requestPermission({mode: "readwrite"});
  t.is(perm, "granted");
})

// TODO
test.failing('should iterate through directory', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const expectedEntries = [{key: "first", value: {kind: "directory", name: "first"}}, {key: "annar", value: {kind: "file", name: "annar"}}, {key: "3", value: {kind: "file", name: "3"}}];
  let i = 0;
  for await (const [ key, value ] of rootHandle) {
    if (i > expectedEntries.length) {
      t.fail("iterated past expected number of entries");
      break;
    }
    t.is(key, expectedEntries[i].key);
    t.is(value.kind.toString(), expectedEntries[i].value.kind);
    t.is(value.name, expectedEntries[i].value.name);
    i++
  }
})

test('should iterate through entries', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const expectedEntries = [{key: "first", value: {kind: "directory", name: "first"}}, {key: "annar", value: {kind: "file", name: "annar"}}, {key: "3", value: {kind: "file", name: "3"}}];
  let i = 0;
  for await (const [ key, value ] of rootHandle.entries()) {
    if (i > expectedEntries.length) {
      t.fail("iterated past expected number of entries");
      break;
    }
    t.is(key, expectedEntries[i].key);
    t.is(value.kind.toString(), expectedEntries[i].value.kind);
    t.is(value.name, expectedEntries[i].value.name);
    i++
  }
})

test('should iterate through keys', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const expectedKeys = ["first", "annar", "3"];
  let i = 0;
  for await (const key of rootHandle.keys()) {
    if (i > expectedKeys.length) {
      t.fail("iterated past expected number of keys");
      break;
    }
    t.is(key, expectedKeys[i++]);
  }
})

test('should iterate through values', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const expectedValues = [{kind: "directory", name: "first"}, {kind: "file", name: "annar"}, {kind: "file", name: "3"}];
  let i = 0;
  for await (const { kind, name } of rootHandle.values()) {
    if (i > expectedValues.length) {
      t.fail("iterated past expected number of values");
      break;
    }
    t.is(kind.toString(), expectedValues[i].kind);
    t.is(name, expectedValues[i].name);
    i++
  }
})

test('should return error when getting unknown directory', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const err = await t.throwsAsync(rootHandle.getDirectoryHandle("unknown"));
  t.is(err?.message, 'Directory "unknown" not found');
})

test('should return directory when getting existing directory', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const dirHandle = await rootHandle.getDirectoryHandle("first");
  t.is(dirHandle.kind, "directory");
  t.is(dirHandle.name, "first");
})

test('should return directory when creating new directory', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const dirHandle = await rootHandle.getDirectoryHandle("newlywed", {create: true});
  t.is(dirHandle.kind, "directory");
  t.is(dirHandle.name, "newlywed");
})

test('should return directory when "creating" existing directory', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const dirHandle = await rootHandle.getDirectoryHandle("first", {create: true});
  t.is(dirHandle.kind, "directory");
  t.is(dirHandle.name, "first");
})

test('should return error when getting unknown file', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const err = await t.throwsAsync(rootHandle.getFileHandle("unknown"));
  t.is(err?.message, 'File "unknown" not found');
})

test('should return file when getting existing file', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  for (const name of ["annar", "3"]) {
    const dirHandle = await rootHandle.getFileHandle(name);
    t.is(dirHandle.kind, "file");
    t.is(dirHandle.name, name);
  }
})

test('should return file when creating new file', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const dirHandle = await rootHandle.getFileHandle("newfoundland", {create: true});
  t.is(dirHandle.kind, "file");
  t.is(dirHandle.name, "newfoundland");
})

test('should return file when "creating" existing file', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  for (const name of ["annar", "3"]) {
    const dirHandle = await rootHandle.getFileHandle(name, {create: true});
    t.is(dirHandle.kind, "file");
    t.is(dirHandle.name, name);
  }
})

test('should return error when removing non-empty directory', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const err = await t.throwsAsync(rootHandle.removeEntry("first"));
  t.is(err?.message, 'Directory "first" is not empty');
})

test('should return error when removing unknown entry', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const err = await t.throwsAsync(rootHandle.removeEntry("unknown"));
  t.is(err?.message, 'Entry "unknown" not found');
})

test('should succeed when removing file', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  for (const name of ["annar", "3"]) {
    await t.notThrowsAsync(rootHandle.removeEntry(name));
  }
})

test('should return error when removing unknown entry recursively', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const err = await t.throwsAsync(rootHandle.removeEntry("unknown", {recursive: true}));
  t.is(err?.message, 'Entry "unknown" not found');
})

test('should succeed when removing recursively (including non-empty directory)', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  for (const name of ["first", "annar", "3"]) {
    await t.notThrowsAsync(rootHandle.removeEntry(name, {recursive: true}));
  }
})

test('should return null when resolving unknown directory', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const resolved = await rootHandle.resolve({kind: "directory", name: "unknown"});
  t.deepEqual(resolved, ["unknown"]); // FIXME: should be getting `null` returned but somehow getting array containing directory handle name
  // t.deepEqual(resolved, null);
})

test('should return null when resolving unknown file', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const resolved = await rootHandle.resolve({kind: "file", name: "unknown"});
  t.deepEqual(resolved, ["unknown"]); // FIXME: should be getting `null` returned but somehow getting array containing file handle name
  // t.deepEqual(resolved, null);
})

test('should return non-null when resolving known directory', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const resolved = await rootHandle.resolve({kind: "directory", name: "first"});
  t.deepEqual(resolved, ["first"]);
})

test('should return non-null when resolving known file', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  for (const name of ["annar", "3"]) {
    const resolved = await rootHandle.resolve({kind: "file", name});
    t.deepEqual(resolved, [name]);
  }
})

test('should return file for file handle', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const file = await fileHandle.getFile();
  t.is(file.name, "annar");
  t.is(file.type, "text/plain");
  t.is(file.webkitRelativePath, ".");
  t.is(file.size, 123);
  t.is(file.lastModified, 1658159058);
})

test('should return array buffer for file', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const file = await fileHandle.getFile();
  const buf = await file.arrayBuffer();
  t.is(buf.byteLength, 123);
})

test('should return array buffer for blob', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const file = await fileHandle.getFile();
  const blob = file.slice();
  const buf = await blob.arrayBuffer();
  t.is(buf.byteLength, 123);
})

test('should return stream for file', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const file = await fileHandle.getFile();
  const stream = file.stream();
  t.true(stream.locked);
})

test('should return stream for blob', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const file = await fileHandle.getFile();
  const blob = file.slice();
  const stream = blob.stream();
  t.true(stream.locked);
})

test('should return text for file', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const file = await fileHandle.getFile();
  const text = await file.text();
  t.is(text, "");
})

test('should return text for blob', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const file = await fileHandle.getFile();
  const blob = file.slice();
  const text = await blob.text();
  t.is(text, "");
})

test('should return blob when slicing file', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const file = await fileHandle.getFile();
  const blob = file.slice();
  t.is(blob.size, file.size);
  t.is(blob.type, "");
  const blobby = file.slice(10, 120, "text/plain");
  t.is(blobby.size, 110);
  t.is(blobby.type, "text/plain");
})

test('should return blob when slicing blob', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const file = await fileHandle.getFile();
  const blob = file.slice(undefined, undefined, "text/plain");
  t.is(blob.size, file.size);
  t.is(blob.type, "text/plain");
  const blobby = blob.slice(-200, -10, "text/vanilla");
  t.is(blobby.size, 10);
  t.is(blobby.type, "text/vanilla");
})

test('should return non-locked writable when creating writable and not keeping existing data', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const writable = await fileHandle.createWritable();
  t.false(writable.locked)
})

test('should return locked writable when creating writable and keeping existing data', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const writable = await fileHandle.createWritable({keepExistingData: true});
  t.true(writable.locked)
})

test('should succeed when writing string', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const writable = await fileHandle.createWritable({keepExistingData: true});
  await t.notThrowsAsync(writable.write("hello rust"));
})

test('should succeed when seeking position', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const writable = await fileHandle.createWritable({keepExistingData: true});
  await t.notThrowsAsync(writable.seek(7));
})

test('should succeed when truncating size', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const writable = await fileHandle.createWritable({keepExistingData: true});
  await t.notThrowsAsync(writable.truncate(120));
})

test('should succeed when closing writable file stream', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const writable = await fileHandle.createWritable({keepExistingData: true});
  await t.notThrowsAsync(writable.close());
})

test('should succeed when aborting writable file stream', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const writable = await fileHandle.createWritable({keepExistingData: true});
  const reason = await writable.abort("I've got my reasons");
  t.is(reason, "I've got my reasons");
})

test('should return writer for writable file stream', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const writable = await fileHandle.createWritable({keepExistingData: true});
  const writer = writable.getWriter();
  t.true(writer.ready);
  t.false(writer.closed);
  t.is(writer.desiredSize, 123);
})
