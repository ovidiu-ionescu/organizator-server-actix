cargo build --release --target x86_64-unknown-linux-musl

strip target/x86_64-unknown-linux-musl/release/organizator-server

git tag -a v$(head src/version.txt) -F src/version.txt

