
# Cross-compiling for RPi Zero in docker (work in progress).

- Build the docker container
   `docker build docker/ -f docker/pizero.dockerfile -t pizero:local`

- Run cargo build inside the docker container (this will mount the current
  directory inside the container).
   ```
   docker run --rm --mount "type=bind,source=$(pwd),target=/haxo" pizero:local \
      cargo build --target arm-unknown-linux-gnueabihf --release --features midi
   ```

## TODO
- Preserve cargo download cache (by mounting correct volume) 
- Doesn't work with vergen?


## References
- https://capnfabs.net/posts/cross-compiling-rust-apps-linker-shenanigans-multistrap-chroot/
- https://earthly.dev/blog/cross-compiling-raspberry-pi/
