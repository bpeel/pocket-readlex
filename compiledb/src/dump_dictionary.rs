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
use clap::Parser;
use std::ffi::OsString;

#[derive(Parser)]
#[command(name = "dump-dictionary")]
struct Cli {
    #[arg(short, long, value_name = "STR")]
    prefix: Option<String>,
    dictionaries: Vec<OsString>,
}

fn dump_dictionary(
    buf: &[u8],
    prefix: Option<&str>,
) -> Result<(), dictionary::Error> {
    dictionary::check_length(buf)?;

    let mut walker = match prefix {
        Some(prefix) => {
            let Some(start_pos) = dictionary::find_prefix(buf, prefix)?
            else {
                return Ok(());
            };

            dictionary::DictionaryWalker::start_from(buf, start_pos)
        },
        None => dictionary::DictionaryWalker::new(buf)
    };

    while let Some((word, mut variant_pos)) = walker.next()? {
        if let Some(prefix) = prefix {
            print!("{}", prefix);
        }

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
    let cli = Cli::parse();

    for arg in cli.dictionaries.iter() {
        let buf = match std::fs::read(&arg) {
            Ok(buf) => buf,
            Err(e) => {
                eprintln!("{}: {}", arg.to_string_lossy(), e);
                return ExitCode::FAILURE;
            },
        };

        if let Err(e) = dump_dictionary(&buf, cli.prefix.as_deref()) {
            eprintln!("{}: {}", arg.to_string_lossy(), e);
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}
