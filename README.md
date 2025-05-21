# `@netapplabs/nfs-js-node`

![https://github.com/NetAppLabs/nfs-js-node/actions](https://github.com/NetAppLabs/nfs-js-node/actions/workflows/node.js.yml/badge.svg)

> NFS filesystem implementation for JavaScript/TypeScript.


## Install this package

Add an .npmrc file to your home directory or readable location

```
//npm.pkg.github.com/:_authToken=${GITHUB_TOKEN}
@netapplabs:registry=https://npm.pkg.github.com/
```

```
yarn add @netapplabs/nfs-js-node
```

# Usage

### Example JavasScript usage using NFSv3:

```
import { NfsDirectoryHandle, NfsFileHandle } from '@netapplabs/nfs-js-node'

const nfsUrl="nfs://127.0.0.1/Users/Shared/nfs/?rsize=2097152";
const rootDir = new NfsDirectoryHandle(nfsUrl);
const subPath = "sub-dir";
const subDir = await rootDir.getDirectoryHandle(subPath);
const subFileHandle = await subDir.getFileHandle("sub-file")
const subFile = await subFileHandle.getFile();
const textContents = await subFile.text();
console.log("textContents: ", textContents);
```

## Support matrix

### Operating Systems

|                  | node18 | node20 | node22 |
| ---------------- | ------ | ------ | ------ |
| macOS x64        | ✓      | ✓      | ✓      |
| macOS arm64      | ✓      | ✓      | ✓      |
| Linux x64 gnu    | ✓      | ✓      | ✓      |
| Linux arm64 gnu  | ✓      | ✓      | ✓      |

## Ability

### Build

After `yarn build/npm run build` command, you can see `nfs-js-node.[darwin|linux].node` file in project root. This is the native addon built from [lib.rs](./src/lib.rs).

### Test

With [ava](https://github.com/avajs/ava), run `yarn test/npm run test` to testing native addon. You can also switch to another testing framework if you want.

### CI

With GitHub actions, every commits and pull request will be built and tested automatically.

## Develop requirements

- Install latest `Rust`
  - Install via e.g. `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Install `Node.js@20+` which fully supported `Node-API`
- C compiler (gcc/clang)
- Install `yarn@1.x`

## Test in local

- yarn
- yarn build
- yarn test

And you will see:

```bash
$ ava --verbose

  ✔ test ...
  ─

  x tests passed
✨  Done in 1.12s.
```

## Release package

Ensure you have set you **NPM_TOKEN** in `GitHub` project setting.

In `Settings -> Secrets`, add **NPM_TOKEN** into it.

When you want release package:

```
npm version [<newversion> | major | minor | patch | premajor | preminor | prepatch | prerelease [--preid=<prerelease-id>] | from-git]

git push
```

GitHub actions will do the rest job for you.

## License

[Apache-2.0](LICENSE)

Disclaimer: _This is not an officially supported NetApp product._

## Contributing

See [Contributing.md](./CONTRIBUTING.md)
