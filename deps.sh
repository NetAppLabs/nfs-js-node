#!/bin/bash

set -e

git submodule update --init

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

if ! command -v libtool 2>&1 >/dev/null ; then
    if command -v brew 2>&1 >/dev/null ; then
        brew install libtool
    elif command -v apt-get 2>&1 >/dev/null ; then
        sudo apt-get update
        sudo apt-get install -y libtool
    else
        echo "please install libtool"
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