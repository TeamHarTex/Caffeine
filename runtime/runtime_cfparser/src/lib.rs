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

pub mod class;

use anyhow::{ensure, Result};
use crate::class::ClassFile;

/// A parser for a Java class file (`.class` file).
///
/// This operates on a slice for optimum performance.
pub struct ClassParser<'class> {
    bytes: &'class [u8],
    position: usize,
}

impl<'class> ClassParser<'class> {
    /// Construct a new [`ClassParser`] instance by providing a slice of bytes
    /// to parse from.
    ///
    /// [`ClassParser`]: crate::ClassParser
    pub fn new(bytes: &'class [u8]) -> Self {
        Self { bytes, position: 0 }
    }
}

impl<'class> ClassParser<'class> {
    /// Reads a 1-byte unsigned value from the bytes of the class file if it succeeds
    /// in doing so.
    fn u1(&'class mut self) -> Result<u8> {
        ensure!(self.position + 1 < self.bytes.len(), "insufficient data in provided bytes");

        let result = u8::from_be(self.bytes[self.position]);
        self.position += 1;

        Ok(result)
    }

    /// Reads a 2-byte unsigned value from the bytes of the class file if it succeeds
    /// in doing so.
    fn u2(&'class mut self) -> Result<u16> {
        ensure!(self.position + 2 < self.bytes.len(), "insufficient data in provided bytes");

        let data= (&self.bytes[self.position..self.position + 2]).try_into()?;
        let result = u16::from_be_bytes(data);
        self.position += 1;

        Ok(result)
    }

    /// Reads a 4-byte unsigned value from the bytes of the class file if it succeeds
    /// in doing so.
    fn u4(&'class mut self) -> Result<u32> {
        ensure!(self.position + 4 < self.bytes.len(), "insufficient data in provided bytes");

        let data= (&self.bytes[self.position..self.position + 4]).try_into()?;
        let result = u32::from_be_bytes(data);
        self.position += 1;

        Ok(result)
    }
}

impl<'class> ClassParser<'class> {
    /// Parses a [`ClassFile`] and returns it if it succeeds in doing so.
    /// 
    /// [`ClassFile`]: crate::class::ClassFile
    pub fn parse(&'class mut self) -> Result<ClassFile> {
        todo!()
    }
}
