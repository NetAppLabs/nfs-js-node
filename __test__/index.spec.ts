import test from 'ava'

import { JsNfsDirectoryHandle } from '../index'

const nfsURL = "nfs://127.0.0.1/Users/Shared/nfs/";

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

test('should be granted readwrite permission when querying on directory', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const perm = await rootHandle.queryPermission({mode: "readwrite"});
  t.is(perm, "granted");
})

test('should be granted read permission when requesting on directory', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const perm = await rootHandle.requestPermission({mode: "read"});
  t.is(perm, "granted");
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

test('should be granted readwrite permission when querying on file', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const perm = await fileHandle.queryPermission({mode: "readwrite"});
  t.is(perm, "granted");
})

test('should be granted read permission when requesting on file', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const perm = await fileHandle.requestPermission({mode: "read"});
  t.is(perm, "granted");
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
  const expectedEntries = [
    {key: "3", value: {kind: "file", name: "3"}},
    {key: "annar", value: {kind: "file", name: "annar"}},
    {key: "first", value: {kind: "directory", name: "first"}},
    {key: "..", value: {kind: "directory", name: ".."}},
    {key: ".", value: {kind: "directory", name: "."}},
  ];
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
  const expectedEntries = [
    {key: "3", value: {kind: "file", name: "3"}},
    {key: "annar", value: {kind: "file", name: "annar"}},
    {key: "first", value: {kind: "directory", name: "first"}},
    {key: "..", value: {kind: "directory", name: ".."}},
    {key: ".", value: {kind: "directory", name: "."}},
  ];
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
  const expectedKeys = ["3", "annar", "first", "..", "."];
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
  const expectedValues = [
    {kind: "file", name: "3"},
    {kind: "file", name: "annar"},
    {kind: "directory", name: "first"},
    {kind: "directory", name: ".."},
    {kind: "directory", name: "."},
  ];
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
  await rootHandle.removeEntry(dirHandle.name);
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
  const fileHandle = await rootHandle.getFileHandle("newfoundland", {create: true});
  t.is(fileHandle.kind, "file");
  t.is(fileHandle.name, "newfoundland");
  await rootHandle.removeEntry(fileHandle.name);
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
  const fileHandle = await rootHandle.getFileHandle("doomed", {create: true});
  await t.notThrowsAsync(rootHandle.removeEntry(fileHandle.name));
})

test('should return error when removing unknown entry recursively', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const err = await t.throwsAsync(rootHandle.removeEntry("unknown", {recursive: true}));
  t.is(err?.message, 'Entry "unknown" not found');
})

test('should succeed when removing recursively non-empty directory', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const dirHandle = await rootHandle.getDirectoryHandle("condemned", {create: true});
  await t.notThrowsAsync(dirHandle.getFileHandle("asylum", {create: true}))
  await t.notThrowsAsync(rootHandle.removeEntry(dirHandle.name, {recursive: true}));
})

test('should succeed when removing recursively empty directory', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const dirHandle = await rootHandle.getDirectoryHandle("terminal", {create: true});
  await t.notThrowsAsync(rootHandle.removeEntry(dirHandle.name, {recursive: true}));
})

test('should return null when resolving unknown directory', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const resolved = await rootHandle.resolve({kind: "directory", name: "unknown"});
  t.deepEqual(resolved, null);
})

test('should return null when resolving unknown file', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const resolved = await rootHandle.resolve({kind: "file", name: "unknown"});
  t.deepEqual(resolved, null);
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
  t.true(file.lastModified >= 1658159058);
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
  t.is(text, "In order to make sure that this file is exactly 123 bytes in size, I have written this text while watching its chars count.");
})

test('should return text for blob', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const file = await fileHandle.getFile();
  const blob = file.slice();
  const text = await blob.text();
  t.is(text, "In order to make sure that this file is exactly 123 bytes in size, I have written this text while watching its chars count.");
})

test('should return blob when slicing file', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const file = await fileHandle.getFile();
  const blob = file.slice();
  t.is(blob.size, file.size);
  t.is(blob.type, "");
  const text = await blob.text();
  t.is(text, "In order to make sure that this file is exactly 123 bytes in size, I have written this text while watching its chars count.");
  const blobby = file.slice(12, 65, "text/plain");
  t.is(blobby.size, 53);
  t.is(blobby.type, "text/plain");
  const texty = await blobby.text();
  t.is(texty, "make sure that this file is exactly 123 bytes in size");
})

test('should return blob when slicing blob', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("annar");
  const file = await fileHandle.getFile();
  const blob = file.slice(undefined, undefined, "text/plain");
  t.is(blob.size, file.size);
  t.is(blob.type, "text/plain");
  const text = await blob.text();
  t.is(text, "In order to make sure that this file is exactly 123 bytes in size, I have written this text while watching its chars count.");
  const blobby = blob.slice(-200, -107, "text/vanilla");
  t.is(blobby.size, 16);
  t.is(blobby.type, "text/vanilla");
  const texty = await blobby.text();
  t.is(texty, "In order to make");
})

test('should return non-locked writable when creating writable and not keeping existing data', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("writable-unlocked", {create: true});
  const writable = await fileHandle.createWritable();
  t.false(writable.locked)
  await rootHandle.removeEntry(fileHandle.name);
})

test('should return locked writable when creating writable and keeping existing data', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("writable-locked", {create: true});
  const writable = await fileHandle.createWritable({keepExistingData: true});
  t.true(writable.locked)
  await rootHandle.removeEntry(fileHandle.name);
})

test('should succeed when writing string', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("writable-write-string", {create: true});
  const writable = await fileHandle.createWritable();
  await t.notThrowsAsync(writable.write("hello rust"));
  const file = await fileHandle.getFile();
  t.is(file.size, 10);
  const text = await file.text();
  t.is(text, "hello rust");
  await rootHandle.removeEntry(fileHandle.name);
})

test('should succeed when seeking position', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("writable-seek", {create: true});
  const writable = await fileHandle.createWritable();
  await writable.write("hello rust");
  await t.notThrowsAsync(writable.seek(6));
  await rootHandle.removeEntry(fileHandle.name);
})

test('should succeed when truncating size', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("writable-truncate", {create: true});
  const writable = await fileHandle.createWritable();
  await writable.write("hello rust");
  await t.notThrowsAsync(writable.truncate(5));
  const file = await fileHandle.getFile();
  t.is(file.size, 5);
  const text = await file.text();
  t.is(text, "hello");
  await rootHandle.removeEntry(fileHandle.name);
})

test('should succeed when closing writable file stream', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("writable-close", {create: true});
  const writable = await fileHandle.createWritable();
  await t.notThrowsAsync(writable.close());
  await rootHandle.removeEntry(fileHandle.name);
})

test('should succeed when aborting writable file stream', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("writable-abort", {create: true});
  const writable = await fileHandle.createWritable();
  const reason = await writable.abort("I've got my reasons");
  t.is(reason, "I've got my reasons");
  await rootHandle.removeEntry(fileHandle.name);
})

test('should return writer for writable file stream', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfsURL);
  const fileHandle = await rootHandle.getFileHandle("writable-writer", {create: true});
  const writable = await fileHandle.createWritable();
  const writer = writable.getWriter();
  t.true(writer.ready);
  t.false(writer.closed);
  t.is(writer.desiredSize, 123);
  await rootHandle.removeEntry(fileHandle.name);
})
