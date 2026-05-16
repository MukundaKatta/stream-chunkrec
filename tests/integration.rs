use stream_chunkrec::Recombiner;

#[test]
fn ascii_passes_through() {
    let mut r = Recombiner::new();
    assert_eq!(r.push(b"hello"), "hello");
    assert_eq!(r.pending(), 0);
}

#[test]
fn partial_2byte_codepoint_buffered() {
    let mut r = Recombiner::new();
    // "café": c=63 a=61 f=66 é=C3 A9
    assert_eq!(r.push(&[0x63, 0x61, 0x66, 0xC3]), "caf");
    assert_eq!(r.pending(), 1);
    assert_eq!(r.push(&[0xA9]), "é");
    assert_eq!(r.pending(), 0);
}

#[test]
fn partial_3byte_codepoint_buffered() {
    // U+1F600 grinning face? No, that's 4-byte. Use U+2603 SNOWMAN = E2 98 83.
    let mut r = Recombiner::new();
    assert_eq!(r.push(&[0xE2, 0x98]), "");
    assert_eq!(r.pending(), 2);
    assert_eq!(r.push(&[0x83]), "\u{2603}");
}

#[test]
fn partial_4byte_codepoint_buffered() {
    // U+1F600 GRINNING FACE = F0 9F 98 80
    let mut r = Recombiner::new();
    assert_eq!(r.push(&[0xF0, 0x9F]), "");
    assert_eq!(r.push(&[0x98]), "");
    assert_eq!(r.push(&[0x80]), "\u{1F600}");
}

#[test]
fn flush_replaces_invalid_pending_with_fffd() {
    let mut r = Recombiner::new();
    r.push(&[0xC3]); // start of 2-byte, never completed
    let s = r.flush();
    assert!(s.contains('\u{FFFD}'));
    assert_eq!(r.pending(), 0);
}

#[test]
fn many_small_chunks_reconstruct_correctly() {
    let mut r = Recombiner::new();
    let text = "hello, world: café résumé 日本語";
    let mut out = String::new();
    for byte in text.as_bytes() {
        out.push_str(&r.push(&[*byte]));
    }
    out.push_str(&r.flush());
    assert_eq!(out, text);
}
