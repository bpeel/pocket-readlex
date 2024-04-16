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

use std::process::ExitCode;

fn dump_dictionary(buf: &[u8]) -> Result<(), dictionary::Error> {
    dictionary::check_length(buf)?;

    let mut walker = dictionary::DictionaryWalker::new(buf);

    while let Some((word, mut variant_pos)) = walker.next()? {
        print!("{}", word);

        loop {
            let mut variant = dictionary::extract_variant(buf, variant_pos)?;

            print!(" ({}, {}, ", variant.payload, variant.article_num);

            while let Some(ch) = variant.translation.next() {
                let ch = ch?;

                print!("{}", ch);
            }

            print!(")");

            match variant.into_next_offset()? {
                Some(pos) => variant_pos = pos,
                None => break,
            }
        }

        println!();
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
