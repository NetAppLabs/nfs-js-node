#!/bin/bash

set -e

NFS_TEST_DIR="/tmp/nfs-js-testrun-$RANDOM"
mkdir -p ${NFS_TEST_DIR}

./setup-nfs-testdir.sh ${NFS_TEST_DIR} $(id -u) $(id -g)

NFS_PORT=20490
export NFS_URL="nfs://127.0.0.1${NFS_TEST_DIR}?nfsport=$NFS_PORT&mountport=$NFS_PORT&auto-traverse-mounts=0"

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
