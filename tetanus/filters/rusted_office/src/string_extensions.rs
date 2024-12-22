use num_traits::Zero;

pub struct StringCursor<'a> {
    underlying: &'a str,
    index: usize,
}

pub trait Matcher {
    fn is_start(&self, target: &str) -> bool;
}

impl Matcher for str {
    fn is_start(&self, target: &str) -> bool {
        target.starts_with(self)
    }
}

impl Matcher for char {
    fn is_start(&self, target: &str) -> bool {
        target.starts_with(*self)
    }
}

pub trait StringCursorPredicate {
    fn compare(&self, remaining: &str) -> bool;
    fn len(&self) -> usize;
}

impl StringCursorPredicate for str {
    fn compare(&self, remaining: &str) -> bool {
        remaining.starts_with(self)
    }

    fn len(&self) -> usize {
        self.len()
    }
}

impl StringCursorPredicate for char {
    fn compare(&self, remaining: &str) -> bool {
        remaining.starts_with(*self)
    }

    fn len(&self) -> usize {
        1
    }
}

impl<'a> StringCursor<'a> {
    pub fn new(str: &'a str) -> Self {
        Self {
            underlying: str,
            index: 0,
        }
    }

    pub fn skip(&mut self, count: usize) {
        self.index += count
    }

    pub fn next(&mut self) -> Option<&str> {
        self.index += 1;
        self.underlying.get(self.index - 1..self.index)
    }

    pub fn peak(&self) -> Option<&str> {
        self.underlying.get(self.index..self.index + 1)
    }

    pub fn next_if(&mut self, condition: impl StringCursorPredicate) -> bool {
        if condition.compare(self.remaining()) {
            self.index += condition.len();
            true
        } else {
            false
        }
    }

    pub fn new_from_underlying(&self, start: Option<usize>, skip_end: bool) -> Self {
        let start = start.unwrap_or(0);
        Self {
            underlying: &self.underlying[start..self.index - skip_end as usize],
            index: 0,
        }
    }

    pub fn remaining(&self) -> &str {
        &self.underlying[self.index..]
    }

    pub fn position(&self) -> usize {
        self.index
    }
}

pub struct ScopeDeclaration {
    pub begin: char,
    pub end: char,
}

impl ScopeDeclaration {
    pub fn begin(&self, char: char) -> bool {
        self.begin == char
    }

    pub fn end(&self, char: char) -> bool {
        self.end == char
    }
}

#[derive(Debug, Copy, Clone)]
pub struct CharIndex(pub usize);
#[derive(Debug, Copy, Clone)]
pub struct ByteIndex(pub usize);
#[derive(Debug, Copy, Clone)]
pub struct EndIndex;

pub trait IndexConvert: Clone + Copy {
    fn to_str_index(self, str: &str) -> usize;
}

impl IndexConvert for CharIndex {
    fn to_str_index(self, _: &str) -> usize {
        self.0
    }
}
impl IndexConvert for EndIndex {
    fn to_str_index(self, str: &str) -> usize {
        str.len()
    }
}

impl IndexConvert for ByteIndex {
    fn to_str_index(self, s: &str) -> usize {
        let mut char_index = 0;
        let mut byte_pos = 0;
        let byte_index = self.0;

        for (i, c) in s.char_indices() {
            if byte_pos == byte_index {
                return char_index;
            }
            byte_pos = i + c.len_utf8(); // advance to the next character's byte position
            char_index += 1;
        }

        panic!("Invalid byte index")
    }
}

pub trait StringExtension {
    fn index_out_of_scope(&self, scope_declarations: &[ScopeDeclaration]) -> Option<usize>;
    fn scoped_slice(&self, scope_declarations: &[ScopeDeclaration]) -> Option<&str>;
    fn scoped_split<P: Matcher>(
        &self,
        haystack: &P,
        scope_declarations: &[ScopeDeclaration],
        include_haystack: bool,
        haystack_size: usize,
    ) -> (&str, Option<&str>);
    fn slice_extended(&self, begin: impl IndexConvert, end: impl IndexConvert) -> &str;
}

impl StringExtension for str {
    fn index_out_of_scope(&self, scope_declarations: &[ScopeDeclaration]) -> Option<usize> {
        let mut cursor = StringCursor::new(self);
        let mut start_indent = 0;

        let continue_call = scope_declarations.iter().any(|r| cursor.next_if(r.begin));
        if !continue_call {
            return None;
        }
        start_indent += 1;

        while !start_indent.is_zero() {
            if scope_declarations.iter().any(|r| cursor.next_if(r.begin)) {
                start_indent += 1;
            } else if scope_declarations.iter().any(|r| cursor.next_if(r.end)) {
                start_indent -= 1;
            } else {
                let _ = cursor.next()?;
            }
        }

        Some(cursor.position() + 1)
    }

    fn scoped_slice(&self, scope_declarations: &[ScopeDeclaration]) -> Option<&str> {
        let idx = self.index_out_of_scope(scope_declarations)?;
        Some(&self[1..idx - 2])
    }

    fn scoped_split<P: Matcher>(
        &self,
        haystack: &P,
        scope_declarations: &[ScopeDeclaration],
        include_haystack: bool,
        haystack_size: usize,
    ) -> (&str, Option<&str>) {
        let mut idx: usize = 0;
        while idx < self.len() {
            if let Some(e) = self[idx..].index_out_of_scope(scope_declarations) {
                idx += e - 1;
            } else if haystack.is_start(&self[idx..]) {
                return (
                    &self[..idx],
                    // Seems a little confusing but since a bool is 0 for false and 1 for true this basically just sets it to 0 if false is passed
                    Some(&self[idx + haystack_size * !include_haystack as usize..]),
                );
            } else {
                idx += 1;
            }
        }

        (self, None)
    }

    fn slice_extended(&self, begin: impl IndexConvert, end: impl IndexConvert) -> &str {
        &self[begin.to_str_index(self)..end.to_str_index(self)]
    }
}

#[cfg(test)]
mod tests {
    use crate::string_extensions::{ScopeDeclaration, StringExtension};

    #[test]
    fn scoped_slice() {
        let str = "(this is a { nested [ slice ]})";
        let result = str
            .scoped_slice(&[
                ScopeDeclaration {
                    begin: '{',
                    end: '}',
                },
                ScopeDeclaration {
                    begin: '[',
                    end: ']',
                },
                ScopeDeclaration {
                    begin: '(',
                    end: ')',
                },
            ])
            .unwrap();
        assert_eq!(result, "this is a { nested [ slice ]}");
    }

    #[test]
    fn scoped_split() {
        let information = [
            (
                "this string, should split",
                ("this string", Some(" should split")),
                false,
            ),
            (
                "this string shouldn't split",
                ("this string shouldn't split", None),
                false,
            ),
            (
                "this string {should split}, Hello",
                ("this string {should split}", Some(", Hello")),
                true,
            ),
            (
                "this string {shouldn't, split}",
                ("this string {shouldn't, split}", None),
                false,
            ),
        ];

        for (str, expected, include_haystack) in information {
            let out = str.scoped_split(
                &',',
                &[ScopeDeclaration {
                    begin: '{',
                    end: '}',
                }],
                include_haystack,
                1,
            );

            assert_eq!(expected, out);
        }
    }
}
