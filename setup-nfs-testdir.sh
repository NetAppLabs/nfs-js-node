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

NFS_TEST_DIR=$1
NFS_UID=$2
NFS_GID=$3
if [ -z $NFS_UID ]; then
    NFS_UID=nobody
fi
if [ -z $NFS_GID ]; then
    NFS_GID=nogroup
fi

rm -rf ${NFS_TEST_DIR}/*
mkdir -p ${NFS_TEST_DIR}/first ${NFS_TEST_DIR}/quatre
echo -n "In order to make sure that this file is exactly 123 bytes in size, I have written this text while watching its chars count." > ${NFS_TEST_DIR}/annar
touch ${NFS_TEST_DIR}/3 ${NFS_TEST_DIR}/first/comment ${NFS_TEST_DIR}/quatre/points
chmod 555 ${NFS_TEST_DIR}/quatre
chmod 775 ${NFS_TEST_DIR}/first
chmod 664 ${NFS_TEST_DIR}/annar
chmod 444 ${NFS_TEST_DIR}/3
chown -R $NFS_UID:$NFS_GID ${NFS_TEST_DIR}
