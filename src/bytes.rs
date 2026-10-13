//! Byte-string helpers standing in for the C library calls upstream leans on.
//!
//! The searches go to the `memchr` crate, which looks at many bytes at a
//! time, as the C library's `memchr` does for upstream. That matters little
//! on the few bytes of most tokens, and a lot when a search runs to the end
//! of a long input, as several do on input that is not an attack.

/// `memchr` over `s[from..]`, returning an index into `s`.
pub(crate) fn find_byte(s: &[u8], from: usize, byte: u8) -> Option<usize> {
    memchr::memchr(byte, &s[from..]).map(|i| from + i)
}

/// Whether `byte` is anywhere in `s`.
pub(crate) fn contains(s: &[u8], byte: u8) -> bool {
    memchr::memchr(byte, s).is_some()
}

/// Upstream's `memchr2`: the first occurrence of the pair `c0 c1`.
pub(crate) fn find_pair(haystack: &[u8], c0: u8, c1: u8) -> Option<usize> {
    let mut from = 0;
    while let Some(at) = find_byte(haystack, from, c0) {
        if haystack.get(at + 1) == Some(&c1) {
            return Some(at);
        }
        from = at + 1;
    }
    None
}

/// Upstream's `my_memmem`.
pub(crate) fn find_slice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    memchr::memmem::find(haystack, needle)
}

/// A membership table for `strchr(accept, ch) != NULL`.
///
/// `strchr` also matches the NUL that terminates `accept`, so NUL is a member
/// of every set, whether or not upstream spelled it out.
pub(crate) const fn byte_set(accept: &[u8]) -> [bool; 256] {
    let mut set = [false; 256];
    set[0] = true;
    let mut i = 0;
    while i < accept.len() {
        set[accept[i] as usize] = true;
        i += 1;
    }
    set
}

/// Upstream's `strlenspn`: length of the leading run of bytes in `accept`.
pub(crate) fn span(s: &[u8], accept: &[bool; 256]) -> usize {
    s.iter()
        .position(|&b| !accept[usize::from(b)])
        .unwrap_or(s.len())
}

/// Upstream's `strlencspn`: length of the leading run of bytes not in `reject`.
pub(crate) fn cspan(s: &[u8], reject: &[bool; 256]) -> usize {
    s.iter()
        .position(|&b| reject[usize::from(b)])
        .unwrap_or(s.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_bytes_from_an_offset() {
        assert_eq!(find_byte(b"a'b'c", 0, b'\''), Some(1));
        assert_eq!(find_byte(b"a'b'c", 2, b'\''), Some(3));
        assert_eq!(find_byte(b"a'b'c", 4, b'\''), None);
        assert_eq!(find_byte(b"a'b'c", 5, b'\''), None);

        assert!(contains(b"a'b", b'\''));
        assert!(!contains(b"a'b", b'"'));
        assert!(!contains(b"", b'"'));
    }

    #[test]
    fn finds_pairs() {
        assert_eq!(find_pair(b"a */ b */", b'*', b'/'), Some(2));
        // The first byte of the pair alone is not it, twice over or last.
        assert_eq!(find_pair(b"a * b **/", b'*', b'/'), Some(7));
        assert_eq!(find_pair(b"a / b *", b'*', b'/'), None);
        assert_eq!(find_pair(b"$$", b'$', b'$'), Some(0));
        assert_eq!(find_pair(b"$", b'$', b'$'), None);
        assert_eq!(find_pair(b"", b'$', b'$'), None);
    }

    #[test]
    fn finds_slices() {
        assert_eq!(find_slice(b"$t$ a $t$", b"$t$"), Some(0));
        assert_eq!(find_slice(b"a $t $t$", b"$t$"), Some(5));
        assert_eq!(find_slice(b"a $t", b"$t$"), None);
        assert_eq!(find_slice(b"", b"$t$"), None);
    }
}
