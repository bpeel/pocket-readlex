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
use std::io::{Write, BufWriter};
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
    #[arg(short, long, value_name = "DIR")]
    article_dir: Option<OsString>,
}

#[derive(Deserialize)]
struct Entry {
    #[serde(rename = "Latn")]
    latin: String,
    #[serde(rename = "Shaw")]
    shavian: String,
    pos: String,
    ipa: String,
    var: String,
}

struct ArticleEntry<'a> {
    latin: &'a str,
    pos: Vec<u8>,
    variants: Vec<ArticleVariant<'a>>,
}

struct ArticleVariant<'a> {
    shavian: &'a str,
    ipa: &'a str,
    var: u8,
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

static VARIATIONS: [&'static str; 6] = [
    "GenAm",
    "GenAus",
    "RRP",
    "RRPVar",
    "SSB",
    "TrapBath",
];

// The articles are really small. We want to split them into multiple
// files because they will probably be compressed in the app package
// so we can’t seek into the file to get the right position and
// therefore we can’t just have one big file with them all. However if
// we make a single file for each article that will make a lot of
// files and just the filenames will start to take up a lot of space.
// As a compromise the articles are grouped with this many in each
// file.
const ARTICLES_PER_FILE: usize = 128;

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

// Iterator adaptor to help filter out certain entries
struct EntryFilter<'a, T: Iterator<Item = &'a Entry>> {
    had_verb: bool,
    inner: T,
}

impl<'a, T: Iterator<Item = &'a Entry>> EntryFilter<'a, T> {
    fn new(inner: T) -> Self {
        EntryFilter {
            had_verb: false,
            inner,
        }
    }
}

impl<'a, T: Iterator<Item = &'a Entry>> Iterator for EntryFilter<'a, T> {
    type Item = &'a Entry;

    fn next(&mut self) -> Option<&'a Entry> {
        loop {
            if let Some(entry) = self.inner.next() {
                // The ReadLex seems to have the “finite base form”
                // and the “infinitive form”, but they are both
                // presented as just “verb” and only one of them is
                // shown. Let’s filter the second one out in the same
                // way.
                if entry.pos == "VVB" || entry.pos == "VVI" {
                    if self.had_verb {
                        continue;
                    }
                    self.had_verb = true;
                }

                break Some(entry);
            } else {
                break None;
            }
        }
    }
}

fn build_trie<P: AsRef<Path>>(
    map: &ReadLexMap,
    keys: &[&String],
    output: P,
) -> Result<(), ()> {
    let mut builder = TrieBuilder::new();

    for (article_num, &key) in keys.iter().enumerate() {
        for entry in EntryFilter::new(map[key].iter()) {
            let Some(pos) = remap_pos(&entry.pos)
            else {
                eprintln!(
                    "unknown part of speech “{}” for “{}/{}”",
                    entry.pos,
                    entry.latin,
                    entry.shavian,
                );
                return Err(());
            };

            builder.add_word(
                &entry.shavian,
                &entry.latin,
                pos as u8,
                article_num as u16,
            );
        }
    }

    if let Err(e) = File::create(&output).and_then(|file| {
        builder.into_dictionary(&mut BufWriter::new(file))
    }) {
        eprintln!("{}: {}", output.as_ref().to_string_lossy(), e);
        return Err(());
    }

    Ok(())
}

fn write_string(s: &str, output: &mut impl Write) -> std::io::Result<()> {
    output.write_all(&[s.len() as u8])?;
    output.write_all(s.as_bytes())
}

fn lookup_var(var: &str) -> std::io::Result<u8> {
    match VARIATIONS.binary_search(&var) {
        Ok(var_pos) => Ok(var_pos as u8),
        Err(_) => {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("unknown variation: {}", var),
            ))
        },
    }
}

fn lookup_pos(pos: &str) -> std::io::Result<Vec<u8>> {
    let mut result = Vec::<u8>::new();

    for pos in pos.split('+') {
        match remap_pos(pos) {
            Some(pos) => {
                result.push(pos);
            },
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("unknown part of speech: {}", pos),
                ));
            },
        }
    }

    Ok(result)
}

fn combine_variants(article: &[Entry]) -> std::io::Result<Vec<ArticleEntry>> {
    let mut entries = Vec::<ArticleEntry>::new();

    for entry in EntryFilter::new(article.iter()) {
        let variant = ArticleVariant {
            shavian: &entry.shavian,
            ipa: &entry.ipa,
            var: lookup_var(&entry.var)?,
        };

        let pos = lookup_pos(&entry.pos)?;

        if let Some(last_entry) = entries.last_mut() {
            if last_entry.latin == entry.latin
                && last_entry.pos == pos
            {
                last_entry.variants.push(variant);
                continue;
            }
        }

        entries.push(ArticleEntry {
            latin: &entry.latin,
            pos,
            variants: vec![variant],
        });
    }

    Ok(entries)
}

fn write_article(
    article: &[Entry],
    output: &mut impl Write,
) -> std::io::Result<()> {
    let entries = combine_variants(article)?;

    let article_len = entries.iter().map(|entry| {
        1 + entry.latin.len()
            + 1 + entry.pos.len()
            + 1 + entry.variants.iter().map(|variant| {
                1
                    + 1 + variant.shavian.len()
                    + 1 + variant.ipa.len()
            }).sum::<usize>()
    }).sum::<usize>();

    output.write_all(&(article_len as u16).to_le_bytes())?;

    for entry in entries.iter() {
        write_string(&entry.latin, output)?;
        output.write_all(&[entry.pos.len() as u8])?;
        output.write_all(&entry.pos)?;
        output.write_all(&[entry.variants.len() as u8])?;

        for variant in entry.variants.iter() {
            output.write_all(&[variant.var])?;
            write_string(&variant.shavian, output)?;
            write_string(&variant.ipa, output)?;
        }
    }

    Ok(())
}

fn build_articles<P: AsRef<Path>>(
    map: &ReadLexMap,
    keys: &[&String],
    output: P,
) -> Result<(), ()> {
    if let Err(e) = std::fs::create_dir(&output) {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            eprintln!("{}: {}", output.as_ref().to_string_lossy(), e);
            return Err(());
        }
    }

    for (chunk_num, chunk) in keys.chunks(ARTICLES_PER_FILE).enumerate() {
        let filename = format!(
            "article-{:04x}.bin",
            chunk_num * ARTICLES_PER_FILE,
        );
        let path = output.as_ref().join(filename);

        if let Err(e) = File::create(&path).and_then(|file| {
            let mut writer = BufWriter::new(file);

            for &key in chunk.iter() {
                write_article(&map[key], &mut writer)?;
            }

            writer.flush()
        }) {
            eprintln!("{}: {}", path.to_string_lossy(), e);
            return Err(());
        }
    }

    Ok(())
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

    let mut keys = map.keys().collect::<Vec<_>>();
    // Sort the keys so that we can iterate the hash map in a
    // reproducible order.
    keys.sort_unstable();

    if build_trie(&map, &keys, &cli.output).is_err() {
        return ExitCode::FAILURE;
    }

    if let Some(article_dir) = cli.article_dir {
        if build_articles(&map, &keys, &article_dir).is_err() {
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}
