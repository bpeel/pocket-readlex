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

    fn flush_buf(&mut self) -> Result<(), Error> {
        if !self.buf.is_empty() {
            match dictionary::find_word(self.dictionary, &self.buf)? {
                Some(variant_pos) => {
                    let variant =
                        dictionary::extract_variant(
                            self.dictionary,
                            variant_pos
                        )?;
                    self.write_path(variant.translation)?;
                },
                None => self.output.write_str(&self.buf)?,
            }

            self.buf.clear();
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
