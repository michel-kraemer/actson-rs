Fuzz testing

```sh
cargo install cargo-afl
cargo afl build
cargo afl fuzz -i ../tests/fixtures -o out target/debug/actson-fuzz
```
