## Installing cross compilers to build from macOS and run on Raspberry Pi

See [this article](https://amritrathie.vercel.app/posts/2020/03/06/cross-compiling-rust-from-macos-to-raspberry-pi/) I used for reference.

```
# Add Rust target
rustup target add armv7-unknown-linux-musleabihf
# Install linker
brew install arm-linux-gnueabihf-binutils
# Configure Cargo with the following:
# [target.armv7-unknown-linux-musleabihf]
# linker = "arm-linux-gnueabihf-ld"
echo "[target.armv7-unknown-linux-musleabihf]\nlinker = \"arm-linux-gnueabihf-ld\"" >> ~/.cargo/config
```
