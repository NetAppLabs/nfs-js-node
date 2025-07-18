#!/bin/bash
# Copyright 2025 NetApp Inc. All Rights Reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
# SPDX-License-Identifier: Apache-2.0


set -ex

ARG1="$1"

LOCAL_TARGET_TRIPLE=`rustc --version --verbose | grep ^host | awk -F ' ' '{print $2}'`
TARGET_TRIPLE="${LOCAL_TARGET_TRIPLE}"

if [[ "${ARG1}" == "--target" ]]; then
  ARG2="$2"
  if [ -n "${ARG2}" ]; then
    TARGET_TRIPLE="${ARG2}"
  fi
fi

rustup target add ${TARGET_TRIPLE}
TARGET_TRIPLE_FOR_CC=`echo ${TARGET_TRIPLE} | sed 's/-unknown//g'`

if [[ "${TARGET_TRIPLE}" == *"linux"* ]]; then
  export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/${TARGET_TRIPLE_FOR_CC}/include"
elif [[ "${TARGET_TRIPLE}" == *"darwin"* ]]; then
  TARGET_TRIPLE_FOR_CC=`echo ${TARGET_TRIPLE} | sed 's/aarch64/arm64/g'`
fi

./deps.sh ${TARGET_TRIPLE} ${TARGET_TRIPLE_FOR_CC}

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

LIBNFS_BASE="${SCRIPT_DIR}/libnfs"
LIBNFS_BASE_INSTALL="${SCRIPT_DIR}/libnfs/local-install/${TARGET_TRIPLE}"

NODE_ARCH=`echo ${TARGET_TRIPLE} | awk -F '-' '{print $1}' | sed 's/aarch64/arm64/g' | sed 's/x86_64/x64/g'`
NODE_PLATFORM=`echo ${TARGET_TRIPLE} | awk -F '-' '{print $2}'`
NODE_OS=`echo ${TARGET_TRIPLE} | awk -F '-' '{print $3}'`
NODE_OS_VARIANT=`echo ${TARGET_TRIPLE} | awk -F '-' '{print $4}'`

LIBNFS_BASE_LIB_INSTALL_PATH="${LIBNFS_BASE_INSTALL}/lib"

NFS_JS_LIB_FULL_VER=`cat ${LIBNFS_BASE_LIB_INSTALL_PATH}/pkgconfig/libnfs.pc | grep '^Version:' | awk '{print $2}'`
NFS_JS_LIB_VER=`echo ${NFS_JS_LIB_FULL_VER} | awk -F '.' '{print $1}'`

export LIBNFS_LIB_PATH="./lib/${NODE_OS}/${NODE_ARCH}"
if [ -n "${NODE_OS_VARIANT}" ]; then
  export LIBNFS_LIB_PATH="./lib/${NODE_OS}/${NODE_ARCH}/${NODE_OS_VARIANT}"
fi
mkdir -p ${LIBNFS_LIB_PATH}
if [ "${NODE_OS}" == "darwin" ]; then
  cp -R ${LIBNFS_BASE_LIB_INSTALL_PATH}/libnfs.${NFS_JS_LIB_VER}.dylib ${LIBNFS_LIB_PATH}/
  cp -R ${LIBNFS_BASE_LIB_INSTALL_PATH}/libnfs.dylib ${LIBNFS_LIB_PATH}/
elif [ "${NODE_OS}" == "linux" ]; then
  cp -R ${LIBNFS_BASE_LIB_INSTALL_PATH}/libnfs.so* ${LIBNFS_LIB_PATH}/
fi

export LIBNFS_INCLUDE_PATH="${LIBNFS_BASE_INSTALL}/include"

export LIBNFS_LINK_STATIC="false"

export DYLD_LIBRARY_PATH=${LIBNFS_LIB_PATH}:$DYLD_LIBRARY_PATH
export LD_LIBRARY_PATH=${LIBNFS_LIB_PATH}:$LD_LIBRARY_PATH

export RUST_BACKTRACE=1

if [ "$ARG1" == "test" ]; then
  cargo test --release
else
  yarn build-tsc
  yarn build-napi --target ${TARGET_TRIPLE}

  # amend napi generated index.js a bit so that it plays nicer with esbuild
  for x in `cat index.js | grep -o "nfs-js-node\..*\.node" | sort | uniq`; do
    cat index.js | sed "s/join(__dirname, '$x')/new URL('$x', import.meta.url)/g" > index.js~
    mv index.js{~,}
  done
fi

if [ "${NODE_OS}" == "darwin" ]; then
  # rewrite dylib search path after build for macos
  install_name_tool -change ${LIBNFS_BASE_LIB_INSTALL_PATH}/libnfs.${NFS_JS_LIB_VER}.dylib @loader_path/lib/${NODE_OS}/${NODE_ARCH}/libnfs.${NFS_JS_LIB_VER}.dylib nfs-js-node.${NODE_OS}-${NODE_ARCH}.node
fi
