#!/bin/bash

set -e

NFS_UID=$1
NFS_GID=$2
if [ -z $NFS_UID ]; then
    NFS_UID=nobody
fi
if [ -z $NFS_GID ]; then
    NFS_GID=nogroup
fi

mkdir -p /tmp/nfs-js/first /tmp/nfs-js/quatre
echo -n "In order to make sure that this file is exactly 123 bytes in size, I have written this text while watching its chars count." > /tmp/nfs-js/annar
touch /tmp/nfs-js/3 /tmp/nfs-js/first/comment /tmp/nfs-js/quatre/points
chmod 555 /tmp/nfs-js/quatre
chmod 775 /tmp/nfs-js/first
chmod 664 /tmp/nfs-js/annar
chmod 444 /tmp/nfs-js/3
chown -R $NFS_UID:$NFS_GID /tmp/nfs-js
