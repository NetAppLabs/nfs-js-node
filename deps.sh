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
# patch bug in go-nfs which results in bad NFS struct being returned when doing READDIR or READDIRPLUS for empty dir
cat <<EOF > readdir.patch
diff --git a/nfs_onreaddir.go b/nfs_onreaddir.go
index 9297880..cbde15b 100644
--- a/nfs_onreaddir.go
+++ b/nfs_onreaddir.go
@@ -110,10 +110,12 @@ func onReadDir(ctx context.Context, w *response, userHandle Handler) error {
 		return &NFSStatusError{NFSStatusServerFault, err}
 	}

-	if len(entities) > 0 {
-		if err := xdr.Write(writer, uint32(1)); err != nil { //next
-			return &NFSStatusError{NFSStatusServerFault, err}
-		}
+	next := uint32(1)
+	if len(entities) == 0 && obj.Cookie != 0 {
+		next = 0
+	}
+	if err := xdr.Write(writer, next); err != nil { // "next"
+		return &NFSStatusError{NFSStatusServerFault, err}
 	}
 	if obj.Cookie == 0 {
 		// prefix the special "." and ".." entries.
diff --git a/nfs_onreaddirplus.go b/nfs_onreaddirplus.go
index 628f4e7..da3f89e 100644
--- a/nfs_onreaddirplus.go
+++ b/nfs_onreaddirplus.go
@@ -129,10 +129,12 @@ func onReadDirPlus(ctx context.Context, w *response, userHandle Handler) error {
 		return &NFSStatusError{NFSStatusServerFault, err}
 	}

-	if len(entities) > 0 {
-		if err := xdr.Write(writer, uint32(1)); err != nil { //next
-			return &NFSStatusError{NFSStatusServerFault, err}
-		}
+	next := uint32(1)
+	if len(entities) == 0 && obj.Cookie != 0 {
+		next = 0
+	}
+	if err := xdr.Write(writer, next); err != nil { // "next"
+		return &NFSStatusError{NFSStatusServerFault, err}
 	}
 	if obj.Cookie == 0 {
 		// prefix the special "." and ".." entries.
EOF
patch -i readdir.patch
go build ./example/osnfs
popd
