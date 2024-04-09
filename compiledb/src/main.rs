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

use std::process::ExitCode;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::BufWriter;
use std::fs::File;
use trie_builder::TrieBuilder;

static DICTIONARY_FILENAME: &'static str = "data/dictionary.bin";

#[derive(Deserialize)]
struct Entry {
    #[serde(rename = "Latn")]
    latin: String,
    #[serde(rename = "Shaw")]
    shavian: String,
}

type ReadLexMap = HashMap<String, Vec<Entry>>;

fn main() -> ExitCode {
    let map = match serde_json::from_reader::<_, ReadLexMap>(std::io::stdin()) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", e);
            return ExitCode::FAILURE;
        },
    };

    let mut builder = TrieBuilder::new();
    let entries = map.into_values()
        .flatten()
        .collect::<Vec::<Entry>>();

    for entry in entries.iter() {
        builder.add_word(&entry.shavian);
    }

    if let Err(e) = File::create(DICTIONARY_FILENAME).and_then(|file| {
        builder.into_dictionary(&mut BufWriter::new(file))
    }) {
        eprintln!("{}: {}", DICTIONARY_FILENAME, e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
