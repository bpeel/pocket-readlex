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
use jni::objects::{JClass, JObject, JByteArray, JStaticMethodID, ReleaseMode};
use jni::objects::JMethodID;
use jni::sys::jvalue;
use jni::signature::{ReturnType, Primitive};
use std::fmt;
use crate::transliteration;

pub struct CharSequenceIter<'local> {
    env: JNIEnv<'local>,
    input: JObject<'local>,
    character_class: JClass<'local>,
    code_point_at_method: JStaticMethodID,
    offset_by_code_points_method: JStaticMethodID,
    length: i32,
    pos: i32,
}

impl<'local> CharSequenceIter<'local> {
    pub fn new(
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

    pub fn next_char(&mut self) -> Result<Option<char>, jni::errors::Error> {
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
