#!/bin/bash

# Fix broken symlinks in sysroot.
#
# Some libraries are installed with absolute paths, but they are now
# nested within the /sysroot/ folder.

find /sysroot/ -xtype l | while read link; do
	fixed=/sysroot$(readlink $link)
	if [[ -e $fixed ]]; then
		ln -sf $fixed $link
	else
		echo "failed to fix symlink $link" >&2
		exit 1
	fi
done
