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
mod dictionary;
mod transliteration;

use std::process::ExitCode;
use clap::Parser;
use std::ffi::OsString;
use std::io::{self, BufRead, LineWriter};
use std::fmt;

#[derive(Parser)]
#[command(name = "Transliterate")]
struct Cli {
    dictionary: OsString,
}

struct CharRead<I: BufRead> {
    error: Option<io::Error>,
    buf_read: I,
}

impl<I: BufRead> Iterator for CharRead<I> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        match self.read_char() {
            Err(e) => {
                self.error = Some(e);
                None
            },
            Ok(c) => c,
        }
    }
}

impl<I: BufRead> CharRead<I> {
    fn new(buf_read: I) -> CharRead<I> {
        CharRead {
            error: None,
            buf_read,
        }
    }

    fn read_char(&mut self) -> Result<Option<char>, io::Error> {
        let mut buf = [0u8; 4];

        let got = self.buf_read.read(&mut buf[0..1])?;

        if got == 0 {
            return Ok(None);
        }

        let utf8_len = (buf[0].leading_ones() as usize).max(1).min(4);

        if utf8_len > 1 {
            self.buf_read.read_exact(&mut buf[1..utf8_len])?;
        }

        match std::str::from_utf8(&buf[0..utf8_len]) {
            Ok(s) => Ok(Some(s.chars().next().unwrap())),
            Err(_) => {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid UTF-8",
                ))
            },
        }
    }
}

struct Utf8Write<T: io::Write> {
    output: LineWriter<T>,
}

impl<T: io::Write> Utf8Write<T> {
    fn new(output: T) -> Utf8Write<T> {
        Utf8Write {
            output: LineWriter::new(output)
        }
    }
}

impl<T: io::Write> fmt::Write for Utf8Write<T> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        <LineWriter<T> as io::Write>::write_all(&mut self.output, s.as_bytes())
            .map_err(|_| fmt::Error)
    }
}

enum Error {
    Transliteration(transliteration::Error),
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Transliteration(e) => e.fmt(f),
            Error::Io(e) => e.fmt(f),
        }
    }
}

impl From<transliteration::Error> for Error {
    fn from(e: transliteration::Error) -> Error {
        Error::Transliteration(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

fn run_transliteration(
    dictionary: &[u8],
) -> Result<(), Error> {
    let mut input = CharRead::new(io::stdin().lock());
    let mut output = Utf8Write::new(io::stdout().lock());

    transliteration::transliterate(dictionary, &mut input, &mut output)?;

    match input.error {
        None => {
            <LineWriter<_> as std::io::Write>::flush(&mut output.output)?;
            Ok(())
        },
        Some(e) => Err(e.into()),
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let buf = match std::fs::read(&cli.dictionary) {
        Ok(buf) => buf,
        Err(e) => {
            eprintln!("{}: {}", cli.dictionary.to_string_lossy(), e);
            return ExitCode::FAILURE;
        },
    };

    match run_transliteration(&buf) {
        Err(Error::Transliteration(transliteration::Error::Dictionary(e))) => {
            eprintln!("{}: {}", cli.dictionary.to_string_lossy(), e);
            ExitCode::FAILURE
        },
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::FAILURE
        },
        Ok(_) => {
            ExitCode::SUCCESS
        }
    }
}
