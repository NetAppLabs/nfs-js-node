#!/bin/bash

set -e

sudo ./setup-nfs.sh $(id -u) $(id -g)

if `brew -v &> /dev/null`; then
    brew install automake
fi

if [ ! -f /usr/local/lib/libnfs.a ]; then
    if [ -f /usr/lib/libnfs.a ]; then
        ln -s /usr/lib/libnfs.a /usr/local/lib/libnfs.a
    else
        git clone https://github.com/sahlberg/libnfs.git /tmp/libnfs
        pushd /tmp/libnfs
        ./bootstrap
        CFLAGS=-fPIC ./configure
        make
        sudo make install
        popd
    fi
fi

git clone https://github.com/willscott/go-nfs.git /tmp/go-nfs
pushd /tmp/go-nfs
go build ./example/osnfs
popd
