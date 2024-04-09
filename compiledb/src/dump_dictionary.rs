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

mod bit_reader;

use std::process::ExitCode;
use std::fmt;
use bit_reader::BitReader;

enum Error {
    UnexpectedEof,
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

fn dump_path(buf: &[u8], pos: usize) -> Result<usize, Error> {
    let mut node_pos = 0;
    let mut reader = BitReader::new(&buf[pos..]);

    loop {
        // Skip the sibling offset
        let (_, sibling_offset_len) = read_sibling_offset(&buf[node_pos..])?;

        let (ch, ch_len) =
            read_character(&buf[node_pos + sibling_offset_len..])?;

        if ch == '\0' {
            break;
        }

        print!("{}", ch);

        node_pos += sibling_offset_len + ch_len;

        let children = &buf[node_pos..];

        let n_children = count_siblings(children)?;

        let Some(child_index) = reader.read_bits(
            (u32::BITS - (n_children as u32 - 1).leading_zeros()) as u8
        ) else {
            return Err(Error::UnexpectedEof);
        };

        if child_index as usize >= n_children {
            return Err(Error::ChildIndexOutOfRange);
        }

        node_pos += skip_nodes(children, child_index as usize)?;
    }

    Ok(pos + reader.bytes_consumed())
}

fn dump_payload(buf: &[u8], mut pos: usize) -> Result<(), Error> {
    loop {
        let Some(&payload_byte) = buf.get(pos)
        else {
            return Err(Error::UnexpectedEof);
        };

        print!(" ({}, ", payload_byte & 0x7f);

        pos = dump_path(buf, pos + 1)?;

        print!(")");

        if payload_byte & 0x80 == 0 {
            break;
        }
    }

    Ok(())
}

fn dump_dictionary(buf: &[u8]) -> Result<(), Error> {
    let mut word = String::new();
    let mut stack = vec![StackEntry { word_length: 0, pos: 0 }];

    while let Some(entry) = stack.pop() {
        word.truncate(entry.word_length);

        let node = read_node(&buf[entry.pos..])?;

        if node.sibling_offset > 0 {
            stack.push(StackEntry {
                word_length: word.len(),
                pos: entry.pos + node.char_offset + node.sibling_offset,
            });
        }

        if node.ch == '\0' {
            print!("{}", word);

            dump_payload(buf, entry.pos + node.data_offset)?;

            println!();
        } else {
            word.push(node.ch);

            stack.push(StackEntry {
                word_length: word.len(),
                pos: entry.pos + node.data_offset,
            });
        }
    }

    Ok(())
}

fn main() -> ExitCode {
    for arg in std::env::args_os().skip(1) {
        let buf = match std::fs::read(&arg) {
            Ok(buf) => buf,
            Err(e) => {
                eprintln!("{}: {}", arg.to_string_lossy(), e);
                return ExitCode::FAILURE;
            },
        };

        if let Err(e) = dump_dictionary(&buf) {
            eprintln!("{}: {}", arg.to_string_lossy(), e);
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}
