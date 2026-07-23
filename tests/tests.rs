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
