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
use nom::multi::length_count;
use nom::number::complete::be_u16;
use nom::number::complete::be_u8;
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
    let (input_3, constant_pool) = length_count(be_u16, parse_constant_pool_entry_from_bytes)(input_2)?;

    Ok((
        input_3,
        Classfile {
            version,
            constant_pool,
        },
    ))
}

fn parse_constant_pool_entry_from_bytes<'a>(
    bytes: &[u8],
) -> IResult<&[u8], ConstantPoolEntry<'a>> {
    let (input_1, tag) = be_u8(bytes)?;

    todo!()
}

fn classfile_version_from_bytes(bytes: &[u8]) -> IResult<&[u8], Version> {
    let (input_1, minor) = be_u16(bytes)?;
    let (input_2, major) = be_u16(input_1)?;

    Ok((input_2, Version { minor, major }))
}
