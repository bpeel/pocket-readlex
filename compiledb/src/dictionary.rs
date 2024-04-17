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

use super::bit_reader::BitReader;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    UnexpectedEof,
    InvalidLengthHeader,
    OffsetTooLong,
    InvalidCharacter,
    ChildIndexOutOfRange,
}

impl From<std::str::Utf8Error> for Error {
    fn from(_: std::str::Utf8Error) -> Error {
        Error::InvalidCharacter
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::UnexpectedEof => write!(f, "unexpected EOF"),
            Error::InvalidLengthHeader => write!(f, "invalid length header"),
            Error::OffsetTooLong => write!(f, "offset too long"),
            Error::InvalidCharacter => write!(f, "invalid character"),
            Error::ChildIndexOutOfRange => {
                write!(f, "child index out of range")
            },
        }
    }
}

struct StackEntry {
    word_length: usize,
    pos: usize,
}

struct Node {
    char_offset: usize,
    data_offset: usize,
    ch: char,
    sibling_offset: usize,
}

fn read_sibling_offset(buf: &[u8]) -> Result<(usize, usize), Error> {
    let mut offset = 0usize;
    let mut len = 0usize;

    for &byte in buf.iter() {
        if (len + 1) * 7 > usize::BITS as usize {
            return Err(Error::OffsetTooLong);
        }

        offset |= (byte as usize & 0x7f) << (len * 7);
        len += 1;

        if byte & 0x80 == 0 {
            return Ok((offset, len));
        }
    }

    return Err(Error::UnexpectedEof);
}

fn read_character(buf: &[u8]) -> Result<(char, usize), Error> {
    let Some(&byte) = buf.first()
    else {
        return Err(Error::UnexpectedEof);
    };

    let utf8_len = (byte.leading_ones() as usize).max(1);

    let Some(ch_data) = buf.get(0..utf8_len)
    else {
        return Err(Error::UnexpectedEof);
    };

    let ch = std::str::from_utf8(ch_data)?.chars().next().unwrap();

    Ok((ch, utf8_len))
}

fn read_node(buf: &[u8]) -> Result<Node, Error> {
    let (sibling_offset, sibling_offset_len) = read_sibling_offset(buf)?;

    let (ch, utf8_len) = read_character(&buf[sibling_offset_len..])?;

    Ok(Node {
        char_offset: sibling_offset_len,
        data_offset: sibling_offset_len + utf8_len,
        ch,
        sibling_offset,
    })
}

fn count_siblings(mut buf: &[u8]) -> Result<usize, Error> {
    let mut count = 1;

    loop {
        let (sibling_offset, sibling_offset_len) = read_sibling_offset(buf)?;

        if sibling_offset == 0 {
            break;
        }

        buf = &buf[sibling_offset + sibling_offset_len..];
        count += 1;
    }

    Ok(count)
}

fn skip_nodes(buf: &[u8], n_nodes: usize) -> Result<usize, Error> {
    let mut pos = 0;

    for _ in 0..n_nodes {
        let (sibling_offset, sibling_offset_len) =
            read_sibling_offset(&buf[pos..])?;
        pos += sibling_offset_len + sibling_offset;
    }

    Ok(pos)
}

pub struct PathWalker<'a> {
    buf: &'a [u8],
    node_pos: usize,
    reader: BitReader<'a>,
    found_end: bool,
}

impl<'a> PathWalker<'a> {
    pub fn new(buf: &'a [u8], pos: usize) -> PathWalker<'a> {
        PathWalker {
            buf,
            node_pos: 4,
            reader: BitReader::new(&buf[pos..]),
            found_end: false,
        }
    }

    fn next_char(&mut self) -> Result<Option<char>, Error> {
        if self.found_end {
            return Ok(None)
        }

        let n_children = count_siblings(&self.buf[self.node_pos..])?;

        let Some(child_index) = self.reader.read_bits(
            (u32::BITS - (n_children as u32 - 1).leading_zeros()) as u8
        ) else {
            return Err(Error::UnexpectedEof);
        };

        if child_index as usize >= n_children {
            return Err(Error::ChildIndexOutOfRange);
        }

        self.node_pos += skip_nodes(
            &self.buf[self.node_pos..],
            child_index as usize
        )?;

        // Skip the sibling offset
        let (_, sibling_offset_len) =
            read_sibling_offset(&self.buf[self.node_pos..])?;

        let (ch, ch_len) =
            read_character(&self.buf[self.node_pos + sibling_offset_len..])?;

        if ch == '\0' {
            self.found_end = true;
            Ok(None)
        } else {
            self.node_pos += sibling_offset_len + ch_len;

            Ok(Some(ch))
        }
    }
}

impl<'a> Iterator for PathWalker<'a> {
    type Item = Result<char, Error>;

    fn next(&mut self) -> Option<Result<char, Error>> {
        match self.next_char() {
            Ok(Some(ch)) => Some(Ok(ch)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

pub struct Variant<'a> {
    pos: usize,
    is_last: bool,
    pub payload: u8,
    pub article_num: u16,
    pub translation: PathWalker<'a>,
}

impl<'a> Variant<'a> {
    pub fn into_next_offset(mut self) -> Result<Option<usize>, Error> {
        if self.is_last {
            return Ok(None);
        }

        // Consume any remaining characters not yet visited by the
        // path walker so we can get an offset to the end of it.
        for ch in &mut self.translation {
            let _ = ch?;
        }

        Ok(Some(self.pos + self.translation.reader.bytes_consumed() + 3))
    }
}

pub fn extract_variant(buf: &[u8], pos: usize) -> Result<Variant, Error> {
    let Some(payload_and_article) = buf.get(pos..pos + 3)
    else {
        return Err(Error::UnexpectedEof);
    };

    let payload_byte = payload_and_article[0];
    let article_num = u16::from_le_bytes(
        payload_and_article[1..3].try_into().unwrap()
    );

    Ok(Variant {
        pos,
        payload: payload_byte & 0x7f,
        article_num,
        is_last: payload_byte & 0x80 == 0,
        translation: PathWalker::new(buf, pos + 3),
    })
}

pub fn check_length(buf: &[u8]) -> Result<(), Error> {
    if buf.len() < 4 {
        return Err(Error::UnexpectedEof);
    }

    let len = u32::from_le_bytes(buf[0..4].try_into().unwrap());

    if len as usize != buf.len() - 4 {
        Err(Error::InvalidLengthHeader)
    } else {
        Ok(())
    }
}

pub struct DictionaryWalker<'a> {
    buf: &'a [u8],
    word: String,
    stack: Vec<StackEntry>,
}

impl<'a> DictionaryWalker<'a> {
    pub fn new(buf: &[u8]) -> DictionaryWalker {
        Self::start_from(buf, 4)
    }

    pub fn start_from(buf: &[u8], pos: usize) -> DictionaryWalker {
        DictionaryWalker {
            buf,
            word: String::new(),
            stack: vec![StackEntry { word_length: 0, pos }],
        }
    }

    pub fn next(&mut self) -> Result<Option<(&str, usize)>, Error> {
        loop {
            let Some(entry) = self.stack.pop()
            else {
                break Ok(None);
            };

            self.word.truncate(entry.word_length);

            let node = read_node(&self.buf[entry.pos..])?;

            if node.sibling_offset > 0 {
                self.stack.push(StackEntry {
                    word_length: self.word.len(),
                    pos: entry.pos + node.char_offset + node.sibling_offset,
                });
            }

            if node.ch == '\0' {
                break Ok(Some((&self.word, entry.pos + node.data_offset)));
            }

            self.word.push(node.ch);

            self.stack.push(StackEntry {
                word_length: self.word.len(),
                pos: entry.pos + node.data_offset,
            });
        }
    }
}

fn find_sibling_for_character(
    buf: &[u8],
    mut pos: usize,
    ch: char,
) -> Result<Option<usize>, Error> {
    loop {
        let Some(buf) = buf.get(pos..)
        else {
            return Err(Error::UnexpectedEof);
        };

        let node = read_node(buf)?;

        if node.ch == ch {
            return Ok(Some(pos + node.data_offset));
        } else if node.sibling_offset == 0 {
            return Ok(None);
        }

        pos += node.char_offset + node.sibling_offset;
    }
}

// Walks through the trie using the path for the given prefix. If the
// path is found then it returns the offset of the first child after
// the prefix. Otherwise it returns None.
pub fn find_prefix(buf: &[u8], prefix: &str) -> Result<Option<usize>, Error> {
    let mut pos = 4;

    for ch in prefix.chars() {
        // The dictionary uses '\0' as a special marker so we can’t
        // find prefixes that contain it.
        if ch == '\0' {
            return Ok(None);
        }

        pos = match find_sibling_for_character(buf, pos, ch)? {
            Some(pos) => pos,
            None => return Ok(None),
        }
    }

    Ok(Some(pos))
}

// Finds the exact word in the dictionary. If it is found then the
// offset of the first variant is returned.
pub fn find_word(buf: &[u8], word: &str) -> Result<Option<usize>, Error> {
    match find_prefix(buf, word)? {
        Some(pos) => find_sibling_for_character(buf, pos, '\0'),
        None => Ok(None),
    }
}
