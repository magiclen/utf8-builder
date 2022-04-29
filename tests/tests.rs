use utf8_builder::Utf8Builder;

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
fn push_chunk() {
    for &text in TEXTS {
        let mut builder = Utf8Builder::new();

        for c in text.as_bytes().chunks(3) {
            builder.push_chunk(c).unwrap();
        }

        let result = builder.finalize().unwrap();

        assert_eq!(text, result.as_str());
    }
}
