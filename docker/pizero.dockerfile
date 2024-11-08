FROM --platform=linux/amd64 rust:1.75-bullseye

WORKDIR /setup

RUN apt-get update && apt-get install multistrap gpg --assume-yes

# This is where we'll create the sysroot for compilation.
RUN mkdir /sysroot/

# Install raspberry pi keyring.
# We install it directly into the sysroot so that multistrap can use it.
RUN mkdir -p /sysroot/etc/apt/trusted.gpg.d
RUN curl -sL http://archive.raspbian.org/raspbian.public.key | gpg --import - \
  && gpg --export 9165938D90FDDD2E \
    > /sysroot/etc/apt/trusted.gpg.d/raspbian-archive-keyring.gpg

# Setup the sysroot.
COPY multistrap-config .
RUN multistrap -f ./multistrap-config
# Fix broken symlinks in the sysroot.
COPY fix-sysroot.sh .
RUN ./fix-sysroot.sh

# Download the cross compiler.
RUN mkdir -p /opt/
RUN wget -q -O- https://github.com/tttapa/docker-arm-cross-toolchain/releases/download/1.0.0/x-tools-armv6-rpi-linux-gnueabihf-gcc14.tar.xz \
  | tar xJ -C /opt

# Add the rust target.
RUN rustup target add arm-unknown-linux-gnueabihf

# Setup compilation environment variables.
ENV PKG_CONFIG_LIBDIR_arm_unknown_linux_gnueabihf=/sysroot/usr/lib/arm-linux-gnueabihf/pkgconfig
ENV PKG_CONFIG_SYSROOT_DIR_arm_unknown_linux_gnueabihf=/sysroot/
ENV CARGO_HOME=/cargo
ENV RUSTFLAGS="-C link-arg=--sysroot=/sysroot/ -C linker=/opt/x-tools/armv6-rpi-linux-gnueabihf/bin/armv6-rpi-linux-gnueabihf-gcc"

# Change workdir (this is where the haxo source should get mounted).
WORKDIR /haxo
# Prevent git complaining about "detected dubious ownership in repository"
RUN git config --global --add safe.directory /haxo
