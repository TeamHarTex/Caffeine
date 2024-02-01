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

use std::borrow::Cow;

pub trait CowExt<'a> {
    fn to_str_lossy(self) -> Cow<'a, str>;
}

impl<'a> CowExt<'a> for Cow<'a, [u8]> {
    fn to_str_lossy(self) -> Cow<'a, str> {
        match self {
            Cow::Borrowed(slice) => String::from_utf8_lossy(slice),
            Cow::Owned(bytes) => String::from_utf8_lossy(&bytes),
        }
    }
}
