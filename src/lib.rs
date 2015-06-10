#![feature(libc)]
extern crate libc;

use std::ffi::{CStr, CString};
use std::fmt;

mod sophia;

#[derive(Debug)]
pub enum Error {
    Defined(String),
    Undefined
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    fn from_ctl(ctl: &Ctl) -> Error {
        let e = ctl.get("sophia.error").unwrap();
        let s = e.get("value").unwrap();
        Error::Defined(String::from_utf8_lossy(s).to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Defined(ref s) => write!(f, "{}", s),
            Error::Undefined => write!(f, "undefined")
        }
    }
}

impl std::error::Error for Error {
     fn description(&self) -> &str {
        ""
     }
}

pub struct Sophia {
    env: *mut libc::c_void,
    ctl: Ctl
}

pub struct Ctl {
    ctl: *mut libc::c_void
}

pub struct Db<'c> {
    db: *mut libc::c_void,
    ctl: &'c Ctl
}

pub struct Object<'c> {
    object: *mut libc::c_void,
    ctl: &'c Ctl
}

pub struct Cursor<'c> {
    cursor: *mut libc::c_void,
    ctl: &'c Ctl

}

pub struct Transaction<'c> {
    transaction: *mut libc::c_void,
    ctl: &'c Ctl
}

pub trait Native {
    fn get_type(&self) -> Result<&str> {
        unsafe {
            let sp_type = sophia::sp_type(self.as_ptr());
            match sp_type as usize {
                0 => Err(Error::from_ctl(self.ctl())),
                _ => {
                    let cstr = CStr::from_ptr(sp_type as *const libc::c_char);
                    Ok(std::str::from_utf8(cstr.to_bytes()).unwrap())
                }
            }
        }
    }

    // TODO should be private
    fn as_ptr(&self) -> *mut libc::c_void;

    fn ctl(&self) -> &Ctl;

    fn destroy(&self) -> Result<()> {
        unsafe {
            match sophia::sp_destroy(self.as_ptr()) as isize {
                0 => Ok(()),
                _ => Err(Error::from_ctl(self.ctl()))
            }
        }
    }
}

impl Native for Sophia {
    fn as_ptr(&self) -> *mut libc::c_void {
        self.env
    }

    fn ctl(&self) -> &Ctl {
        &self.ctl
    }
}

impl Native for Ctl {
    fn as_ptr(&self) -> *mut libc::c_void {
        self.ctl
    }

    fn ctl(&self) -> &Ctl {
        self
    }
}

impl <'c> Native for Db<'c> {
    fn as_ptr(&self) -> *mut libc::c_void {
        self.db
    }

    fn ctl(&self) -> &Ctl {
        &self.ctl
    }
}

impl <'c> Native for Object<'c> {
    fn as_ptr(&self) -> *mut libc::c_void {
        self.object
    }

    fn ctl(&self) -> &Ctl {
        &self.ctl
    }
}

impl <'c> Native for Transaction<'c> {
    fn as_ptr(&self) -> *mut libc::c_void {
        self.transaction
    }

    fn ctl(&self) -> &Ctl {
        &self.ctl
    }
}

impl <'c> Native for Cursor<'c> {
    fn as_ptr(&self) -> *mut libc::c_void {
        self.cursor
    }

    fn ctl(&self) -> &Ctl {
        &self.ctl
    }
}

impl Sophia {
    pub fn new() -> Result<Sophia> {
        unsafe {
            let env = sophia::sp_env();
            match env as usize {
                0 => Err(Error::Undefined),
                _ => {
                    let ctl = sophia::sp_ctl(env);
                    match ctl as usize {
                        0 => Err(Error::Undefined),
                        _ => Ok(Sophia{env: env, ctl: Ctl{ctl: ctl}})
                    }
                }
            }
        }
    }

    /// create or open database
    pub fn open(&self) -> Result<()> {
        unsafe {
            match sophia::sp_open(self.as_ptr()) as isize {
                0 => Ok(()),
                _ => Err(Error::from_ctl(self.ctl()))
            }
        }
    }

    pub fn transaction(&self) -> Result<Transaction> {
        unsafe {
            let transaction = sophia::sp_begin(self.as_ptr());
            match transaction as usize {
                0 => Err(Error::from_ctl(self.ctl())),
                _ => Ok(Transaction{transaction: transaction, ctl: self.ctl()})
            }
        }
    }

    /// check if there are any errors that lead to a shutdown
    pub fn error(&self) -> Option<isize> {
        unsafe {
            match sophia::sp_error(self.as_ptr()) as isize {
                0 => None,
                e => Some(e)
            }
        }
    }
}

impl Ctl {
    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        let key = CString::new(key).unwrap();
        let value = CString::new(value).unwrap();
        unsafe {
            match sophia::sp_set(self.as_ptr(), key.as_ptr(), value.as_ptr()) as isize {
                0 => Ok(()),
                _ => Err(Error::from_ctl(self.ctl()))
            }
        }
    }

    pub fn get(&self, key: &str) -> Result<Object> {
        let key = CString::new(key).unwrap();
        unsafe {
            let object = sophia::sp_get(self.as_ptr(), key.as_ptr());
            match object as usize {
                0 => Err(Error::from_ctl(self.ctl())),
                _ => Ok(Object{object: object, ctl: self.ctl()})
            }
        }
    }

    pub fn get_db(&self, key: &str) -> Result<Db> {
        let key = CString::new(key).unwrap();
        unsafe {
            let db = sophia::sp_get(self.as_ptr(), key.as_ptr());
            match db as usize {
                0 => Err(Error::from_ctl(self.ctl())),
                _ => Ok(Db{db: db, ctl: self.ctl()})
            }
        }
    }
}

impl <'c> Db<'c> {
    pub fn object(&self) -> Result<Object> {
        unsafe {
            let object = sophia::sp_object(self.db);
            match object as usize {
                0 => Err(Error::from_ctl(self.ctl())),
                _ => Ok(Object{object: object, ctl: self.ctl()})
            }
        }
    }

    pub fn set(&self, object: &Object) -> Result<()> {
        unsafe {
            match sophia::sp_set(self.as_ptr(), object.object) as isize {
                0 => Ok(()),
                _ => Err(Error::from_ctl(self.ctl()))
            }
        }
    }

    pub fn get(&self, object: &Object) -> Result<Object> {
        unsafe {
            let object = sophia::sp_get(self.as_ptr(), object.object);
            match object as usize {
                0 => Err(Error::from_ctl(self.ctl())),
                _ => Ok(Object{object: object, ctl: self.ctl()})
            }
        }
    }

    pub fn delete(&self, object: &Object) -> Result<()> {
        unsafe {
            match sophia::sp_delete(self.as_ptr(), object.object) as isize {
                0 => Ok(()),
                _ => Err(Error::from_ctl(self.ctl()))
            }
        }
    }

    pub fn cursor(&self, object: &Object) -> Result<Cursor> {
        unsafe {
            let cursor = sophia::sp_cursor(self.as_ptr(), object.object);
            match cursor as usize {
                0 => Err(Error::from_ctl(self.ctl())),
                _ => Ok(Cursor{cursor: cursor, ctl: self.ctl()})
            }
        }
    }
}

impl <'c> Object<'c> {
    pub fn set(&self, field: &str, value: &[u8]) -> Result<()> {
        let field = CString::new(field).unwrap();
        unsafe {
            match sophia::sp_set(self.as_ptr(), field.as_ptr(), value, value.len()) as isize {
                0 => Ok(()),
                _ => Err(Error::from_ctl(self.ctl()))
            }
        }
    }

    pub fn get(&self, field: &str) -> Result<&[u8]> {
        let field = CString::new(field).unwrap();
        unsafe {
            let len = 0;
            let value = sophia::sp_get(self.as_ptr(), field.as_ptr(), &len);
            match value as usize {
                0 => Err(Error::from_ctl(self.ctl())),
                _ => {
                    Ok(std::slice::from_raw_parts(value as *const u8, len))
                }
            }
        }
    }
}

impl <'c> Cursor<'c> {
    pub fn get(&self) -> Result<Object> {
        unsafe {
            let object = sophia::sp_get(self.as_ptr());
            match object as usize {
                0 => Err(Error::from_ctl(self.ctl())),
                _ => Ok(Object{object: object, ctl: self.ctl()})
            }
        }
    }
}

// TODO set/get/delete are the same as for Db, make a trait?
impl <'c> Transaction<'c> {
    pub fn set(&self, object: &Object) -> Result<()> {
        unsafe {
            match sophia::sp_set(self.as_ptr(), object.as_ptr()) as isize {
                0 => Ok(()),
                _ => Err(Error::from_ctl(self.ctl()))
            }
        }
    }

    fn get(&self, object: &Object) -> Result<Object> {
        unsafe {
            let object = sophia::sp_get(self.as_ptr(), object.as_ptr());
            match object as usize {
                0 => Err(Error::from_ctl(self.ctl())),
                _ => Ok(Object{object: object, ctl: self.ctl()})
            }
        }
    }

    pub fn delete(&self, object: &Object) -> Result<()> {
        unsafe {
            match sophia::sp_delete(self.as_ptr(), object.as_ptr()) as isize {
                0 => Ok(()),
                _ => Err(Error::from_ctl(self.ctl()))
            }
        }
    }

    pub fn commit(&self) -> Result<isize> {
        unsafe {
            match sophia::sp_commit(self.as_ptr()) as isize {
                -1 => Err(Error::from_ctl(self.ctl())),
                s => Ok(s)
            }
        }
    }
}

impl Drop for Sophia {
    fn drop(&mut self) {
        let res = self.destroy();
        println!("env destroy {:?}", res.unwrap());
    }
}

impl <'c> Drop for Object<'c> {
    fn drop(&mut self) {
        let res = self.destroy();
        println!("object destroy {:?}", res.unwrap());
    }
}

impl <'c> Drop for Cursor<'c> {
    fn drop(&mut self) {
        let res = self.destroy();
        println!("cursor destroy {:?}", res.unwrap());
    }
}
