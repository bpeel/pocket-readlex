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

use jni::JNIEnv;
use jni::objects::{JClass, JObject, JByteArray, JStaticMethodID, ReleaseMode};
use jni::objects::JMethodID;
use jni::sys::jvalue;
use jni::signature::{ReturnType, Primitive};
use std::fmt;

struct CharSequenceIter<'local> {
    env: JNIEnv<'local>,
    input: JObject<'local>,
    character_class: JClass<'local>,
    code_point_at_method: JStaticMethodID,
    offset_by_code_points_method: JStaticMethodID,
    length: i32,
    pos: i32,
}

impl<'local> CharSequenceIter<'local> {
    fn new(
        mut env: JNIEnv<'local>,
        input: JObject<'local>,
    ) -> Result<CharSequenceIter<'local>, jni::errors::Error> {
        let length = env.call_method(
            &input,
            "length",
            "()I",
            &[],
        )?.i()?;

        let character_class = env.find_class("java/lang/Character")?;
        let code_point_at_method = env.get_static_method_id(
            &character_class,
            "codePointAt",
            "(Ljava/lang/CharSequence;I)I",
        )?;
        let offset_by_code_points_method = env.get_static_method_id(
            &character_class,
            "offsetByCodePoints",
            "(Ljava/lang/CharSequence;II)I",
        )?;
        Ok(CharSequenceIter {
            env,
            input,
            character_class,
            code_point_at_method,
            offset_by_code_points_method,
            length,
            pos: 0,
        })
    }

    fn next_char(&mut self) -> Result<Option<char>, jni::errors::Error> {
        if self.pos >= self.length {
            return Ok(None);
        }

        let ch = unsafe {
            self.env.call_static_method_unchecked(
                &self.character_class,
                self.code_point_at_method,
                ReturnType::Primitive(Primitive::Int),
                &[
                    jvalue { l: self.input.as_raw() },
                    jvalue { i: self.pos },
                ],
            )
        }?.i()?;

        self.pos = unsafe {
            self.env.call_static_method_unchecked(
                &self.character_class,
                self.offset_by_code_points_method,
                ReturnType::Primitive(Primitive::Int),
                &[
                    jvalue { l: self.input.as_raw() },
                    jvalue { i: self.pos },
                    jvalue { i: 1 },
                ],
            )
        }?.i()?;

        Ok(Some(
            char::from_u32(ch as u32)
                .unwrap_or(char::REPLACEMENT_CHARACTER)
        ))
    }
}

impl<'local> Iterator for CharSequenceIter<'local> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        self.next_char().unwrap_or(None)
    }
}

struct StringBuilderWriter<'local> {
    env: JNIEnv<'local>,
    output: JObject<'local>,
    append_code_point_method: JMethodID,
}

impl<'local> StringBuilderWriter<'local> {
    fn new(
        mut env: JNIEnv<'local>,
        output: JObject<'local>,
    ) -> Result<StringBuilderWriter<'local>, jni::errors::Error> {
        let string_builder_class = env.find_class("java/lang/StringBuilder")?;
        let append_code_point_method = env.get_method_id(
            &string_builder_class,
            "appendCodePoint",
            "(I)Ljava/lang/StringBuilder;",
        )?;

        Ok(StringBuilderWriter {
            env,
            output,
            append_code_point_method,
        })
    }
}

impl<'local> fmt::Write for StringBuilderWriter<'local> {
    fn write_char(&mut self, ch: char) -> fmt::Result {
        unsafe {
            self.env.call_method_unchecked(
                &self.output,
                self.append_code_point_method,
                ReturnType::Object,
                &[
                    jvalue { i: ch as i32 },
                ],
            )
        }.map_err(|_| fmt::Error).map(|_| ())
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        for ch in s.chars() {
            self.write_char(ch)?;
        }

        Ok(())
    }
}

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
