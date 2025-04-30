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

if ! command -v git 2>&1 >/dev/null ; then
    if command -v brew 2>&1 >/dev/null ; then
        brew install git
    elif command -v apt-get 2>&1 >/dev/null ; then
        sudo apt-get update
        sudo apt-get install -y git-all
    else
        echo "please install git"
    fi
fi

git submodule update --init


if ! command -v cargo 2>&1 >/dev/null ; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
fi

if ! command -v automake 2>&1 >/dev/null ; then
    if command -v brew 2>&1 >/dev/null ; then
        brew install automake
    elif command -v apt-get 2>&1 >/dev/null ; then
        sudo apt-get update
        sudo apt-get install -y automake
    else
        echo "please install automake"
    fi
fi

OS=`uname -s`
if [ "${OS}" == "Darwin" ]; then
    if ! command -v glibtool 2>&1 >/dev/null ; then
        if command -v brew 2>&1 >/dev/null ; then
            brew install libtool
        fi
    fi
elif [ "${OS}" == "Linux" ]; then
    if ! command -v make 2>&1 >/dev/null ; then
        if command -v apt-get 2>&1 >/dev/null ; then
            sudo apt-get update
            sudo apt-get install -y make
        else
            echo "please install make"
        fi
    fi
    if ! command -v node 2>&1 >/dev/null ; then
        if command -v apt-get 2>&1 >/dev/null ; then
            sudo apt-get update
            curl -sL https://deb.nodesource.com/setup_22.x -o /tmp/nodesource_setup.sh
            chmod 775 /tmp/nodesource_setup.sh
            sudo /tmp/nodesource_setup.sh
            sudo apt-get install nodejs -y
        else
            echo "please install node"
        fi
    fi
    if ! command -v clang 2>&1 >/dev/null ; then
        if command -v apt-get 2>&1 >/dev/null ; then
            sudo apt-get update
            sudo apt-get install -y clang
        else
            echo "please install clang"
        fi
    fi
    if ! command -v yarn 2>&1 >/dev/null ; then
        sudo npm install -g yarn
    fi
    if ! command -v libtoolize 2>&1 >/dev/null ; then
        if command -v apt-get 2>&1 >/dev/null ; then
            sudo apt-get update
            sudo apt-get install -y libtool
        else
            echo "please install libtool"
        fi
    fi
fi


if [ ! -d libnfs ]; then
    git clone https://github.com/sahlberg/libnfs.git libnfs
else
    pushd libnfs &> /dev/null
    GIT_PULL_OUTPUT=$(git pull)
    if [[ ! "$GIT_PULL_OUTPUT" =~ "Already up to date." ]]; then
        echo "new commits pulled for libnfs - triggering rebuild"
        rm -rf local-install
    fi
    popd &> /dev/null
fi

if [ ! -f libnfs/local-install/lib/libnfs.a ]; then
    pushd libnfs &> /dev/null
    CURDIR="$(pwd)"
    INSTALL_DIR="${CURDIR}/local-install"
    mkdir -p "${INSTALL_DIR}"
    ./bootstrap
    ./configure --without-libkrb5 --prefix="${INSTALL_DIR}" --exec-prefix="${INSTALL_DIR}" CFLAGS='-fPIC -Wno-cast-align'
    make
    make install
    popd &> /dev/null
fi

if [ ! -d go-nfs ]; then
    git clone https://github.com/willscott/go-nfs.git go-nfs
else
    pushd go-nfs &> /dev/null
    GIT_PULL_OUTPUT=$(git pull)
    if [[ ! "$GIT_PULL_OUTPUT" =~ "Already up to date." ]]; then
        echo "new commits pulled for go-nfs - triggering rebuild"
        rm -f osnfs
    fi
    popd &> /dev/null
fi

if [ ! -f go-nfs/osnfs ]; then
    pushd go-nfs &> /dev/null
    go build ./example/osnfs
    popd &> /dev/null
fi
