use core::fmt;
use core::ops::Range;
use smallvec::SmallVec;

pub type ArgumentSplitterOwned = ArgumentSplitter<'static>;

/// Utility to split a string into segments delimited by whitespace and grouped by double quotation marks
///
/// Whitespace is removed except in quoted groups, and empty quoted groups are included.
pub struct ArgumentSplitter<'a> {
    buf: &'a str,
    arguments: SmallVec<[Argument<'a>; 4]>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Argument<'a> {
    buf: &'a str,
    inner_start: usize,
    inner_end: usize,
    outer_start: usize,
    outer_end: usize,
}

impl fmt::Debug for Argument<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.outer_str())
    }
}

impl<'a> Argument<'a> {
    fn unquoted(buf: &'a str, inner: Range<usize>) -> Self {
        Argument::new(buf, inner.clone(), inner)
    }

    fn new(buf: &'a str, inner: Range<usize>, outer: Range<usize>) -> Self {
        Argument {
            buf,
            inner_start: inner.start,
            inner_end: inner.end,
            outer_start: outer.start,
            outer_end: outer.end,
        }
    }

    #[must_use]
    pub fn orig(&self) -> &'a str {
        self.buf
    }

    #[must_use]
    pub fn inner_str(&self) -> &'a str {
        &self.buf[self.inner()]
    }

    #[must_use]
    pub fn outer_str(&self) -> &'a str {
        &self.buf[self.outer()]
    }

    #[must_use]
    pub fn inner(&self) -> Range<usize> {
        self.inner_start..self.inner_end
    }

    #[must_use]
    pub fn outer(&self) -> Range<usize> {
        self.outer_start..self.outer_end
    }

    #[must_use]
    pub fn is_quoted(&self) -> bool {
        self.inner() != self.outer()
    }

    #[must_use]
    pub fn is_quoted_with(&self, (start, end): (char, char)) -> bool {
        let outer = self.outer_str();

        self.is_quoted() && outer.starts_with(start) && outer.ends_with(end)
    }
}

impl<'a> ArgumentSplitter<'a> {
    #[must_use]
    pub fn orig(&self) -> &'a str {
        self.buf
    }

    #[must_use]
    pub fn arguments(&self) -> &[Argument<'a>] {
        &self.arguments
    }

    pub fn iter<'b>(&'b self) -> impl Iterator<Item = &'a str> + 'b
    where
        'a: 'b,
    {
        self.arguments.iter().map(|r| r.inner_str())
    }

    #[must_use]
    pub fn split(args: &'a str) -> Self {
        Self::split_delimiters(args, &[('`', '`'), ('"', '"'), ('\u{201C}', '\u{201D}')])
    }

    #[must_use]
    pub fn split_delimiters(args: &'a str, delmiters: &[(char, char)]) -> Self {
        let buf = args;
        let mut arguments = SmallVec::new();

        let mut escaped = false;
        let mut last_idx = 0;

        let mut ic = buf.char_indices();

        'outer: while let Some((idx, token)) = ic.next() {
            if escaped || token == '\\' {
                escaped ^= true;
                continue;
            }

            if token.is_whitespace() {
                if last_idx != idx {
                    arguments.push(Argument::unquoted(buf, last_idx..idx));
                }
                last_idx = idx + 1; // skip ws
                continue;
            }

            let Some(&(_, end_token)) = delmiters.iter().find(|(start, _)| token == *start) else {
                continue;
            };

            let token_len = token.len_utf8();

            // text"text" without whitespace
            if last_idx != idx {
                arguments.push(Argument::unquoted(buf, last_idx..idx));
            }

            for (end_idx, quoted_char) in &mut ic {
                if escaped || quoted_char == '\\' {
                    escaped ^= true;
                    continue;
                }

                if quoted_char == end_token {
                    let end_token_len = end_token.len_utf8();

                    // skip first quote
                    arguments.push(Argument::new(buf, idx + token_len..end_idx, idx..end_idx + end_token_len));

                    last_idx = end_idx + end_token_len;

                    continue 'outer; // skip fallback behavior if iterator runs out
                }
            }

            last_idx = idx + token_len;

            if last_idx == buf.len() {
                // empty
                arguments.push(Argument::new(buf, last_idx..last_idx, idx..last_idx));
            }
        }

        if last_idx != buf.len() {
            arguments.push(Argument::unquoted(buf, last_idx..buf.len()));
        }

        ArgumentSplitter { buf, arguments }
    }
}

#[cfg(test)]
mod test {
    use super::ArgumentSplitter;

    #[test]
    fn test_arg_splitter() {
        fn do_test(args: &str, expected: &[&str]) {
            let mut i = 0;
            for (a, &b) in ArgumentSplitter::split(args).iter().zip(expected) {
                assert_eq!(a, b, "Argument mismatch for {args} on argument {i}");
                i += 1;
            }
            assert_eq!(i, expected.len(), "Length mismatch!");
        }

        do_test("Hello,    World!    ", &["Hello,", "World!"]);
        do_test("    \"Testing\"", &["Testing"]);
        do_test("    Testing\"", &["Testing", ""]);
        do_test("    \"Test ing", &["Test ing"]);
        do_test("    \"Test ing\"   \"\"", &["Test ing", ""]);
        do_test("    \"Test ing\"   \"  \"", &["Test ing", "  "]);
        do_test("    `Test ing`   \"  \"", &["Test ing", "  "]);
        do_test(" This is \u{201C}a test", &["This", "is", "a test"]);
        do_test(" This is \u{201C}a test\u{201D}", &["This", "is", "a test"]);
        do_test("test\"test", &["test", "test"]);
    }
}
