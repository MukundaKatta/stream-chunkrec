//! # stream-chunkrec
//!
//! Recombine streaming token deltas (bytes from a server-sent-event
//! body, an HTTP chunk transfer, or any other framed source) into
//! valid UTF-8 text.
//!
//! Streams that arrive byte-by-byte can split a multi-byte UTF-8
//! sequence across two chunks. Naive `String::from_utf8_lossy(chunk)`
//! turns the split fragments into replacement characters.
//!
//! This crate buffers the trailing partial codepoint between pushes
//! and emits only the bytes that resolve into whole characters.
//!
//! ## Example
//!
//! ```
//! use stream_chunkrec::Recombiner;
//! let mut r = Recombiner::new();
//! // "café" = 63 61 66 C3 A9
//! assert_eq!(r.push(&[0x63, 0x61, 0x66, 0xC3]), "caf"); // C3 is incomplete
//! assert_eq!(r.push(&[0xA9]), "é"); // completes the codepoint
//! assert_eq!(r.flush(), "");
//! ```

#![deny(missing_docs)]

/// UTF-8-safe streaming recombiner.
#[derive(Debug, Default, Clone)]
pub struct Recombiner {
    pending: Vec<u8>,
}

impl Recombiner {
    /// Build an empty recombiner.
    pub fn new() -> Self {
        Self::default()
    }

    /// Push the next chunk. Returns whatever resolved to whole UTF-8
    /// codepoints; any trailing fragment is buffered.
    pub fn push(&mut self, bytes: &[u8]) -> String {
        self.pending.extend_from_slice(bytes);
        // Find the longest prefix that is valid UTF-8.
        let split = longest_valid_utf8_prefix(&self.pending);
        let prefix: Vec<u8> = self.pending.drain(..split).collect();
        // SAFETY: validated above.
        String::from_utf8(prefix).unwrap_or_default()
    }

    /// Flush any buffered bytes. Invalid pending bytes are emitted as
    /// U+FFFD (Unicode replacement character).
    pub fn flush(&mut self) -> String {
        let out = String::from_utf8_lossy(&self.pending).into_owned();
        self.pending.clear();
        out
    }

    /// Number of bytes currently buffered (incomplete codepoint).
    pub fn pending(&self) -> usize {
        self.pending.len()
    }
}

fn longest_valid_utf8_prefix(bytes: &[u8]) -> usize {
    // Walk back at most 3 bytes from the end and check whether the
    // truncated buffer parses cleanly.
    for tail in 0..=3 {
        if tail > bytes.len() {
            break;
        }
        let end = bytes.len() - tail;
        if std::str::from_utf8(&bytes[..end]).is_ok() {
            return end;
        }
    }
    0
}
