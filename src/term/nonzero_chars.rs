// Copyright 2016 Joe Wilm, The Alacritty Project Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use std::collections::BTreeMap;
use std::num::NonZeroU16;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref NONZERO_CHARS: Arc<Mutex<BTreeMap<NonzeroCharId, Vec<char>>>> =
        { Arc::new(Mutex::new(BTreeMap::new())) };
}

/// ID for looking up the extra chars for a specific cell. This cannot be `Clone` or `Copy`, since
/// that would create two cells with access to the same nonzero chars, which could cause a
/// use-after-free.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct NonzeroCharId(NonZeroU16);

impl NonzeroCharId {
    pub fn new() -> Self {
        let mut lock = NONZERO_CHARS.lock().expect("new nonzero poisoned");

        let mut new_index = 1;
        for key in lock.keys() {
            if key.0.get() != new_index {
                break;
            }

            if new_index == u16::max_value() {
                panic!("exceeded maximum zerowidth characters");
            }

            new_index += 1;
        }
        let new_index = unsafe { NonZeroU16::new_unchecked(new_index) };

        lock.insert(NonzeroCharId(new_index), Vec::new());

        NonzeroCharId(new_index)
    }

    pub fn get_chars(&self) -> Vec<char> {
        let lock = NONZERO_CHARS.lock().expect("get nonzero poisoned");
        lock.get(self).cloned().unwrap_or_else(|| Vec::new())
    }

    pub fn put_char(&mut self, c: char) {
        let mut lock = NONZERO_CHARS.lock().expect("put nonzero poisoned");
        if let Some(ref mut chars) = lock.get_mut(self) {
            chars.push(c);
        }
    }
}

impl Drop for NonzeroCharId {
    #[inline(always)]
    fn drop(&mut self) {
        panic!("NOOO");
        let mut lock = NONZERO_CHARS.lock().expect("drop nonzero poisoned");
        lock.remove(self);
    }
}
