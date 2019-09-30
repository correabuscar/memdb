#![allow(unused_imports)]  //TODO: remove, eventually!

use memdb::Memdb;
//use std::error;
use std::io;
use futures::try_join;
use futures::executor::block_on;
use std::io::{Error, ErrorKind};


#[runtime::test]
async fn set_get() -> io::Result<()> {
    let mut db = Memdb::open().await?;
    db.set("beep", "boop").await?;
    let val = db.get("beep").await?;
    assert_eq!(val, Some("boop".as_bytes().to_owned()));
    Ok(())
}

#[runtime::test]
async fn set_get_del() -> io::Result<()> {
    let mut db = Memdb::open().await?;
    db.set("beep", "boop").await?;
    let val = db.get("beep").await?;
    assert_eq!(val, Some("boop".as_bytes().to_owned()));
    assert_eq!(Some(String::from_utf8(val.unwrap()).unwrap()), Some("boop".to_string()));
    let deleted=db.ensure_del("beep").await?;
    assert_eq!(deleted, Some("boop".as_bytes().to_owned()));
    let val = db.get("beep").await?;
    //println!("{:#?}", val);//None
    assert_eq!(val, None);

    db.set("beep", "boop").await?;
    let deleted=db.del("beep").await?;
    assert_eq!(deleted, "boop".as_bytes().to_owned());
    let deleted=db.del("beep").await;
    assert!(deleted.is_err());
    //println!("{0:#?} {1}",deleted,deleted.clone().err().unwrap());
    //let expected=Error::new(ErrorKind::NotFound,
    //               format!("Attempted to delete inexisting key '{}'", "beep"));
    let expected=memdb::MyError::NotFound { key: "beep".as_bytes().to_owned() };
    //println!("{:#?}",expected);
    assert_eq!(deleted.err().unwrap(), expected);

    Ok(())
}


#[runtime::test]
async fn double_del() -> io::Result<()> {
    let mut db = Memdb::open().await?;
    db.set("beep", "boop").await?;
    let val = db.get("beep").await?;
    assert_eq!(val, Some("boop".as_bytes().to_owned()));
    let deleted=db.ensure_del("beep").await?;
    assert_eq!(deleted, Some("boop".as_bytes().to_owned()));
    let val = db.get("beep").await?;
    //println!("{:#?}", val);//None
    assert_eq!(val, None);
    let deleted=db.ensure_del("beep").await?;
    assert_eq!(deleted, None);
    let val = db.get("beep").await?;
    assert_eq!(val, None);
    Ok(())
}

#[runtime::test]
async fn threaded_set_get() -> io::Result<()> {
    let db = Memdb::open().await?;

    let mut handle = db.clone();
    runtime::spawn(async move {
        handle.set("beep", "boop").await?;
        runtime::spawn(async move {
            let handle = handle.clone();
            let val = handle.get("beep").await?;
            assert_eq!(val, Some("boop".as_bytes().to_owned()));
            Ok::<(), std::io::Error>(()) // https://rust-lang.github.io/async-book/07_workarounds/03_err_in_async_blocks.html
        })
        .await?;
        Ok::<(), std::io::Error>(()) // https://rust-lang.github.io/async-book/07_workarounds/03_err_in_async_blocks.html
    })
    .await?;
    //Ok::<(), std::io::Error>(()) //not needed here.
    return Ok(());
}

#[runtime::test]
async fn threaded_set_get_mod1() -> io::Result<()> {
    let db = Memdb::open().await?;

    let mut handle = db.clone();
    let handle2 = handle.clone();
    let f0=runtime::spawn(async move {
        let f1=handle.set("beep", "boop");//.await?;
        block_on(f1)?;
        let f2=runtime::spawn(async move {
            let val = handle2.get("beep").await?;
            assert_eq!(val, Some("boop".as_bytes().to_owned()));
            Ok::<(), std::io::Error>(())
        });
        //.await?;
        //try_join!(f1,f2)?;
        block_on(f2)?;
        Ok::<(), std::io::Error>(())
    });
    //.await?;
    block_on(f0)?;
    Ok::<(), std::io::Error>(())
}
