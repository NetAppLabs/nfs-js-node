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
TARGET_TRIPLE=$1
TARGET_TRIPLE_FOR_CC=$2

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

git submodule update --recursive --init


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

if ! command -v add-apt-repository 2>&1 >/dev/null ; then
    if command -v apt-get 2>&1 >/dev/null ; then
        sudo apt-get update
        sudo apt-get -y install software-properties-common
    fi
fi

if command -v lsb_release 2>&1 >/dev/null ; then
    lsb_rel_version=`lsb_release -c | grep '^Codename:' | awk -F ' ' '{print $2}'`
    if [ "${lsb_rel_version}" == "focal" ]; then
        # install backported autoconf 2.71 backported for ubuntu 20.04 / focal
        sudo add-apt-repository ppa:savoury1/build-tools -y
        sudo apt-get -y install autoconf2.71
    fi
fi

if ! command -v yacc 2>&1 >/dev/null ; then
    if command -v brew 2>&1 >/dev/null ; then
        brew install byacc
    elif command -v apt-get 2>&1 >/dev/null ; then
        sudo apt-get update
        sudo apt-get install -y byacc
    else
        echo "please install yacc"
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

PROCS=8
YACC="yacc"
if [ "${OS}" == "Darwin" ]; then
    YACC="/opt/homebrew/bin/byacc"
fi
if [ ! -f libnfs/local-install/${TARGET_TRIPLE}/lib/libnfs.a ]; then
    if [ "${OS}" == "Darwin" ]; then
        pushd libnfs &> /dev/null
        CURDIR="$(pwd)"
        INSTALL_DIR="${CURDIR}/local-install/${TARGET_TRIPLE}"
        BUILD_DIR="${CURDIR}/local-build/${TARGET_TRIPLE}"
        mkdir -p "${BUILD_DIR}"
        mkdir -p "${INSTALL_DIR}"
        chmod 775 ./bootstrap
        ./bootstrap
        pushd ${BUILD_DIR} &> /dev/null
        ../../configure \
            --disable-werror \
            --host=${TARGET_TRIPLE_FOR_CC} \
            --prefix="${INSTALL_DIR}" \
            --exec-prefix="${INSTALL_DIR}" \
            CFLAGS="-fPIC -Wno-cast-align -I${CURDIR}" \
            YACC="${YACC}"
        make -j${PROCS} install
        popd &> /dev/null
        popd &> /dev/null
    elif [ "${OS}" == "Linux" ]; then
        MAIN_CURDIR="$(pwd)"

        EXTRA_CFLAGS=""
        EXTRA_LDFLAGS=""

        HOST_ARCH=`uname -m`
        COMPILE_FOR_ARCH=`echo ${TARGET_TRIPLE} | awk -F '-' '{print $1}'`
        CROSS_COMPILE="false"
        if [ "${HOST_ARCH}" != "${COMPILE_FOR_ARCH}" ]; then
            CROSS_COMPILE="true"
        fi

        if [ "${CROSS_COMPILE}" == "true" ]; then
            OPENSSL_INSTALL_DIR="${MAIN_CURDIR}/openssl/local-install/${TARGET_TRIPLE}"
            EXTRA_CFLAGS="-I${OPENSSL_INSTALL_DIR}/include"
            EXTRA_LDFLAGS="-L${OPENSSL_INSTALL_DIR}/lib"
            if [ ! -e openssl ]; then
                mkdir -p ${OPENSSL_INSTALL_DIR}
                echo "building openssl for cross compile"
                if [ "${HOST_ARCH}" == "x86_64" ]; then
                    sudo add-apt-repository -s "deb http://archive.ubuntu.com/ubuntu $(lsb_release -sc) main restricted universe multiverse" -y
                else
                    sudo add-apt-repository -s "deb http://ports.ubuntu.com/ubuntu-ports $(lsb_release -sc) main restricted universe multiverse" -y
                fi
                apt-get source openssl
                OPENSSL_VER=`apt-cache showsrc openssl | grep '^Version' | awk '{print $2}' | awk -F '-' '{print $1}'`
                OPENSSL_SRC_DIR="openssl-${OPENSSL_VER}"
                pushd ${OPENSSL_SRC_DIR}
                ./Configure linux-${COMPILE_FOR_ARCH} --prefix=${OPENSSL_INSTALL_DIR} CC=${COMPILE_FOR_ARCH}-linux-gnu-gcc
                make -j${PROCS}
                make -j${PROCS} install
                popd
            fi
        fi

        if [ ! -e krb5 ]; then
            git clone --branch krb5-1.21.3-final https://github.com/krb5/krb5.git
        fi

        KRB5_INSTALL_DIR="${MAIN_CURDIR}/krb5/local-install/${TARGET_TRIPLE}"

        if [ ! -e krb5/local-install/${TARGET_TRIPLE}/lib/krb5 ]; then
            pushd krb5 &> /dev/null
            CURDIR="$(pwd)"
            INSTALL_DIR="${CURDIR}/local-install/${TARGET_TRIPLE}"
            mkdir -p "${INSTALL_DIR}"
            BUILD_DIR="${CURDIR}/local-build/${TARGET_TRIPLE}"
            mkdir -p "${BUILD_DIR}"
            pushd src &> /dev/null
            # for cross compile to work
            export krb5_cv_attr_constructor_destructor=yes
            export ac_cv_func_regcomp=yes
            export ac_cv_printf_positional=yes
            autoreconf --force
            pushd ${BUILD_DIR} &> /dev/null
            ../../src/configure \
                --host=${TARGET_TRIPLE_FOR_CC} \
                --prefix="${INSTALL_DIR}" \
                --exec-prefix="${INSTALL_DIR}" \
                --enable-static \
                --disable-shared \
                CFLAGS="-fPIC -fcommon -Wno-cast-align ${EXTRA_CFLAGS}" \
                LDFLAGS="${EXTRA_LDFLAGS}"
            make -j${PROCS}
            make install
            popd &> /dev/null
            popd &> /dev/null
            popd &> /dev/null
        fi

        pushd libnfs &> /dev/null
        CURDIR="$(pwd)"
        INSTALL_DIR="${CURDIR}/local-install/${TARGET_TRIPLE}"
        BUILD_DIR="${CURDIR}/local-build/${TARGET_TRIPLE}"
        mkdir -p "${BUILD_DIR}"
        mkdir -p "${INSTALL_DIR}"
        chmod 775 ./bootstrap
        ./bootstrap
        pushd ${BUILD_DIR} &> /dev/null
        ../../configure \
            --disable-werror \
            --host=${TARGET_TRIPLE_FOR_CC} \
            --prefix="${INSTALL_DIR}" \
            --exec-prefix="${INSTALL_DIR}" \
            CFLAGS="-fPIC -Wno-cast-align -I${KRB5_INSTALL_DIR}/include -I${CURDIR}" \
            LDFLAGS="-L${KRB5_INSTALL_DIR}/lib" \
            LIBS="-ldl -lgssapi_krb5 -lkrb5 -lcom_err -lgssrpc -lk5crypto -lkdb5 -lkrad -lkrb5_db2 -lkrb5_k5tls -lkrb5_otp -lkrb5_spake -lkrb5support -lverto -lresolv"
        make -j${PROCS} install
        popd &> /dev/null
        popd &> /dev/null
    fi
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
