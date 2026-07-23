use utf8_builder::{Utf8Builder, Utf8Error};

const TEXT1: &str = "This is English. And 這是中文。This is number, 123.";
const TEXT2: &str = "あれは えんびつですか";
const TEXT3: &str = "😀 😃 😄 😁 😆 😅 😂 🤣 😇 😉 😊 🙂 🙃";
const TEXTS: &[&str] = &[TEXT1, TEXT2, TEXT3];

#[test]
fn push() {
    for &text in TEXTS {
        let mut builder = Utf8Builder::new();

        for b in text.as_bytes().iter().copied() {
            builder.push(b).unwrap();
        }

        let result = builder.finalize().unwrap();

        assert_eq!(text, result.as_str());
    }
}

#[test]
fn push_str() {
    for &text in TEXTS {
        let mut builder = Utf8Builder::new();

        builder.push_str(text).unwrap();

        let result = builder.finalize().unwrap();

        assert_eq!(text, result.as_str());
    }
}

#[test]
fn push_char() {
    for &text in TEXTS {
        let mut builder = Utf8Builder::new();

        for c in text.chars() {
            builder.push_char(c).unwrap();
        }

        let result = builder.finalize().unwrap();

        assert_eq!(text, result.as_str());
    }
}

#[test]
fn invalid_data() {
    const INVALID_DATA: &[&[u8]] = &[
        &[0xE4, 0x61, 0x61],       // invalid continuation byte
        &[0xE0, 0x80, 0x80],       // overlong encoding
        &[0xED, 0xA0, 0x80],       // UTF-16 surrogate
        &[0xF4, 0x90, 0x80, 0x80], // out of the Unicode range
    ];

    for &data in INVALID_DATA {
        let mut builder = Utf8Builder::new();

        assert!(data.iter().copied().any(|b| builder.push(b).is_err()));

        let mut builder = Utf8Builder::new();

        assert_eq!(Err(Utf8Error), builder.push_chunk(data));

        // split after the lead byte to check the validation for the pending incomplete character
        let mut builder = Utf8Builder::new();

        builder.push_chunk(&data[..1]).unwrap();

        assert_eq!(Err(Utf8Error), builder.push_chunk(&data[1..]));
    }
}

#[test]
fn incomplete_data() {
    let mut builder = Utf8Builder::new();

    builder.push_chunk("中".as_bytes().split_last().unwrap().1).unwrap();

    assert!(!builder.is_valid());
    assert!(builder.finalize().is_err());
}

#[test]
fn push_chunk() {
    for chunk_size in 1..=8 {
        for &text in TEXTS {
            let mut builder = Utf8Builder::new();

            for c in text.as_bytes().chunks(chunk_size) {
                builder.push_chunk(c).unwrap();
            }

            let result = builder.finalize().unwrap();

            assert_eq!(text, result.as_str());
        }
    }
}

#[test]
fn recover_incomplete_data() {
    let incomplete = &"中".as_bytes()[..2];

    let mut builder = Utf8Builder::new();

    builder.push_chunk(incomplete).unwrap();

    assert_eq!(incomplete, builder.as_bytes());
    assert_eq!(incomplete, builder.into_bytes().as_slice());
}

#[test]
fn fmt_write() {
    use std::fmt::Write;

    let mut builder = Utf8Builder::new();

    write!(builder, "Hello {}!", 123).unwrap();

    let result = builder.finalize().unwrap();

    assert_eq!("Hello 123!", result.as_str());
}

fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

#[test]
fn differential() {
    // compare the chunked validation result with the one-shot validation result of the standard library
    let mut state = 0x2545_F491_4F6C_DD1Du64;

    for _ in 0..2000 {
        let mut data: Vec<u8> = Vec::new();

        for _ in 0..xorshift(&mut state) % 24 {
            match xorshift(&mut state) % 3 {
                0 => data.push((xorshift(&mut state) & 0x7F) as u8),
                1 => {
                    if let Some(c) = char::from_u32((xorshift(&mut state) % 0x11_0000) as u32) {
                        let mut tmp = [0u8; 4];

                        data.extend_from_slice(c.encode_utf8(&mut tmp).as_bytes());
                    }
                },
                // an arbitrary byte which may produce invalid or incomplete sequences
                _ => data.push(xorshift(&mut state) as u8),
            }
        }

        let mut builder = Utf8Builder::new();
        let mut failed = false;
        let mut i = 0;

        while i < data.len() {
            let end = (i + 1 + (xorshift(&mut state) % 5) as usize).min(data.len());

            if builder.push_chunk(&data[i..end]).is_err() {
                failed = true;
                break;
            }

            i = end;
        }

        let result = if failed { None } else { builder.finalize().ok() };

        match std::str::from_utf8(&data) {
            Ok(s) => assert_eq!(Some(s), result.as_deref()),
            Err(_) => assert_eq!(None, result),
        }
    }
}
