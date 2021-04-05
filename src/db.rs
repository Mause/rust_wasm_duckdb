use crate::{
    duckdb_open, ext_duckdb_close, malloc, query, Database, DuckDBState, ResolvedResult, PTR,
};
use std::ffi::{CStr, CString};

pub struct DB {
    db: *const Database,
}
impl DB {
    pub fn new(path: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        let db = malloc(PTR);

        unsafe {
            match path {
                Some(path) => {
                    let path = CString::new(path).unwrap();
                    duckdb_open(path.as_ptr(), db)?
                }
                None => duckdb_open(std::ptr::null(), db)?,
            }
        };

        Ok(Self { db })
    }
}
impl DB {
    pub fn query(&self, que: &str) -> Result<ResolvedResult, Box<dyn std::error::Error>> {
        unsafe {
            let s = CString::new(que).expect("string");

            let result = malloc(PTR);
            let status = query(self.db, s.as_ptr(), result);

            if matches!(status, DuckDBState::DuckDBError) {
                let error_message = CStr::from_ptr((*result).error_message).to_string_lossy();

                Err(string_error::new_err(&*error_message))
            } else {
                Ok(ResolvedResult::new(result))
            }
        }
    }
}
impl Drop for DB {
    fn drop(&mut self) {
        unsafe { ext_duckdb_close(self.db) };
    }
}
