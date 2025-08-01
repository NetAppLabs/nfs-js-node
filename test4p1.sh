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


# TODO: setup NFSv4.1 server against which to test and add test-sh-4p1 to test in packages.json scripts

export NFS_URL="nfs://192.168.64.2/srv/nfs4share?uid=0&gid=0&version=4.1&rsize=131072"

echo "Test using NFSv4.1 via nfs-rs (and nfs4/nfs4_client)"
yarn test-ava
