UTF-8 Width
====================

[![Build Status](https://travis-ci.org/magiclen/utf8-width.svg?branch=master)](https://travis-ci.org/magiclen/utf8-width)

To determine the width of a UTF-8 character by providing its first byte.

References: https://tools.ietf.org/html/rfc3629

## Examples

```rust
extern crate utf8_width;

assert_eq!(1, utf8_width::get_width(b'1'));
assert_eq!(3, utf8_width::get_width("中".as_bytes()[0]));
```

## Benchmark

```bash
cargo bench
```

## Crates.io

https://crates.io/crates/utf8-width

## Documentation

https://docs.rs/utf8-width

## License

[MIT](LICENSE)