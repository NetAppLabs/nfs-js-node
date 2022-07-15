import test from 'ava'

import { JsNfsDirectoryHandle } from '../index'

const nfs_url = "nfs://1.2.3.4/export?vers=3";

test('should convert directory handle to handle', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfs_url);
  const handle = rootHandle.toHandle();
  t.is(handle.kind, rootHandle.kind);
  t.is(handle.name, rootHandle.name);
})

// test('should be same entry as self for directory', async (t) => {
//   const rootHandle = new JsNfsDirectoryHandle(nfs_url);
//   const handle = rootHandle.toHandle();
//   t.true(rootHandle.isSameEntry(handle));
// })

test('should be granted read permission when querying', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfs_url);
  const perm = await rootHandle.queryPermission({mode: "read"});
  t.is(perm, "granted");
})

test('should be denied readwrite permission when querying', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfs_url);
  const perm = await rootHandle.queryPermission({mode: "readwrite"});
  t.is(perm, "denied");
})

test('should be prompted for read permission when requesting', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfs_url);
  const perm = await rootHandle.requestPermission({mode: "read"});
  t.is(perm, "prompt");
})

test('should be granted readwrite permission when requesting', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfs_url);
  const perm = await rootHandle.requestPermission({mode: "readwrite"});
  t.is(perm, "granted");
})

// TODO
test.skip('should iterate through directory', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfs_url);
  const expectedEntries = [{key: "directory", value: {kind: "directory", name: "first"}}, {key: "file", value: {kind: "file", name: "annar"}}, {key: "file", value: {kind: "file", name: "3"}}];
  let i = 0;
  for await (const [ key, value ] of rootHandle) {
    if (i > expectedEntries.length) {
      t.fail("iterated past expected number of entries");
      break;
    }
    t.is(key, expectedEntries[i].key);
    t.is(value.kind.toString(), expectedEntries[i].value.kind);
    t.is(value.name, expectedEntries[i].value.name);
  }
})

// TODO
test.skip('should iterate through entries', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfs_url);
  const expectedEntries = [{key: "directory", value: {kind: "directory", name: "first"}}, {key: "file", value: {kind: "file", name: "annar"}}, {key: "file", value: {kind: "file", name: "3"}}];
  let i = 0;
  for await (const [ key, value ] of rootHandle.entries()) {
    if (i > expectedEntries.length) {
      t.fail("iterated past expected number of entries");
      break;
    }
    t.is(key, expectedEntries[i].key);
    t.is(value.kind.toString(), expectedEntries[i].value.kind);
    t.is(value.name, expectedEntries[i].value.name);
  }
})

// TODO
test.skip('should iterate through keys', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfs_url);
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

// TODO
test.skip('should iterate through values', async (t) => {
  const rootHandle = new JsNfsDirectoryHandle(nfs_url);
  const expectedValues = [{kind: "directory", name: "first"}, {kind: "file", name: "annar"}, {kind: "file", name: "3"}];
  let i = 0;
  for await (const { kind, name } of rootHandle.values()) {
    if (i > expectedValues.length) {
      t.fail("iterated past expected number of values");
      break;
    }
    t.is(kind.toString(), expectedValues[i].kind);
    t.is(name, expectedValues[i].name);
  }
})
