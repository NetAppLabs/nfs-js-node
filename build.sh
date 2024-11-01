#!/bin/bash

set -e

./deps.sh

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

LIBNFS_BASE="${SCRIPT_DIR}/libnfs"
LIBNFS_BASE_INSTALL="${SCRIPT_DIR}/libnfs/local-install"
export LIBNFS_LIB_PATH="${LIBNFS_BASE}/lib/.libs/"
LIBNFS_INCLUDE="${LIBNFS_BASE_INSTALL}/include"
LIBNFS_NFS_INCLUDE="${LIBNFS_BASE}/"

export LIBNFS_LINK_STATIC="true"

export DYLD_LIBRARY_PATH=${LIBNFS_LIB_PATH}:$DYLD_LIBRARY_PATH
export LD_LIBRARY_PATH=${LIBNFS_LIB_PATH}:$LD_LIBRARY_PATH
export C_INCLUDE_PATH="/usr/local/include/:${LIBNFS_INCLUDE}:${LIBNFS_NFS_INCLUDE}"

export RUST_BACKTRACE=1

yarn build-napi
