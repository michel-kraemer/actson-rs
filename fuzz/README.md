Fuzz testing

```sh
cargo install afl
cargo afl build
cargo afl fuzz -i ../tests/fixtures -o out target/debug/actson-fuzz
```
