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

use jni::JNIEnv;
use jni::objects::{JClass, JObject, JByteArray, ReleaseMode};
use jni::objects::{JObjectArray, JMethodID};
use jni::sys::{jvalue, jsize};
use jni::signature::{ReturnType, Primitive};
use std::fmt::{self, Write};
use crate::dictionary;
use super::char_tools::{CharSequenceIter, StringBuilderWriter};

static SEARCH_RESULT_CLASS: &'static str =
    "uk/co/busydoingnothing/pocketrl/SearchResult";

struct Finder<'local, 'dictionary> {
    env: JNIEnv<'local>,
    dictionary_slice: &'dictionary [u8],
    prefix: JObject<'local>,
    buf_writer: StringBuilderWriter<'local>,
    buf: JObject<'local>,
    set_length_method: JMethodID,
    append_char_sequence_method: JMethodID,
    search_result_class: JClass<'local>,
    search_result_constructor: JMethodID,
    to_string_method: JMethodID,
}

impl<'local, 'dictionary> Finder<'local, 'dictionary> {
    fn clear_buffer(&mut self) -> Result<(), Error> {
        Ok(unsafe {
            self.env.call_method_unchecked(
                &self.buf,
                self.set_length_method,
                ReturnType::Primitive(Primitive::Void),
                &[jvalue { i: 0 }],
            )?;
        })
    }

    fn append_prefix(&mut self) -> Result<(), Error> {
        let string_buf = unsafe {
            self.env.call_method_unchecked(
                &self.buf,
                self.append_char_sequence_method,
                ReturnType::Object,
                &[jvalue { l: self.prefix.as_raw() }],
            )
        }?.l()?;

        self.env.auto_local(string_buf);

        Ok(())
    }

    fn buffer_to_string(&mut self) -> Result<JObject<'local>, Error> {
        Ok(unsafe {
            self.env.call_method_unchecked(
                &self.buf,
                self.to_string_method,
                ReturnType::Object,
                &[],
            )
        }?.l()?)
    }

    fn make_search_result(
        &mut self,
        word: &JObject<'local>,
        variant: &mut dictionary::Variant<'dictionary>,
    ) -> Result<JObject<'local>, Error> {
        self.clear_buffer()?;

        while let Some(ch) = variant.translation.next() {
            self.buf_writer.write_char(ch?)?;
        }

        let translation = self.buffer_to_string()?;
        let translation = self.env.auto_local(translation);

        Ok(unsafe {
            self.env.new_object_unchecked(
                &self.search_result_class,
                self.search_result_constructor,
                &[
                    jvalue { l: word.as_raw() },
                    jvalue { l: translation.as_raw() },
                    jvalue { b: variant.payload as i8 },
                    jvalue { i: variant.article_num as i32 },
                ],
            )
        }?)
    }

    fn add_variants(
        &mut self,
        word: &JObject<'local>,
        mut variant_pos: usize,
        mut result_num: jsize,
        results_length: jsize,
        results: &JObjectArray<'local>,
    ) -> Result<jsize, Error> {
        while result_num < results_length {
            let mut variant = dictionary::extract_variant(
                self.dictionary_slice,
                variant_pos,
            )?;

            let search_result = self.make_search_result(word, &mut variant)?;
            let search_result = self.env.auto_local(search_result);

            self.env.set_object_array_element(
                &results,
                result_num,
                &search_result,
            )?;

            result_num += 1;

            match variant.into_next_offset()? {
                Some(pos) => variant_pos = pos,
                None => break,
            }
        }

        Ok(result_num)
    }

    fn find_results(
        &mut self,
        prefix_pos: usize,
        results: JObjectArray<'local>,
    ) -> Result<i32, Error> {
        let mut walker = dictionary::DictionaryWalker::start_from(
            self.dictionary_slice,
            prefix_pos,
        );

        let results_length = self.env.get_array_length(&results)?;
        let mut result_num = 0;

        while result_num < results_length {
            let Some((word, variant_pos)) = walker.next()?
            else {
                break;
            };

            self.clear_buffer()?;
            self.append_prefix()?;
            self.buf_writer.write_str(&word)?;

            let word = self.buffer_to_string()?;
            let word = self.env.auto_local(word);

            result_num = self.add_variants(
                &word,
                variant_pos,
                result_num,
                results_length,
                &results,
            )?;
        }

        Ok(result_num)
    }

    fn search(
        &mut self,
        results: JObjectArray<'local>,
    ) -> Result<i32, Error> {
        match dictionary::find_prefix_iter(
            self.dictionary_slice,
            CharSequenceIter::new(
                unsafe { self.env.unsafe_clone() },
                self.env.new_local_ref(&self.prefix)?,
            )?,
        )? {
            Some(prefix_pos) => {
                self.find_results(
                    prefix_pos,
                    results
                )
            },
            None => Ok(0),
        }
    }
}

fn run_search<'local>(
    mut env: JNIEnv<'local>,
    dictionary: JByteArray<'local>,
    prefix: JObject<'local>,
    results: JObjectArray<'local>,
) -> Result<i32, Error> {
    let dictionary = unsafe {
        env.get_array_elements(
            &dictionary,
            ReleaseMode::NoCopyBack,
        )
    }?;

    // The AutoElements is of type i8 but we need a u8 slice :/
    let dictionary_slice = unsafe {
        std::slice::from_raw_parts(
            dictionary.as_ptr() as *const u8,
            dictionary.len(),
        )
    };

    let set_length_method = env.get_method_id(
        "java/lang/StringBuilder",
        "setLength",
        "(I)V",
    )?;

    let append_char_sequence_method = env.get_method_id(
        "java/lang/StringBuilder",
        "append",
        "(Ljava/lang/CharSequence;)Ljava/lang/StringBuilder;",
    )?;

    let search_result_class = env.find_class(SEARCH_RESULT_CLASS)?;

    let search_result_constructor = env.get_method_id(
        SEARCH_RESULT_CLASS,
        "<init>",
        "(Ljava/lang/String;Ljava/lang/String;BI)V",
    )?;

    let to_string_method = env.get_method_id(
        "java/lang/StringBuilder",
        "toString",
        "()Ljava/lang/String;",
    )?;

    let buf = env.new_object(
        "java/lang/StringBuilder",
        "()V",
        &[],
    )?;

    let buf_writer = StringBuilderWriter::new(
        unsafe { env.unsafe_clone() },
        env.new_local_ref(&buf)?,
    )?;

    let mut finder = Finder {
        env,
        dictionary_slice,
        prefix,
        set_length_method,
        append_char_sequence_method,
        buf,
        buf_writer,
        to_string_method,
        search_result_class,
        search_result_constructor,
    };

    finder.search(results)
}

#[derive(Debug)]
enum Error {
    Dictionary(dictionary::Error),
    Jni(jni::errors::Error),
    Format(fmt::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Dictionary(e) => e.fmt(f),
            Error::Jni(e) => e.fmt(f),
            Error::Format(e) => e.fmt(f),
        }
    }
}

impl From<dictionary::Error> for Error {
    fn from(e: dictionary::Error) -> Error {
        Error::Dictionary(e)
    }
}

impl From<jni::errors::Error> for Error {
    fn from(e: jni::errors::Error) -> Error {
        Error::Jni(e)
    }
}

impl From<fmt::Error> for Error {
    fn from(e: fmt::Error) -> Error {
        Error::Format(e)
    }
}

#[no_mangle]
pub extern "system" fn Java_uk_co_busydoingnothing_pocketrl_Compiledb_search<'local>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
    dictionary: JByteArray<'local>,
    prefix: JObject<'local>,
    results: JObjectArray<'local>,
) -> i32 {
    run_search(env, dictionary, prefix, results).unwrap()
}
