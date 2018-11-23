use std::collections::HashMap;
use std::num::NonZeroU16;
use std::sync::Mutex;

use smallvec::SmallVec;

lazy_static! {
    static ref STORAGE: ExtraCharStorage = ExtraCharStorage::new();
}

pub struct ExtraCharStorage {
    storage: Mutex<HashMap<NonZeroU16, SmallVec<[char; 5]>>>,
}

impl ExtraCharStorage {
    fn new() -> Self {
        Self {
            storage: Mutex::new(HashMap::new()),
        }
    }

    fn put(&self, c: SmallVec<[char; 5]>) -> NonZeroU16 {
        let mut storage = self.storage.lock().unwrap();

        let next_key = {
            let mut keys: Vec<&NonZeroU16> = storage.keys().collect();
            keys.sort();

            let mut next_key = unsafe { NonZeroU16::new_unchecked(0) };
            for key in keys {
                if key != &next_key {
                    break;
                }
                next_key = unsafe { NonZeroU16::new_unchecked(next_key.get() + 1) };
            }

            next_key
        };

        storage.insert(next_key, c);
        next_key
    }

    fn get(&self, index: &NonZeroU16) -> Option<SmallVec<[char; 5]>> {
        let storage = self.storage.lock().unwrap();
        storage.get(index).map(|sv| sv.clone())
    }

    fn remove(&self, index: &NonZeroU16) {
        let mut storage = self.storage.lock().unwrap();
        storage.remove(index);
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ExtraCharStorageIndex(Option<NonZeroU16>);

impl ExtraCharStorageIndex {
    pub fn new() -> Self {
        ExtraCharStorageIndex(None)
    }
}

impl Clone for ExtraCharStorageIndex {
    fn clone(&self) -> Self {
        // if let Some(ref index) = self.0 {
        //     if let Some(extra_chars) = STORAGE.get(index) {
        //         return ExtraCharStorageIndex(Some(STORAGE.put(extra_chars)));
        //     }
        // }
        ExtraCharStorageIndex(None)
    }
}

impl Drop for ExtraCharStorageIndex {
    fn drop(&mut self) {
        if let Some(ref index) = self.0 {
            STORAGE.remove(index);
        }
    }
}
