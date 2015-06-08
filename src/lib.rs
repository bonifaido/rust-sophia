#![feature(libc)]
extern crate libc;

use std::ffi::{CStr, CString};

mod sophia;

trait Native {
    fn get_type(&self) -> Result<&str, usize> {
        unsafe {
            let sp_type = sophia::sp_type(self.ptr());
            match sp_type as usize {
                0 => Err(0),
                _ => {
                    let cstr = CStr::from_ptr(sp_type as *const libc::c_char);
                    Ok(std::str::from_utf8(cstr.to_bytes()).unwrap())
                }
            }
        }
    }

    fn ptr(&self) -> *mut libc::c_void;
}

impl Native for Object {
    fn ptr(&self) -> *mut libc::c_void {
        self.object
    }    
}

impl Native for Sophia {
    fn ptr(&self) -> *mut libc::c_void {
        self.env
    }    
}

impl Native for Ctl {
    fn ptr(&self) -> *mut libc::c_void {
        self.ctl
    }    
}

// Env
pub struct Sophia {
    env: *mut libc::c_void,
    ctl: Ctl
}

pub struct Ctl {
    ctl: *mut libc::c_void
}

pub struct Db {
    db: *mut libc::c_void
}

pub struct Object {
    object: *mut libc::c_void
}

pub struct Cursor {
    cursor: *mut libc::c_void
}

pub struct Transaction {
    transaction: *mut libc::c_void
}

impl Ctl {
    pub fn set(&self, key: &str, value: &str) -> Result<(), isize> {
        let key = CString::new(key).unwrap();
        let value = CString::new(value).unwrap();
        unsafe {
            match sophia::sp_set(self.ctl, key.as_ptr(), value.as_ptr()) as isize {
                0 => Ok(()),
                e => Err(e)
            }
        }
    }

    pub fn get(&self, key: &str) -> Result<Object, usize> {
        let key = CString::new(key).unwrap();
        unsafe {
            let object = sophia::sp_get(self.ctl, key.as_ptr());
            match object as usize {
                0 => Err(0),
                _ => Ok(Object{object: object})
            }
        }
    }

    pub fn get_db(&self, key: &str) -> Result<Db, usize> {
        let key = CString::new(key).unwrap();
        unsafe {
            let db = sophia::sp_get(self.ctl, key.as_ptr());
            match db as usize {
                0 => Err(0),
                _ => Ok(Db{db: db})
            }
        }
    }
}

impl Db {
    fn object(&self) -> Result<Object, usize> {
        unsafe {
            let object = sophia::sp_object(self.db);
            match object as usize {
                0 => Err(0),
                _ => Ok(Object{object: object})
            }
        }
    }

    fn set(&self, object: &Object) -> Result<(), isize> {
        unsafe {
            match sophia::sp_set(self.db, object.object) as isize {
                0 => Ok(()),
                e => Err(e)
            }
        }
    }

    fn get(&self, object: &Object) -> Result<Object, usize> {
        unsafe {
            let object = sophia::sp_get(self.db, object.object);
            match object as usize {
                0 => Err(0),
                _ => Ok(Object{object: object})
            }
        }
    }

    fn delete(&self, object: &Object) -> Result<(), isize> {
        unsafe {
            match sophia::sp_delete(self.db, object.object) as isize {
                0 => Ok(()),
                e => Err(e)
            }
        }
    }

    fn cursor(&self, object: &Object) -> Result<Cursor, usize> {
        unsafe {
            let cursor = sophia::sp_cursor(self.db, object.object);
            match cursor as usize {
                0 => Err(0),
                _ => Ok(Cursor{cursor: cursor})
            }
        }
    }
}

impl Object {
    pub fn set(&self, field: &str, value: &[u8]) -> Result<(), isize> {
        let field = CString::new(field).unwrap();
        unsafe {
            match sophia::sp_set(self.object, field.as_ptr(), value, value.len()) as isize {
                0 => Ok(()),
                e => Err(e)
            }
        }
    }

    pub fn get(&self, field: &str) -> Result<&[u8], usize> {
        let field = CString::new(field).unwrap();
        unsafe {
            let len = 0;
            let value = sophia::sp_get(self.object, field.as_ptr(), &len);
            match value as usize {
                0 => Err(0),
                _ => {
                    Ok(std::slice::from_raw_parts(value as *const u8, len))
                }
            }
        }
    }
}

impl Cursor {
    pub fn get(&self) -> Result<Object, usize> {
        unsafe {
            let object = sophia::sp_get(self.cursor);
            match object as usize {
                0 => Err(0),
                _ => Ok(Object{object: object})
            }
        }
    }
}

impl Drop for Sophia {
    fn drop(&mut self) {
        let res = Sophia::destroy(self.env);
        println!("env destroy {:?}", res.unwrap());
    }
}

impl Drop for Object {
    fn drop(&mut self) {
        let res = Sophia::destroy(self.object);
        println!("object destroy {:?}", res.unwrap());
    }
}

impl Drop for Cursor {
    fn drop(&mut self) {
        let res = Sophia::destroy(self.cursor);
        println!("cursor destroy {:?}", res.unwrap());
    }
}

// TODO set/get/delete are the same as for Db, make a trait?
impl Transaction {
    pub fn set(&self, object: &Object) -> Result<(), isize> {
        unsafe {
            match sophia::sp_set(self.transaction, object.object) as isize {
                0 => Ok(()),
                e => Err(e)
            }
        }
    }

    fn get(&self, object: &Object) -> Result<Object, usize> {
        unsafe {
            let object = sophia::sp_get(self.transaction, object.object);
            match object as usize {
                0 => Err(0),
                _ => Ok(Object{object: object})
            }
        }
    }

    pub fn delete(&self, object: &Object) -> Result<(), isize> {
        unsafe {
            match sophia::sp_delete(self.transaction, object.object) as isize {
                0 => Ok(()),
                e => Err(e)
            }
        }
    }

    pub fn commit(&self) -> Result<isize, isize> {
        unsafe {
            match sophia::sp_commit(self.transaction) as isize {
                -1 => Err(-1),
                s => Ok(s)
            }
        }
    }

    pub fn destroy(&self) -> Result<(), isize> {
        Sophia::destroy(self.transaction)
    }
}

impl Sophia {
    pub fn new() -> Result<Sophia, usize> {
        unsafe {
            let env = sophia::sp_env();
            match env as usize {
                0 => Err(0),
                _ => {
                    let ctl = sophia::sp_ctl(env);
                    match ctl as usize {
                        0 => Err(0),
                        _ => Ok(Sophia{env: env, ctl: Ctl{ctl: ctl}})
                    }
                }
            }
        }
    }

    pub fn ctl(&self) -> &Ctl {
        &self.ctl
    }

    /// create or open database
    pub fn open(&self) -> Result<(), isize> {
        unsafe {
            match sophia::sp_open(self.env) as isize {
                0 => Ok(()),
                _ => Err(-1)
            }
        }
    }

    pub fn transaction(&self) -> Result<Transaction, usize> {
        unsafe {
            let transaction = sophia::sp_begin(self.env);
            match transaction as usize {
                0 => Err(0),
                _ => Ok(Transaction{transaction: transaction})
            }
        }
    }    

    /// check if there any error leads to the shutdown
    pub fn error(&self) -> Option<isize> {
        unsafe {
            match sophia::sp_error(self.env) as isize {
                0 => None,
                e => Some(e)
            }
        }
    }

    fn destroy(object: *mut libc::c_void) -> Result<(), isize> {
        unsafe {
            match sophia::sp_destroy(object) as isize {
                0 => Ok(()),
                _ => Err(-1)
            }
        }
    }
}

//#[test]
pub fn it_works() {

    println!("## Setup ##");

    let env = Sophia::new().unwrap();
    println!("env {:?} type {}", env.env, env.get_type().unwrap());

    let ctl = env.ctl();

    let res = ctl.set("sophia.path", "./test.db");
    println!("ctl.set {:?}", res.unwrap());
    let res = ctl.set("db", "test");
    println!("ctl.set {:?}", res.unwrap());

    let res = env.open();
    println!("env.open {:?}", res.unwrap());

    let db = ctl.get_db("db.test").unwrap();
    println!("ctl.get_db {:?}", db.db);

    println!("## Set ##");
    {
        let object = db.object().unwrap();
        println!("object {:?} type {}", object.object, object.get_type().unwrap());

        let res = object.set("key", "hello".as_bytes());
        println!("object.set.key {:?}", res.unwrap());

        let res = object.set("value", "world".as_bytes());
        println!("object.set.value {:?}", res.unwrap());

        let res = db.set(&object);
        println!("db.set.object {:?}", res.unwrap());
    }

    println!("## Get ##");
    {
        let object = db.object().unwrap();
        println!("object {:?}", object.object);

        let res = object.set("key", "hello".as_bytes());
        println!("object.set.key {:?}", res.unwrap());

        let object = db.get(&object).unwrap();
        println!("db.get.object {:?}", object.object);

        let field = "value";
        let value = object.get(field).ok().expect("value can't be read");
        println!("object.get.{} {:?}", field, std::str::from_utf8(value).unwrap());
    }

    println!("## Transaction ##");
    {
        let object = db.object().unwrap();
        println!("object {:?}", object.object);

        let res = object.set("key", "inside".as_bytes());
        println!("object.set.key {:?}", res.unwrap());

        let res = object.set("value", "transaction".as_bytes());
        println!("object.set.value {:?}", res.unwrap());

        let transaction = env.transaction().unwrap();

        let res = transaction.set(&object);
        println!("transaction.set.object {:?}", res.unwrap());

        let res = transaction.commit();
        println!("transaction.commit {:?}", res.unwrap());
    }

    println!("## Cursor ##");
    {
        let options = db.object().unwrap();
        println!("object {:?}", options.object);

        let cursor = db.cursor(&options).unwrap();
        println!("db.cursor {:?}", cursor.cursor);

        loop {
            let object = cursor.get();
            match object {
                Err(_) => break,
                Ok(object) => {
                    let key = object.get("key").unwrap();
                    let value = object.get("value").unwrap();
                    println!("cursor.object.key = {:?}", std::str::from_utf8(key).unwrap());
                    println!("cursor.object.value = {:?}", std::str::from_utf8(value).unwrap());
                }
            }
        }
    }

    println!("## Delete ##");
    {
        let object = db.object().unwrap();
        println!("object {:?}", object.object);

        let res = object.set("key", "hello".as_bytes());
        println!("object.set.key {:?}", res.unwrap());

        let res = db.delete(&object).unwrap();
        println!("db.delete.object {:?}", res);
    }
}
