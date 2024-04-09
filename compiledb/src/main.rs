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

mod shavian;
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
    pos: String,
    var: String,
    freq: u32,
}

static BANNED_POSITIONS: [&'static str; 1] = [
    "NP0",
];

static ALLOWED_VARIATIONS: [&'static str; 1] = [
    "RRP",
];

impl Entry {
    fn is_allowed(&self) -> bool {
        // Allow only shavian letters, ie, no punctuation
        self.shavian.chars().all(|ch| shavian::is_shavian(ch))
        // Must be five letters long
            && self.shavian.chars().count() == 5
        // No banned positions
            && BANNED_POSITIONS.iter().find(|&p| p == &self.pos).is_none()
        // Only certain variations allowed
            && ALLOWED_VARIATIONS.iter().find(|&v| v == &self.var).is_some()
    }
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
        .filter(Entry::is_allowed)
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
