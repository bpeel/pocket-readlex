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
mod parts_of_speech;

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
    freq: u32,
}

struct FilteredEntry<'a> {
    latin: &'a str,
    shavian: &'a str,
    pos: Vec<u8>,
    ipa: &'a str,
    var: &'a str,
    freq: u32,
}

impl<'a> FilteredEntry<'a> {
    fn new(entry: &'a Entry, pos: Vec<u8>) -> FilteredEntry<'a> {
        FilteredEntry {
            latin: &entry.latin,
            shavian: &entry.shavian,
            pos,
            ipa: &entry.ipa,
            var: &entry.var,
            freq: entry.freq,
        }
    }

    fn matches(&self, entry: &Entry, pos: &Vec<u8>) -> bool {
        self.latin == entry.latin &&
            self.shavian == entry.shavian &&
            &self.pos == pos &&
            self.ipa == entry.ipa &&
            self.var == entry.var
    }
}

struct ArticleEntry<'a> {
    latin: &'a str,
    pos: Vec<u8>,
    variants: Vec<ArticleVariant<'a>>,
}

struct ArticleVariant<'a> {
    shavian: &'a str,
    ipa: String,
    var: u8,
}

static PARTS_OF_SPEECH_REMAP: [(&'static str, &'static str); 20] = [
    // I’m not sure if this is a mistake in the ReadLex. The code
    // isn’t mentioned in the BNC Basic Tagset and it only appears
    // once in the ReadLex.
    ("P0", "NP0"),
    // These are special classes just for the verbs “to be”, “to do”
    // and “to have” but we can treat them just like any other verb.
    ("VBB", "VVB"),
    ("VBD", "VVD"),
    ("VBG", "VVG"),
    ("VBI", "VVB"),
    ("VBN", "VVN"),
    ("VBZ", "VVZ"),
    ("VDB", "VVB"),
    ("VDD", "VVD"),
    ("VDG", "VVG"),
    ("VDI", "VVB"),
    ("VDN", "VVN"),
    ("VDZ", "VVZ"),
    ("VHB", "VVB"),
    ("VHD", "VVD"),
    ("VHG", "VVG"),
    ("VHI", "VVB"),
    ("VHN", "VVN"),
    ("VHZ", "VVZ"),
    // The ReadLex seems to have the “finite base form” and the
    // “infinitive form”, but they are both presented as just “verb”
    // and only one of them is shown. Let’s filter the VVI form and
    // only ever use the VVB form.
    ("VVI", "VVB"),
];

static VARIATIONS: [&'static str; 6] = [
    "GenAm",
    "GenAus",
    "RRP",
    "RRPVar",
    "SSB",
    "TrapBath",
];

// The ReadLex data has some special codings in the IPA where the
// author wanted to leave the possibility of picking a different
// pronunciation. Lets remap them to concrete values.
static IPA_REMAP: [(char, &'static str); 5] = [
    ('I', "ə"),
    ('R', "(r)"),
    ('Æ', "æ"),
    ('Ə', "ə"),
    ('Ɑ', "ɑ"),
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

    parts_of_speech::NAMES.binary_search(&pos)
        .ok()
        .map(|pos| pos as u8)
}

fn filter_entries<'a>(
    entries_in: &'a [Entry],
) -> Result<Vec<FilteredEntry<'a>>, ()> {
    let mut entries_out = Vec::<FilteredEntry>::new();

    for new_entry in entries_in.iter() {
        let Some(pos) = lookup_pos(&new_entry.pos)
        else {
            eprintln!(
                "unknown part of speech “{}” for “{}/{}”",
                new_entry.pos,
                new_entry.latin,
                new_entry.shavian,
            );
            return Err(());
        };

        'find_entry: {
            for old_entry in entries_out.iter_mut() {
                if old_entry.matches(new_entry, &pos) {
                    if old_entry.freq < new_entry.freq {
                        old_entry.freq = new_entry.freq;
                    }
                    break 'find_entry;
                }
            }

            entries_out.push(FilteredEntry::new(&new_entry, pos));
        }
    }

    Ok(entries_out)
}

fn build_trie<P: AsRef<Path>>(
    map: &ReadLexMap,
    keys: &[&String],
    output: P,
) -> Result<(), ()> {
    let mut builder = TrieBuilder::new();

    for (article_num, &key) in keys.iter().enumerate() {
        for entry in filter_entries(&map[key])?.iter() {
            builder.add_word(
                &entry.shavian,
                &entry.latin,
                entry.pos[0],
                article_num as u16,
                // Sort by decreasing frequency
                u32::MAX - entry.freq,
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

fn lookup_pos(pos: &str) -> Option<Vec<u8>> {
    let mut result = Vec::<u8>::new();

    for pos in pos.split('+') {
        match remap_pos(pos) {
            Some(pos) => {
                result.push(pos);
            },
            None => return None,
        }
    }

    Some(result)
}

fn remap_ipa(ipa: &str) -> String {
    let mut buf = String::new();

    for ch in ipa.chars() {
        match IPA_REMAP.binary_search_by_key(&ch, |&(ch, _)| ch) {
            Ok(index) => buf.push_str(IPA_REMAP[index].1),
            Err(_) => buf.push(ch),
        }
    }

    buf
}

fn combine_variants(article: &[Entry]) -> std::io::Result<Vec<ArticleEntry>> {
    let mut entries = Vec::<ArticleEntry>::new();

    for entry in filter_entries(article).unwrap().iter() {
        let variant = ArticleVariant {
            shavian: entry.shavian,
            ipa: remap_ipa(entry.ipa),
            var: lookup_var(entry.var)?,
        };

        if let Some(last_entry) = entries.last_mut() {
            if last_entry.latin == entry.latin
                && last_entry.pos == entry.pos
            {
                last_entry.variants.push(variant);
                continue;
            }
        }

        entries.push(ArticleEntry {
            latin: &entry.latin,
            pos: entry.pos.clone(),
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
