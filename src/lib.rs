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

#![allow(unused_imports)]  //TODO: remove, eventually!

use parking_lot::RwLock;

use std::collections::HashMap;
use std::io;
//use std::io::{Error, ErrorKind};
//use std::error::Error;

use std::sync::Arc;

//use custom_error::custom_error;
////custom_error!{ NotFound{key:Vec<u8>} = format!("Attempted to delete inexisting key '{}'", String::from_utf8(key).unwrap()) }
////custom_error!{ NotFound{key:Vec<u8>} = "Attempted to delete inexisting key '{key}'" }
//custom_error!{ #[derive(PartialEq,PartialOrd)] pub MyError2
//    NotFound{key: Vec<u8>, source: io::Error} =@{ //TODO: how to use that 'source' when returning NotFound?!
//        format!("Attempted to delete inexisting key '{}'", String::from_utf8(key.to_vec()).unwrap())
//    },
//}

//#[derive(Debug, PartialEq, Clone)]
#[derive(Debug, PartialEq)]
pub enum MyError {
    NotFound { key: Vec<u8> },
    //IOError { source: std::io::Error },
    //IOError { source: std::io::ErrorKind },
    //Io{ source: std::io::Error },
    Err41,
}

//impl std::error::Error for MyError {}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter)
    -> std::fmt::Result {
        match self {
            MyError::NotFound { key } => write!(f, "Attempted to delete inexisting key '{}'" ,
                                                String::from_utf8(key.to_vec()).unwrap()),
            //MyError::IOError { source } => write!(f, "{:#?}", source), //FIXME; do I need ErrorKind or io::Error?
            MyError::Err41 => write!(f, "Sit by a lake"),
        }
    }
}
////doneishTODO: https://doc.rust-lang.org/std/error/trait.Error.html#method.source
//impl Error for MyError {
//    fn description(&self) -> &str {
//        "I'm the superhero of errors"
//    }
//
//    fn source(&self) -> Option<&(dyn Error + 'static)> {
//        Some(&self.side) //uhmm....
//    }
//}
//impl std::convert::From<std::io::ErrorKind> for MyError {
//    fn from(kind: std::io::ErrorKind) -> MyError {
//        MyError::IOError{ source: kind }
//    }
//}
impl std::convert::From<MyError> for std::io::Error {
    fn from(kind: MyError) -> std::io::Error {
        //MyError::IOError{ source: kind }
        std::io::Error::new(std::io::ErrorKind::NotFound, kind.to_string())
    }
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
    //pub async fn del(&mut self, key: impl AsRef<[u8]>) -> io::Result<Vec<u8>> {
    pub async fn del(&mut self, key: impl AsRef<[u8]>) -> std::result::Result<Vec<u8>, MyError> {
        let key = key.as_ref().to_owned();
        let hashmap = &mut self.hashmap.write();
        let res=hashmap.remove(&key);
        match res {
            Some(prev_val) => {
                //Err::<(), Option<Vec<u8>>>(Some(prev_val))
                return Ok(prev_val);
            },
            None => {
                //Ok::<(), Option<Vec<u8>>>(()) // bad
                //return Err(Error::new(ErrorKind::NotFound, //this is good aka way 1
                //           format!("Attempted to delete inexisting key '{}'", String::from_utf8(key).unwrap())));
                //return Err(MyError2::NotFound{ key }); //TODO: have to specify 'source' ?? what?
                return Err(MyError::NotFound{ key });
            },
        }
    }
}
