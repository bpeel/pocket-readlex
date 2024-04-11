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

mod trie_builder;
mod bit_writer;

use std::process::ExitCode;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::BufWriter;
use std::fs::File;
use trie_builder::TrieBuilder;
use clap::Parser;
use std::ffi::OsString;
use std::path::Path;

#[derive(Parser)]
#[command(name = "CompileDb")]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    input: OsString,
    #[arg(short, long, value_name = "FILE")]
    output: OsString,
}

#[derive(Deserialize)]
struct Entry {
    #[serde(rename = "Latn")]
    latin: String,
    #[serde(rename = "Shaw")]
    shavian: String,
    pos: String,
}

static PARTS_OF_SPEECH: [&'static str; 40] = [
    "AJ0", "AJC", "AJS", "AT0", "AV0", "AVP", "AVQ", "CJC", "CJS",
    "CJT", "CRD", "DPS", "DT0", "DTQ", "EX0", "ITJ", "NN0", "NN1",
    "NN2", "NP0", "ORD", "PNI", "PNP", "PNQ", "PNX", "POS", "PRE",
    "PRF", "PRP", "TO0", "UNC", "VM0", "VVB", "VVD", "VVG", "VVI",
    "VVN", "VVZ", "XX0", "ZZ0",
];

static PARTS_OF_SPEECH_REMAP: [(&'static str, &'static str); 19] = [
    // I’m not sure if this is a mistake in the ReadLex. The code
    // isn’t mentioned in the BNC Basic Tagset and it only appears
    // once in the ReadLex.
    ("P0", "NP0"),
    // These are special classes just for the verbs “to be”, “to do”
    // and “to have” but we can treat them just like any other verb.
    ("VBB", "VVB"),
    ("VBD", "VVD"),
    ("VBG", "VVG"),
    ("VBI", "VVI"),
    ("VBN", "VVN"),
    ("VBZ", "VVZ"),
    ("VDB", "VVB"),
    ("VDD", "VVD"),
    ("VDG", "VVG"),
    ("VDI", "VVI"),
    ("VDN", "VVN"),
    ("VDZ", "VVZ"),
    ("VHB", "VVB"),
    ("VHD", "VVD"),
    ("VHG", "VVG"),
    ("VHI", "VVI"),
    ("VHN", "VVN"),
    ("VHZ", "VVZ"),
];

type ReadLexMap = HashMap<String, Vec<Entry>>;

fn load_readlex<P: AsRef<Path>>(path: P) -> Result<ReadLexMap, String> {
    match File::open(path) {
        Ok(file) => {
            serde_json::from_reader::<_, ReadLexMap>(file)
                .map_err(|e| format!("{}", e))
        },
        Err(e) => Err(format!("{}", e))
    }
}

fn remap_pos(pos: &str) -> Option<u8> {
    // The part of speech can be a list of values seperated by
    // “+”. We’ll just take the first one.
    let first_pos = pos.split_once('+')
        .map(|(first, _)| first)
        .unwrap_or(pos);

    let pos = PARTS_OF_SPEECH_REMAP.binary_search_by_key(&first_pos, |(a, _)| a)
        .map(|map_index| PARTS_OF_SPEECH_REMAP[map_index].1)
        .unwrap_or(first_pos);

    PARTS_OF_SPEECH.binary_search(&pos)
        .ok()
        .map(|pos| pos as u8)
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let map = match load_readlex(&cli.input) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}: {}", cli.input.to_string_lossy(), e);
            return ExitCode::FAILURE;
        },
    };

    let mut builder = TrieBuilder::new();

    for article in map.into_values() {
        let mut had_verb = false;

        for entry in article.into_iter() {
            // The ReadLex seems to have the “finite base form” and
            // the “infinitive form”, but they are both presented as
            // just “verb” and only one of them is shown. Let’s filter
            // the second one out in the same way.
            if entry.pos == "VVB" || entry.pos == "VVI" {
                if had_verb {
                    continue;
                }
                had_verb = true;
            }

            let Some(pos) = remap_pos(&entry.pos)
            else {
                eprintln!(
                    "unknown part of speech “{}” for “{}/{}”",
                    entry.pos,
                    entry.latin,
                    entry.shavian,
                );
                return ExitCode::FAILURE;
            };

            builder.add_word(&entry.shavian, &entry.latin, pos as u8);
        }
    }

    if let Err(e) = File::create(&cli.output).and_then(|file| {
        builder.into_dictionary(&mut BufWriter::new(file))
    }) {
        eprintln!("{}: {}", cli.output.to_string_lossy(), e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
