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

NFS_TEST_DIR="/tmp/nfs-js-testrun-$RANDOM"
mkdir -p ${NFS_TEST_DIR}

./setup-nfs-testdir.sh ${NFS_TEST_DIR} $(id -u) $(id -g)

NFS_PORT=20490
export NFS_URL="nfs://127.0.0.1${NFS_TEST_DIR}?nfsport=$NFS_PORT&mountport=$NFS_PORT&auto-traverse-mounts=0&rsize=2097152"

echo "Test using mocks"
TEST_USING_MOCKS=1 yarn test-ava

./go-nfs/osnfs ${NFS_TEST_DIR} $NFS_PORT &> ./go-nfs/osnfs.log &
GO_NFS_PID=$!

function kill_go_nfs() {
    EXITCODE=$?
	echo "Stopping go-nfs"
	kill $GO_NFS_PID
    if [ $EXITCODE -ne 0 ]; then
        cat /tmp/go-nfs/osnfs.log
    fi
}

trap kill_go_nfs EXIT

echo "Test using NFS via libnfs"
yarn test-ava

echo "Test using NFS via nfs-rs (pure rust NFS implementation)"
TEST_USING_PURE_RUST=1 yarn test-ava
