// Pocket ReadLex – An offline app for ReadLex
// Copyright (C) 2024  Neil Roberts
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use super::dictionary;

use std::fmt;
use fmt::Write;

#[derive(Debug)]
pub enum Error {
    Dictionary(dictionary::Error),
    Format(std::fmt::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Dictionary(e) => e.fmt(f),
            Error::Format(e) => e.fmt(f),
        }
    }
}

impl From<dictionary::Error> for Error {
    fn from(e: dictionary::Error) -> Error {
        Error::Dictionary(e)
    }
}

impl From<fmt::Error> for Error {
    fn from(e: fmt::Error) -> Error {
        Error::Format(e)
    }
}

struct Transliterater<'a, I: IntoIterator<Item = char>, O: Write> {
    dictionary: &'a [u8],
    input: std::iter::Peekable<I::IntoIter>,
    output: O,
    buf: String,
}

impl<'a, I: IntoIterator<Item = char>, O: Write> Transliterater<'a, I, O> {
    fn new(dictionary: &'a [u8], input: I, output: O) -> Self {
        Transliterater {
            dictionary,
            input: input.into_iter().peekable(),
            output,
            buf: String::new(),
        }
    }

    fn write_path(
        &mut self,
        path: dictionary::PathWalker<'a>,
    ) -> Result<(), Error> {
        for ch in path {
            let ch = ch?;

            self.output.write_char(ch)?;
        }

        Ok(())
    }

    fn write_variant(&mut self, variant_pos: usize) -> Result<(), Error> {
        let variant = dictionary::extract_variant(
            self.dictionary,
            variant_pos,
        )?;
        self.write_path(variant.translation)
    }

    fn write_hyphenated_parts(&mut self, word: &str) -> Result<(), Error> {
        let mut parts = word.split('-').peekable();

        // If there are no hyphens then don’t bother looking up the word again
        if parts.peek().is_none() {
            self.output.write_str(word)?;
            return Ok(());
        }

        while let Some(part) = parts.next() {
            match dictionary::find_word(self.dictionary, part)? {
                Some(variant_pos) => self.write_variant(variant_pos)?,
                None => self.output.write_str(part)?,
            }

            if parts.peek().is_some() {
                self.output.write_char('-')?;
            }
        }

        Ok(())
    }

    fn flush_buf(&mut self) -> Result<(), Error> {
        if !self.buf.is_empty() {
            // Take the buffer so we can have a mutable reference to
            // it even while we call mutable methods on self. The
            // default String shouldn’t allocate so this shouldn’t
            // really cost anything.
            let mut buf = std::mem::take(&mut self.buf);

            match dictionary::find_word(self.dictionary, &buf)? {
                Some(variant_pos) => self.write_variant(variant_pos)?,
                None => self.write_hyphenated_parts(&buf)?,
            }

            buf.clear();
            // Put the buffer back so it can reuse the memory that it
            // probably reallocated
            self.buf = buf;
        }

        Ok(())
    }

    fn run(&mut self) -> Result<(), Error> {
        while let Some(ch) = self.input.next() {
            if ch.is_alphabetic() {
                for ch in ch.to_lowercase() {
                    self.buf.push(ch);
                }
            } else {
                if !self.buf.is_empty() &&
                    "'-’".chars().find(|&x| x == ch).is_some() {
                    // Accept any of these characters if they are in
                    // the middle of a word and followed by an
                    // alpahbetic character.
                    if let Some(next_ch) = self.input.peek() {
                        if next_ch.is_alphabetic() {
                            self.buf.push(if ch == '’' { '\'' } else { ch });
                            continue;
                        }
                    }
                }

                self.flush_buf()?;

                self.output.write_char(ch)?;
            }
        }

        self.flush_buf()
    }
}

pub fn transliterate<I: IntoIterator<Item = char>, O: Write>(
    dictionary: &[u8],
    input: I,
    output: O,
) -> Result<(), Error> {
    Transliterater::new(dictionary, input, output).run()
}

#[cfg(test)]
mod test {
    use super::*;

    static DICTIONARY: [u8; 58] = [
        // Length
        0, 0, 0, 0,
        7, b'a', 0, b'\0', 0, 0, 0, 1,
        7, b'b', 0, b'\0', 0, 0, 0, 0,
        7, b'c', 0, b'\0', 0, 0, 0, 3,
        17, b'd', 5, b'\0', 0, 0, 0, 2,
        0, b'\'', 0, b'b', 0, b'\0', 0, 0, 0, 1,
        0, b'e', 0, b'-', 0, b'f', 0, b'\0', 0, 0, 0, 1,
    ];

    fn transliterate_to_string(input: &str) -> Result<String, Error> {
        let mut output = String::new();
        transliterate(&DICTIONARY[..], input.chars(), &mut output)?;
        Ok(output)
    }

    #[test]
    fn hyphens() {
        assert_eq!(&transliterate_to_string("a").unwrap(), "b");
        assert_eq!(&transliterate_to_string("c").unwrap(), "d");
        // Fallback for a word that isn’t in the dictionary, it’s
        // individual parts should be translated instead.
        assert_eq!(&transliterate_to_string("a-c").unwrap(), "b-d");
        assert_eq!(&transliterate_to_string("a-c-d-b").unwrap(), "b-d-c-a");
        // Hyphenated words that are in the dictionary should use
        // their dictionary translation.
        assert_eq!(&transliterate_to_string("e-f").unwrap(), "b");
    }

    #[test]
    fn apostrophes() {
        // Apostrophes should be part of the word if they are followed
        // by a letter.
        assert_eq!(&transliterate_to_string("d'b").unwrap(), "b");
        assert_eq!(&transliterate_to_string("d’b").unwrap(), "b");
        // Otherwise no.
        assert_eq!(&transliterate_to_string("d' b").unwrap(), "c' a");
        assert_eq!(&transliterate_to_string("d’ b").unwrap(), "c’ a");
        assert_eq!(&transliterate_to_string("d'").unwrap(), "c'");
        assert_eq!(&transliterate_to_string("d’").unwrap(), "c’");
    }
}
