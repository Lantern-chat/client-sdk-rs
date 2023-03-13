use smallvec::SmallVec;
use std::{borrow::Cow, ops::Range};

pub type ArgumentSplitterOwned = ArgumentSplitter<'static>;

/// Utility to split a string into segments delimited by whitespace and grouped by double quotation marks
///
/// Whitespace is removed except in quoted groups, and empty quoted groups are included.
pub struct ArgumentSplitter<'a> {
    buf: Cow<'a, str>,
    segments: SmallVec<[Range<usize>; 4]>,
}

impl<'a> ArgumentSplitter<'a> {
    pub fn args(&self) -> &str {
        &self.buf
    }

    pub fn segments(&self) -> &[Range<usize>] {
        &self.segments
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.segments.iter().map(|r| &self.buf[r.clone()])
    }

    pub fn split(args: impl Into<Cow<'a, str>>) -> Self {
        let buf: Cow<str> = args.into();
        let mut segments = SmallVec::new();

        let mut escaped = false;
        let mut last_idx = 0;

        let mut ic = buf.char_indices();

        'outer: while let Some((idx, token)) = ic.next() {
            if escaped || token == '\\' {
                escaped ^= true;
                continue;
            }

            match token {
                _ if token.is_whitespace() => {
                    if last_idx != idx {
                        segments.push(last_idx..idx);
                    }
                    last_idx = idx + 1; // skip ws
                }
                '"' | '\u{201C}' => {
                    let end_token = if token == '"' { token } else { '\u{201D}' };
                    let token_len = token.len_utf8();

                    // text"text" without whitespace
                    if last_idx != idx {
                        segments.push(last_idx..idx);
                    }

                    for (end_idx, quoted_char) in &mut ic {
                        if escaped || quoted_char == '\\' {
                            escaped ^= true;
                            continue;
                        }

                        if quoted_char == end_token {
                            // skip first quote
                            segments.push(idx + token_len..end_idx);

                            last_idx = end_idx + end_token.len_utf8();

                            continue 'outer; // skip fallback behavior if iterator runs out
                        }
                    }

                    last_idx = idx + token_len;

                    if last_idx == buf.len() {
                        segments.push(last_idx..last_idx); // empty
                    }
                }
                _ => {}
            }
        }

        if last_idx != buf.len() {
            segments.push(last_idx..buf.len());
        }

        ArgumentSplitter { buf, segments }
    }
}

#[cfg(test)]
mod test {
    use super::ArgumentSplitter;

    #[test]
    fn test_arg_splitter() {
        fn do_test<I>(args: &str, expected: impl IntoIterator<IntoIter = I>)
        where
            I: ExactSizeIterator<Item = &'static &'static str>,
        {
            let mut i = 0;
            let expected = expected.into_iter();
            let expected_len = expected.len();
            for (a, &b) in ArgumentSplitter::split(args).iter().zip(expected) {
                assert_eq!(a, b, "Argument mismatch for {args} on argument {i}");
                i += 1;
            }
            assert_eq!(i, expected_len, "Length mismatch!");
        }

        do_test("Hello,    World!    ", &["Hello,", "World!"]);
        do_test("    \"Testing\"", &["Testing"]);
        do_test("    Testing\"", &["Testing", ""]);
        do_test("    \"Testing", &["Testing"]);
        do_test("    \"Testing\"   \"\"", &["Testing", ""]);
        do_test("    \"Testing\"   \"  \"", &["Testing", "  "]);
        do_test(" This is \u{201C}a test", &["This", "is", "a test"]);
        do_test(" This is \u{201C}a test\u{201D}", &["This", "is", "a test"]);
    }
}
