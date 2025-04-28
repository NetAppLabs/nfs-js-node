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


set -e

./deps.sh

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

LIBNFS_BASE="${SCRIPT_DIR}/libnfs"
LIBNFS_BASE_INSTALL="${SCRIPT_DIR}/libnfs/local-install"
export LIBNFS_LIB_PATH="${LIBNFS_BASE}/lib/.libs/"
export LIBNFS_INCLUDE_PATH="${LIBNFS_BASE_INSTALL}/include"

export LIBNFS_LINK_STATIC="true"

export DYLD_LIBRARY_PATH=${LIBNFS_LIB_PATH}:$DYLD_LIBRARY_PATH
export LD_LIBRARY_PATH=${LIBNFS_LIB_PATH}:$LD_LIBRARY_PATH

export RUST_BACKTRACE=1

yarn build-napi
