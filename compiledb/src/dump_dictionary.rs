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

use std::process::ExitCode;
use std::fmt;

enum Error {
    UnexpectedEof,
    OffsetTooLong,
    InvalidCharacter,
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

fn read_node(buf: &[u8]) -> Result<Node, Error> {
    let mut sibling_offset = 0usize;
    let mut sibling_offset_len = 0usize;

    'load_sibling_offset: {
        for &byte in buf.iter() {
            if (sibling_offset_len + 1) * 7 > usize::BITS as usize {
                return Err(Error::OffsetTooLong);
            }

            sibling_offset |=
                (byte as usize & 0x7f) <<
                (sibling_offset_len * 7);
            sibling_offset_len += 1;

            if byte & 0x80 == 0 {
                break 'load_sibling_offset;
            }
        }

        return Err(Error::UnexpectedEof);
    }

    let buf = &buf[sibling_offset_len..];

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

    Ok(Node {
        char_offset: sibling_offset_len,
        data_offset: sibling_offset_len + utf8_len,
        ch,
        sibling_offset,
    })
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
            println!("{}", word);
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
