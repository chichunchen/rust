// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Character manipulation.
//!
//! For more details, see ::core::unicode::char (a.k.a. std::char)

#![allow(non_snake_case)]
#![stable(feature = "core_char", since = "1.2.0")]

mod convert;
mod decode;
mod printable;

// stable re-exports
#[stable(feature = "rust1", since = "1.0.0")]
pub use self::convert::{from_u32, from_digit};
#[stable(feature = "char_from_unchecked", since = "1.5.0")]
pub use self::convert::from_u32_unchecked;
#[stable(feature = "char_from_str", since = "1.20.0")]
pub use self::convert::ParseCharError;
#[stable(feature = "try_from", since = "1.26.0")]
pub use self::convert::CharTryFromError;
#[stable(feature = "rust1", since = "1.0.0")]
pub use unicode::char::{ToLowercase, ToUppercase};
#[stable(feature = "decode_utf16", since = "1.9.0")]
pub use self::decode::{decode_utf16, DecodeUtf16, DecodeUtf16Error};

// unstable re-exports
#[unstable(feature = "unicode", issue = "27783")]
pub use unicode::tables::{UNICODE_VERSION};
#[unstable(feature = "unicode", issue = "27783")]
pub use unicode::version::UnicodeVersion;
#[unstable(feature = "decode_utf8", issue = "33906")]
pub use self::decode::{decode_utf8, DecodeUtf8, InvalidSequence};

use self::printable::is_printable;
use fmt::{self, Write};
use slice;
use str::from_utf8_unchecked_mut;
use iter::FusedIterator;

// UTF-8 ranges and tags for encoding characters
const TAG_CONT: u8    = 0b1000_0000;
const TAG_TWO_B: u8   = 0b1100_0000;
const TAG_THREE_B: u8 = 0b1110_0000;
const TAG_FOUR_B: u8  = 0b1111_0000;
const MAX_ONE_B: u32   =     0x80;
const MAX_TWO_B: u32   =    0x800;
const MAX_THREE_B: u32 =  0x10000;

/*
    Lu  Uppercase_Letter        an uppercase letter
    Ll  Lowercase_Letter        a lowercase letter
    Lt  Titlecase_Letter        a digraphic character, with first part uppercase
    Lm  Modifier_Letter         a modifier letter
    Lo  Other_Letter            other letters, including syllables and ideographs
    Mn  Nonspacing_Mark         a nonspacing combining mark (zero advance width)
    Mc  Spacing_Mark            a spacing combining mark (positive advance width)
    Me  Enclosing_Mark          an enclosing combining mark
    Nd  Decimal_Number          a decimal digit
    Nl  Letter_Number           a letterlike numeric character
    No  Other_Number            a numeric character of other type
    Pc  Connector_Punctuation   a connecting punctuation mark, like a tie
    Pd  Dash_Punctuation        a dash or hyphen punctuation mark
    Ps  Open_Punctuation        an opening punctuation mark (of a pair)
    Pe  Close_Punctuation       a closing punctuation mark (of a pair)
    Pi  Initial_Punctuation     an initial quotation mark
    Pf  Final_Punctuation       a final quotation mark
    Po  Other_Punctuation       a punctuation mark of other type
    Sm  Math_Symbol             a symbol of primarily mathematical use
    Sc  Currency_Symbol         a currency sign
    Sk  Modifier_Symbol         a non-letterlike modifier symbol
    So  Other_Symbol            a symbol of other type
    Zs  Space_Separator         a space character (of various non-zero widths)
    Zl  Line_Separator          U+2028 LINE SEPARATOR only
    Zp  Paragraph_Separator     U+2029 PARAGRAPH SEPARATOR only
    Cc  Control                 a C0 or C1 control code
    Cf  Format                  a format control character
    Cs  Surrogate               a surrogate code point
    Co  Private_Use             a private-use character
    Cn  Unassigned              a reserved unassigned code point or a noncharacter
*/

/// The highest valid code point a `char` can have.
///
/// A [`char`] is a [Unicode Scalar Value], which means that it is a [Code
/// Point], but only ones within a certain range. `MAX` is the highest valid
/// code point that's a valid [Unicode Scalar Value].
///
/// [`char`]: ../../std/primitive.char.html
/// [Unicode Scalar Value]: http://www.unicode.org/glossary/#unicode_scalar_value
/// [Code Point]: http://www.unicode.org/glossary/#code_point
#[stable(feature = "rust1", since = "1.0.0")]
pub const MAX: char = '\u{10ffff}';

/// `U+FFFD REPLACEMENT CHARACTER` (�) is used in Unicode to represent a
/// decoding error.
///
/// It can occur, for example, when giving ill-formed UTF-8 bytes to
/// [`String::from_utf8_lossy`](../../std/string/struct.String.html#method.from_utf8_lossy).
#[stable(feature = "decode_utf16", since = "1.9.0")]
pub const REPLACEMENT_CHARACTER: char = '\u{FFFD}';

// NB: the stabilization and documentation for this trait is in
// unicode/char.rs, not here
#[allow(missing_docs)] // docs in libunicode/u_char.rs
#[doc(hidden)]
#[unstable(feature = "core_char_ext",
           reason = "the stable interface is `impl char` in later crate",
           issue = "32110")]
pub trait CharExt {
    #[stable(feature = "core", since = "1.6.0")]
    fn is_digit(self, radix: u32) -> bool;
    #[stable(feature = "core", since = "1.6.0")]
    fn to_digit(self, radix: u32) -> Option<u32>;
    #[stable(feature = "core", since = "1.6.0")]
    fn escape_unicode(self) -> EscapeUnicode;
    #[stable(feature = "core", since = "1.6.0")]
    fn escape_default(self) -> EscapeDefault;
    #[stable(feature = "char_escape_debug", since = "1.20.0")]
    fn escape_debug(self) -> EscapeDebug;
    #[stable(feature = "core", since = "1.6.0")]
    fn len_utf8(self) -> usize;
    #[stable(feature = "core", since = "1.6.0")]
    fn len_utf16(self) -> usize;
    #[stable(feature = "unicode_encode_char", since = "1.15.0")]
    fn encode_utf8(self, dst: &mut [u8]) -> &mut str;
    #[stable(feature = "unicode_encode_char", since = "1.15.0")]
    fn encode_utf16(self, dst: &mut [u16]) -> &mut [u16];
}

#[stable(feature = "core", since = "1.6.0")]
impl CharExt for char {
    #[inline]
    fn is_digit(self, radix: u32) -> bool {
        self.to_digit(radix).is_some()
    }

    #[inline]
    fn to_digit(self, radix: u32) -> Option<u32> {
        if radix > 36 {
            panic!("to_digit: radix is too high (maximum 36)");
        }
        let val = match self {
          '0' ... '9' => self as u32 - '0' as u32,
          'a' ... 'z' => self as u32 - 'a' as u32 + 10,
          'A' ... 'Z' => self as u32 - 'A' as u32 + 10,
          _ => return None,
        };
        if val < radix { Some(val) }
        else { None }
    }

    #[inline]
    fn escape_unicode(self) -> EscapeUnicode {
        let c = self as u32;

        // or-ing 1 ensures that for c==0 the code computes that one
        // digit should be printed and (which is the same) avoids the
        // (31 - 32) underflow
        let msb = 31 - (c | 1).leading_zeros();

        // the index of the most significant hex digit
        let ms_hex_digit = msb / 4;
        EscapeUnicode {
            c: self,
            state: EscapeUnicodeState::Backslash,
            hex_digit_idx: ms_hex_digit as usize,
        }
    }

    #[inline]
    fn escape_default(self) -> EscapeDefault {
        let init_state = match self {
            '\t' => EscapeDefaultState::Backslash('t'),
            '\r' => EscapeDefaultState::Backslash('r'),
            '\n' => EscapeDefaultState::Backslash('n'),
            '\\' | '\'' | '"' => EscapeDefaultState::Backslash(self),
            '\x20' ... '\x7e' => EscapeDefaultState::Char(self),
            _ => EscapeDefaultState::Unicode(self.escape_unicode())
        };
        EscapeDefault { state: init_state }
    }

    #[inline]
    fn escape_debug(self) -> EscapeDebug {
        let init_state = match self {
            '\t' => EscapeDefaultState::Backslash('t'),
            '\r' => EscapeDefaultState::Backslash('r'),
            '\n' => EscapeDefaultState::Backslash('n'),
            '\\' | '\'' | '"' => EscapeDefaultState::Backslash(self),
            c if is_printable(c) => EscapeDefaultState::Char(c),
            c => EscapeDefaultState::Unicode(c.escape_unicode()),
        };
        EscapeDebug(EscapeDefault { state: init_state })
    }

    #[inline]
    fn len_utf8(self) -> usize {
        let code = self as u32;
        if code < MAX_ONE_B {
            1
        } else if code < MAX_TWO_B {
            2
        } else if code < MAX_THREE_B {
            3
        } else {
            4
        }
    }

    #[inline]
    fn len_utf16(self) -> usize {
        let ch = self as u32;
        if (ch & 0xFFFF) == ch { 1 } else { 2 }
    }

    #[inline]
    fn encode_utf8(self, dst: &mut [u8]) -> &mut str {
        let code = self as u32;
        unsafe {
            let len =
            if code < MAX_ONE_B && !dst.is_empty() {
                *dst.get_unchecked_mut(0) = code as u8;
                1
            } else if code < MAX_TWO_B && dst.len() >= 2 {
                *dst.get_unchecked_mut(0) = (code >> 6 & 0x1F) as u8 | TAG_TWO_B;
                *dst.get_unchecked_mut(1) = (code & 0x3F) as u8 | TAG_CONT;
                2
            } else if code < MAX_THREE_B && dst.len() >= 3  {
                *dst.get_unchecked_mut(0) = (code >> 12 & 0x0F) as u8 | TAG_THREE_B;
                *dst.get_unchecked_mut(1) = (code >>  6 & 0x3F) as u8 | TAG_CONT;
                *dst.get_unchecked_mut(2) = (code & 0x3F) as u8 | TAG_CONT;
                3
            } else if dst.len() >= 4 {
                *dst.get_unchecked_mut(0) = (code >> 18 & 0x07) as u8 | TAG_FOUR_B;
                *dst.get_unchecked_mut(1) = (code >> 12 & 0x3F) as u8 | TAG_CONT;
                *dst.get_unchecked_mut(2) = (code >>  6 & 0x3F) as u8 | TAG_CONT;
                *dst.get_unchecked_mut(3) = (code & 0x3F) as u8 | TAG_CONT;
                4
            } else {
                panic!("encode_utf8: need {} bytes to encode U+{:X}, but the buffer has {}",
                    from_u32_unchecked(code).len_utf8(),
                    code,
                    dst.len())
            };
            from_utf8_unchecked_mut(dst.get_unchecked_mut(..len))
        }
    }

    #[inline]
    fn encode_utf16(self, dst: &mut [u16]) -> &mut [u16] {
        let mut code = self as u32;
        unsafe {
            if (code & 0xFFFF) == code && !dst.is_empty() {
                // The BMP falls through (assuming non-surrogate, as it should)
                *dst.get_unchecked_mut(0) = code as u16;
                slice::from_raw_parts_mut(dst.as_mut_ptr(), 1)
            } else if dst.len() >= 2 {
                // Supplementary planes break into surrogates.
                code -= 0x1_0000;
                *dst.get_unchecked_mut(0) = 0xD800 | ((code >> 10) as u16);
                *dst.get_unchecked_mut(1) = 0xDC00 | ((code as u16) & 0x3FF);
                slice::from_raw_parts_mut(dst.as_mut_ptr(), 2)
            } else {
                panic!("encode_utf16: need {} units to encode U+{:X}, but the buffer has {}",
                    from_u32_unchecked(code).len_utf16(),
                    code,
                    dst.len())
            }
        }
    }
}

/// Returns an iterator that yields the hexadecimal Unicode escape of a
/// character, as `char`s.
///
/// This `struct` is created by the [`escape_unicode`] method on [`char`]. See
/// its documentation for more.
///
/// [`escape_unicode`]: ../../std/primitive.char.html#method.escape_unicode
/// [`char`]: ../../std/primitive.char.html
#[derive(Clone, Debug)]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct EscapeUnicode {
    c: char,
    state: EscapeUnicodeState,

    // The index of the next hex digit to be printed (0 if none),
    // i.e. the number of remaining hex digits to be printed;
    // increasing from the least significant digit: 0x543210
    hex_digit_idx: usize,
}

// The enum values are ordered so that their representation is the
// same as the remaining length (besides the hexadecimal digits). This
// likely makes `len()` a single load from memory) and inline-worth.
#[derive(Clone, Debug)]
enum EscapeUnicodeState {
    Done,
    RightBrace,
    Value,
    LeftBrace,
    Type,
    Backslash,
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Iterator for EscapeUnicode {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        match self.state {
            EscapeUnicodeState::Backslash => {
                self.state = EscapeUnicodeState::Type;
                Some('\\')
            }
            EscapeUnicodeState::Type => {
                self.state = EscapeUnicodeState::LeftBrace;
                Some('u')
            }
            EscapeUnicodeState::LeftBrace => {
                self.state = EscapeUnicodeState::Value;
                Some('{')
            }
            EscapeUnicodeState::Value => {
                let hex_digit = ((self.c as u32) >> (self.hex_digit_idx * 4)) & 0xf;
                let c = from_digit(hex_digit, 16).unwrap();
                if self.hex_digit_idx == 0 {
                    self.state = EscapeUnicodeState::RightBrace;
                } else {
                    self.hex_digit_idx -= 1;
                }
                Some(c)
            }
            EscapeUnicodeState::RightBrace => {
                self.state = EscapeUnicodeState::Done;
                Some('}')
            }
            EscapeUnicodeState::Done => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.len();
        (n, Some(n))
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    fn last(self) -> Option<char> {
        match self.state {
            EscapeUnicodeState::Done => None,

            EscapeUnicodeState::RightBrace |
            EscapeUnicodeState::Value |
            EscapeUnicodeState::LeftBrace |
            EscapeUnicodeState::Type |
            EscapeUnicodeState::Backslash => Some('}'),
        }
    }
}

#[stable(feature = "exact_size_escape", since = "1.11.0")]
impl ExactSizeIterator for EscapeUnicode {
    #[inline]
    fn len(&self) -> usize {
        // The match is a single memory access with no branching
        self.hex_digit_idx + match self.state {
            EscapeUnicodeState::Done => 0,
            EscapeUnicodeState::RightBrace => 1,
            EscapeUnicodeState::Value => 2,
            EscapeUnicodeState::LeftBrace => 3,
            EscapeUnicodeState::Type => 4,
            EscapeUnicodeState::Backslash => 5,
        }
    }
}

#[stable(feature = "fused", since = "1.26.0")]
impl FusedIterator for EscapeUnicode {}

#[stable(feature = "char_struct_display", since = "1.16.0")]
impl fmt::Display for EscapeUnicode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for c in self.clone() {
            f.write_char(c)?;
        }
        Ok(())
    }
}

/// An iterator that yields the literal escape code of a `char`.
///
/// This `struct` is created by the [`escape_default`] method on [`char`]. See
/// its documentation for more.
///
/// [`escape_default`]: ../../std/primitive.char.html#method.escape_default
/// [`char`]: ../../std/primitive.char.html
#[derive(Clone, Debug)]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct EscapeDefault {
    state: EscapeDefaultState
}

#[derive(Clone, Debug)]
enum EscapeDefaultState {
    Done,
    Char(char),
    Backslash(char),
    Unicode(EscapeUnicode),
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Iterator for EscapeDefault {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        match self.state {
            EscapeDefaultState::Backslash(c) => {
                self.state = EscapeDefaultState::Char(c);
                Some('\\')
            }
            EscapeDefaultState::Char(c) => {
                self.state = EscapeDefaultState::Done;
                Some(c)
            }
            EscapeDefaultState::Done => None,
            EscapeDefaultState::Unicode(ref mut iter) => iter.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.len();
        (n, Some(n))
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    fn nth(&mut self, n: usize) -> Option<char> {
        match self.state {
            EscapeDefaultState::Backslash(c) if n == 0 => {
                self.state = EscapeDefaultState::Char(c);
                Some('\\')
            },
            EscapeDefaultState::Backslash(c) if n == 1 => {
                self.state = EscapeDefaultState::Done;
                Some(c)
            },
            EscapeDefaultState::Backslash(_) => {
                self.state = EscapeDefaultState::Done;
                None
            },
            EscapeDefaultState::Char(c) => {
                self.state = EscapeDefaultState::Done;

                if n == 0 {
                    Some(c)
                } else {
                    None
                }
            },
            EscapeDefaultState::Done => return None,
            EscapeDefaultState::Unicode(ref mut i) => return i.nth(n),
        }
    }

    fn last(self) -> Option<char> {
        match self.state {
            EscapeDefaultState::Unicode(iter) => iter.last(),
            EscapeDefaultState::Done => None,
            EscapeDefaultState::Backslash(c) | EscapeDefaultState::Char(c) => Some(c),
        }
    }
}

#[stable(feature = "exact_size_escape", since = "1.11.0")]
impl ExactSizeIterator for EscapeDefault {
    fn len(&self) -> usize {
        match self.state {
            EscapeDefaultState::Done => 0,
            EscapeDefaultState::Char(_) => 1,
            EscapeDefaultState::Backslash(_) => 2,
            EscapeDefaultState::Unicode(ref iter) => iter.len(),
        }
    }
}

#[stable(feature = "fused", since = "1.26.0")]
impl FusedIterator for EscapeDefault {}

#[stable(feature = "char_struct_display", since = "1.16.0")]
impl fmt::Display for EscapeDefault {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for c in self.clone() {
            f.write_char(c)?;
        }
        Ok(())
    }
}

/// An iterator that yields the literal escape code of a `char`.
///
/// This `struct` is created by the [`escape_debug`] method on [`char`]. See its
/// documentation for more.
///
/// [`escape_debug`]: ../../std/primitive.char.html#method.escape_debug
/// [`char`]: ../../std/primitive.char.html
#[stable(feature = "char_escape_debug", since = "1.20.0")]
#[derive(Clone, Debug)]
pub struct EscapeDebug(EscapeDefault);

#[stable(feature = "char_escape_debug", since = "1.20.0")]
impl Iterator for EscapeDebug {
    type Item = char;
    fn next(&mut self) -> Option<char> { self.0.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.0.size_hint() }
}

#[stable(feature = "char_escape_debug", since = "1.20.0")]
impl ExactSizeIterator for EscapeDebug { }

#[stable(feature = "fused", since = "1.26.0")]
impl FusedIterator for EscapeDebug {}

#[stable(feature = "char_escape_debug", since = "1.20.0")]
impl fmt::Display for EscapeDebug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
