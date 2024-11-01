#!/bin/bash

set -e

NFS_TEST_DIR=$1
NFS_UID=$2
NFS_GID=$3
if [ -z $NFS_UID ]; then
    NFS_UID=nobody
fi
if [ -z $NFS_GID ]; then
    NFS_GID=nogroup
fi

mkdir -p ${NFS_TEST_DIR}/first ${NFS_TEST_DIR}/quatre
echo -n "In order to make sure that this file is exactly 123 bytes in size, I have written this text while watching its chars count." > ${NFS_TEST_DIR}/annar
touch ${NFS_TEST_DIR}/3 ${NFS_TEST_DIR}/first/comment ${NFS_TEST_DIR}/quatre/points
chmod 555 ${NFS_TEST_DIR}/quatre
chmod 775 ${NFS_TEST_DIR}/first
chmod 664 ${NFS_TEST_DIR}/annar
chmod 444 ${NFS_TEST_DIR}/3
chown -R $NFS_UID:$NFS_GID ${NFS_TEST_DIR}
