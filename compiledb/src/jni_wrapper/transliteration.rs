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
use std::fmt;
use super::char_tools::{CharSequenceIter, StringBuilderWriter};
use crate::transliteration;

#[derive(Debug)]
enum Error {
    Transliteration(transliteration::Error),
    Jni(jni::errors::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Transliteration(e) => e.fmt(f),
            Error::Jni(e) => e.fmt(f),
        }
    }
}

impl From<transliteration::Error> for Error {
    fn from(e: transliteration::Error) -> Error {
        Error::Transliteration(e)
    }
}

impl From<jni::errors::Error> for Error {
    fn from(e: jni::errors::Error) -> Error {
        Error::Jni(e)
    }
}

fn run_transliteration<'local>(
    mut env: JNIEnv<'local>,
    dictionary: JByteArray<'local>,
    input: JObject<'local>,
    output: JObject<'local>,
) -> Result<(), Error> {
    let input_iter = CharSequenceIter::new(
        unsafe { env.unsafe_clone() },
        input,
    )?;

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

    let output_writer = StringBuilderWriter::new(
        unsafe { env.unsafe_clone() },
        output,
    )?;

    transliteration::transliterate(
        &dictionary_slice,
        input_iter,
        output_writer,
    )?;

    Ok(())
}

#[no_mangle]
pub extern "system" fn Java_uk_co_busydoingnothing_pocketrl_Transliterater_transliterate<'local>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
    dictionary: JByteArray<'local>,
    input: JObject<'local>,
    output: JObject<'local>,
) {
    run_transliteration(env, dictionary, input, output).unwrap();
}
