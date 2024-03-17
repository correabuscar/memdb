//! Thread-safe in-memory key-value store. Ideal for development and prototyping.
//! Does not persist to disk.
//!
//! ## Examples
//!
//! ```
//! # #[async_std::main]
//! # async fn main() -> std::io::Result<()> {
//! let mut db = memdb::Memdb::open().await?;
//! db.set("beep", "boop").await?;
//! let val = db.get("beep").await?;
//! assert_eq!(val, Some("boop".as_bytes().to_owned()));
//! # Ok(())
//! # }
//! ```

#![allow(unused_imports)]  //TODO: remove, eventually!


use dashmap::DashMap;
use std::io;
//use std::io::{Error, ErrorKind};
//use std::error::Error;

use std::sync::Arc;

pub type Result<T> = std::result::Result<T, MyError>;


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
    //Io{ source: std::io::Error }, //binary operation `!=` cannot be applied to type `std::io::Error`
}

//impl std::error::Error for MyError {}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter)
    -> std::fmt::Result {
        match self {
            MyError::NotFound { key } => write!(f, "Attempted to delete inexisting key '{}'" ,
                                                String::from_utf8(key.to_vec()).unwrap()),
            //MyError::IOError { source } => write!(f, "{:#?}", source), //FIXME; do I need ErrorKind or io::Error?
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
    hashmap: Arc<DashMap<Vec<u8>, Vec<u8>>>,
}

impl Memdb {
    /// Create a new instance.
    #[inline]
    pub async fn open() -> io::Result<Self> {
        Ok(Self {
            hashmap: Arc::new(DashMap::<Vec<u8>, Vec<u8>>::new()),
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
        Ok(hashmap.insert(key.as_ref().to_owned(), value.as_ref().to_owned()))
    }

    /// Get a value from the database.
    #[must_use]
    #[inline]
    pub async fn get(&self, key: impl AsRef<[u8]>) -> io::Result<Option<Vec<u8>>> {
        let key = key.as_ref().to_owned();
//        let hashmap = &self.hashmap.read();
//        Ok(hashmap.get(&key).cloned())
        let hashmap = &self.hashmap;
        Ok(match hashmap.get(&key) {
            Some(value) => {
                let value = value.clone();
                Some(value)
            }
            None => None,
        })
    }

    /// Ensure a key doesn't exist in the db.
    /// doesn't fail if value doesn't exist
    #[inline]
    pub async fn ensure_del(&mut self, key: impl AsRef<[u8]>) -> io::Result<Option<Vec<u8>>> {
        let key = key.as_ref().to_owned();
        //let hashmap = &mut self.hashmap.write();
        let hashmap = &mut self.hashmap;
        return Ok(match hashmap.remove(&key) {
            Some((_,prev_val)) => {
                Some(prev_val)
            },
            None => {
                None
            }
        });
    }

    /// Delete a key from the database.
    /// fails if key didn't already exist
    //#[inline]  bad for tracing!  nope, that's not it, `cargo test` simply cannot show me the
    //exact line number for the failing assert_eq!
    //pub async fn del(&mut self, key: impl AsRef<[u8]>) -> io::Result<Vec<u8>> {
    pub async fn del(&mut self, key: impl AsRef<[u8]>) -> std::result::Result<Vec<u8>, MyError> {
//    pub async fn del(&mut self, key: impl AsRef<[u8]>) -> Result<Vec<u8>> {
//    pub async fn del(&mut self, key: impl AsRef<[u8]>) -> io::Result<Option<(Vec<u8>, Vec<u8>)>> {
//    pub async fn del(&mut self, key: impl AsRef<[u8]>) -> io::Result<Option<Vec<u8>>> {
        let key = key.as_ref().to_owned();
        //let hashmap = &mut self.hashmap.write();
        let hashmap = &mut self.hashmap;
        let res=hashmap.remove(&key);
//        Ok(match hashmap.remove(&key) {
//            Some((_, value)) => Some(value),
//            None => None,
//        })
        //let _f = std::fs::File::create("FIXME")?; // `?` couldn't convert the error to `MyError`:
        //the trait `std::convert::From<std::io::Error>` is not implemented for `MyError`
        match res {
            Some((_,prev_val)) => {
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
