#!/bin/bash

set -e

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
    sudo apt-get update
    sudo apt-get -y install libc-dev
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
    if [ ! -f libnfs/local-install/lib/libnfs.a ]; then
        pushd libnfs
        CURDIR="$(pwd)"
        INSTALL_DIR="${CURDIR}/local-install"
        mkdir -p "${INSTALL_DIR}"
        ./bootstrap
        ./configure --without-libkrb5 --prefix="${INSTALL_DIR}" --exec-prefix="${INSTALL_DIR}" CFLAGS='-fPIC -Wno-cast-align'
        make
        make install
        popd
    fi
fi

if [ ! -d go-nfs ]; then
    git clone https://github.com/willscott/go-nfs.git go-nfs
    if [ ! -f go-nfs/osnfs ]; then
        pushd go-nfs
        go build ./example/osnfs
        popd
    fi
fi