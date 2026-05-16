# stream-chunkrec

[![crates.io](https://img.shields.io/crates/v/stream-chunkrec.svg)](https://crates.io/crates/stream-chunkrec)

UTF-8-safe streaming recombiner. Buffers the trailing partial codepoint
between pushes so multi-byte chars never turn into replacement chars.

```rust
use stream_chunkrec::Recombiner;
let mut r = Recombiner::new();
assert_eq!(r.push(&[0x63, 0x61, 0x66, 0xC3]), "caf");  // "café" split
assert_eq!(r.push(&[0xA9]), "é");
```

Zero deps. MIT or Apache-2.0.
