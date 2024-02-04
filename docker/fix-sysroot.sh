#!/bin/bash

# Fix broken symlinks in sysroot.
#
# Some libraries are installed with absolute paths, but they are now
# nested within the /sysroot/ folder.
#
# Inspired by https://capnfabs.net/posts/cross-compiling-rust-apps-linker-shenanigans-multistrap-chroot/

find /sysroot/ -xtype l | while read link; do
	fixed=/sysroot$(readlink $link)
	if [[ -e $fixed ]]; then
		ln -sf $fixed $link
	else
		echo "failed to fix symlink $link" >&2
		exit 1
	fi
done
