//! Thread-safe in-memory key-value store. Ideal for development and prototyping.
//! Does not persist to disk.
//!
//! ## Examples
//!
//! ```
//! # #[runtime::main]
//! # async fn main() -> std::io::Result<()> {
//! let mut db = memdb::Memdb::open().await?;
//! db.set("beep", "boop").await?;
//! let val = db.get("beep").await?;
//! assert_eq!(val, Some("boop".as_bytes().to_owned()));
//! # Ok(())
//! # }
//! ```

use parking_lot::RwLock;

use std::collections::HashMap;
use std::io;
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use custom_error::custom_error;
//custom_error!{ NotFound{key:Vec<u8>} = format!("Attempted to delete inexisting key '{}'", String::from_utf8(key).unwrap()) }
//custom_error!{ NotFound{key:Vec<u8>} = "Attempted to delete inexisting key '{key}'" }
custom_error!{ pub MyError
    NotFound{key: Vec<u8>} =@{
        format!("Attempted to delete inexisting key '{}'", String::from_utf8(*key).unwrap())
    },
}

/// Key-value database.
#[derive(Debug, Clone)]
pub struct Memdb {
    hashmap: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl Memdb {
    /// Create a new instance.
    #[inline]
    pub async fn open() -> io::Result<Self> {
        Ok(Self {
            hashmap: Arc::new(RwLock::new(HashMap::<Vec<u8>, Vec<u8>>::new())),
        })
    }

    /// Set a value in the database.
    #[inline]
    pub async fn set(
        &mut self,
        key: impl AsRef<[u8]>,
        value: impl AsRef<[u8]>,
    ) -> io::Result<Option<Vec<u8>>> {
        let hashmap = self.hashmap.clone();
        let mut hashmap = hashmap.write();
        Ok(hashmap.insert(key.as_ref().to_owned(), value.as_ref().to_owned()))
    }

    /// Get a value from the database.
    #[must_use]
    #[inline]
    pub async fn get(&self, key: impl AsRef<[u8]>) -> io::Result<Option<Vec<u8>>> {
        let key = key.as_ref().to_owned();
        let hashmap = &self.hashmap.read();
        Ok(hashmap.get(&key).cloned())
    }

    /// Ensure a key doesn't exist in the db.
    /// doesn't fail if value doesn't exist
    #[inline]
    pub async fn ensure_del(&mut self, key: impl AsRef<[u8]>) -> io::Result<Option<Vec<u8>>> {
        let key = key.as_ref().to_owned();
        let hashmap = &mut self.hashmap.write();
        Ok(hashmap.remove(&key))
    }

    /// Delete a key from the database.
    /// fails if key didn't already exist
    //#[inline]  bad for tracing!  nope, that's not it, `cargo test` simply cannot show me the
    //exact line number for the failing assert_eq!
    pub async fn del(&mut self, key: impl AsRef<[u8]>) -> io::Result<Vec<u8>> {
        let key = key.as_ref().to_owned();
        let hashmap = &mut self.hashmap.write();
        let res=hashmap.remove(&key);
        match res {
            Some(prev_val) => {
                //Err::<(), Option<Vec<u8>>>(Some(prev_val))
                return Ok(prev_val);
            },
            None => {
                //Ok::<(), Option<Vec<u8>>>(())
                return Err(Error::new(ErrorKind::NotFound,
                           format!("Attempted to delete inexisting key '{}'", String::from_utf8(key).unwrap())));
            },
        }
    }
}
