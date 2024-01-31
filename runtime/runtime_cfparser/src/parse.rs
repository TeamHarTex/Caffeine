/*
 * Copyright (c) 2024 The Caffeine Project Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use nom::bytes::complete::tag;
use nom::bytes::complete::take;
use nom::error::Error;
use nom::error::ErrorKind;
use nom::multi::length_count;
use nom::number::complete::be_u16;
use nom::number::complete::be_u32;
use nom::number::complete::be_u8;
use nom::Err;
use nom::IResult;

use crate::spec::Classfile;
use crate::spec::ConstantPoolEntry;
use crate::spec::Version;

pub fn classfile_from_bytes(bytes: &[u8]) -> IResult<&[u8], Classfile> {
    // make sure the magic bytes are there, to indicate a valid Java classfile
    let (input_1, _) = tag([0xCA, 0xFE, 0xBA, 0xBE])(bytes)?;

    // parse classfile version
    let (input_2, version) = classfile_version_from_bytes(input_1)?;

    // parse constant pool length and constant pool
    let (input_3, constant_pool) = length_count(be_u16, constant_pool_entry_from_bytes)(input_2)?;

    Ok((
        input_3,
        Classfile {
            version,
            constant_pool,
        },
    ))
}

fn classfile_version_from_bytes(bytes: &[u8]) -> IResult<&[u8], Version> {
    let (input_1, minor) = be_u16(bytes)?;
    let (input_2, major) = be_u16(input_1)?;

    Ok((input_2, Version { minor, major }))
}

fn constant_pool_entry_from_bytes<'a>(bytes: &'a [u8]) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input, tag) = be_u8(bytes)?;

    match tag {
        1 => constant_pool_utf8_entry_from_bytes(input),
        3 => constant_pool_integer_entry_from_bytes(input),
        4 => constant_pool_float_entry_from_bytes(input),
        5 => constant_pool_long_entry_from_bytes(input),
        6 => constant_pool_double_entry_from_bytes(input),
        7 => constant_pool_class_entry_from_bytes(input),
        8 => constant_pool_string_entry_from_bytes(input),
        _ => Err(Err::Error(Error::new(bytes, ErrorKind::Tag))),
    }
}

fn constant_pool_class_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input, name_index) = be_u16(bytes)?;

    Ok((input, ConstantPoolEntry::Class { name_index }))
}

fn constant_pool_double_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input_1, high_bytes) = be_u32(bytes)?;
    let (input_2, low_bytes) = be_u32(input_1)?;

    Ok((
        input_2,
        ConstantPoolEntry::Double {
            high_bytes,
            low_bytes,
        },
    ))
}

fn constant_pool_float_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input, float) = be_u32(bytes)?;

    Ok((input, ConstantPoolEntry::Float { bytes: float }))
}

fn constant_pool_field_ref_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input_1, class_index) = be_u16(bytes)?;
    let (input_2, name_and_type_index) = be_u16(input_1)?;

    Ok((
        input_2,
        ConstantPoolEntry::FieldRef {
            class_index,
            name_and_type_index,
        },
    ))
}

fn constant_pool_integer_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input, integer) = be_u32(bytes)?;

    Ok((input, ConstantPoolEntry::Utf8 { bytes: integer }))
}

fn constant_pool_long_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input_1, high_bytes) = be_u32(bytes)?;
    let (input_2, low_bytes) = be_u32(input_1)?;

    Ok((
        input_2,
        ConstantPoolEntry::Long {
            high_bytes,
            low_bytes,
        },
    ))
}

fn constant_pool_string_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input, string_index) = be_u16(bytes)?;

    Ok((input, ConstantPoolEntry::String { string_index }))
}

fn constant_pool_utf8_entry_from_bytes<'a>(
    bytes: &'a [u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input_1, length) = be_u16(bytes)?;
    let (input_2, str_bytes) = take(length as usize)(input_1)?;

    Ok((input_2, ConstantPoolEntry::Utf8 { bytes: str_bytes }))
}
