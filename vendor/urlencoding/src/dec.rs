use std::borrow::Cow;
use std::string::FromUtf8Error;

#[inline]
pub(crate) fn from_hex_digit(digit: u8) -> Option<u8> {
    match digit {
        b'0'..=b'9' => Some(digit - b'0'),
        b'A'..=b'F' => Some(digit - b'A' + 10),
        b'a'..=b'f' => Some(digit - b'a' + 10),
        _ => None,
    }
}

/// Decode percent-encoded string assuming UTF-8 encoding.
///
/// If you need a `String`, call `.into_owned()` (not `.to_owned()`).
///
/// Unencoded `+` is preserved literally, and _not_ changed to a space.
pub fn decode(data: &str) -> Result<Cow<str>, FromUtf8Error> {
    match decode_binary(data.as_bytes()) {
        Cow::Borrowed(_) => Ok(Cow::Borrowed(data)),
        Cow::Owned(s) => Ok(Cow::Owned(String::from_utf8(s)?)),
    }
}

/// Decode percent-encoded string as binary data, in any encoding.
///
/// Unencoded `+` is preserved literally, and _not_ changed to a space.
pub fn decode_binary(mut data: &[u8]) -> Cow<[u8]> {
    let mut out: Vec<u8> = Vec::with_capacity(data.len());
    loop {
        let mut parts = data.splitn(2, |&c| c == b'%');
        // first the decoded non-% part
        out.extend_from_slice(parts.next().unwrap());
        // then decode one %xx
        match parts.next() {
            None => {
                if out.is_empty() {
                    // avoids utf-8 check
                    return data.into();
                }
                break;
            },
            Some(rest) => match rest.get(0..2) {
                Some(&[first, second]) => match from_hex_digit(first) {
                    Some(first_val) => match from_hex_digit(second) {
                        Some(second_val) => {
                            out.push((first_val << 4) | second_val);
                            data = &rest[2..];
                        },
                        None => {
                            out.extend_from_slice(&[b'%', first]);
                            data = &rest[1..];
                        },
                    },
                    None => {
                        out.push(b'%');
                        data = rest;
                    },
                },
                _ => {
                    // too short
                    out.push(b'%');
                    out.extend_from_slice(rest);
                    break;
                },
            },
        };
    }
    Cow::Owned(out)
}
