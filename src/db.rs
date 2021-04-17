use crate::types::{duckdb_connection, duckdb_database};
use crate::{
    duckdb_disconnect, duckdb_open, duckdb_query, ext_duckdb_close, malloc, DuckDBState,
    ResolvedResult, PTR,
};
use std::ffi::{CStr, CString};

extern "C" {
    fn create_connection(db: *const duckdb_database) -> *const duckdb_connection;
}

#[derive(Debug)]
pub struct DB {
    db: *const duckdb_database,
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

    pub fn connection(&self) -> Result<Connection, Box<dyn std::error::Error>> {
        let connection: *const duckdb_connection = unsafe { create_connection(self.db) };
        println!("conn: {:?}", &connection);
        Ok(Connection { connection })
    }
}
impl Drop for DB {
    fn drop(&mut self) {
        println!("Dropping {:?}", self);
        unsafe { ext_duckdb_close(self.db) };
    }
}

#[derive(Debug)]
pub struct Connection {
    connection: *const duckdb_connection,
}
impl Connection {
    pub fn query(&self, que: &str) -> Result<ResolvedResult, Box<dyn std::error::Error>> {
        unsafe {
            let s = CString::new(que).expect("string");

            let result = malloc(PTR);
            let status = duckdb_query(self.connection, s.as_ptr(), result);

            if matches!(status, DuckDBState::DuckDBError) {
                let error_message = CStr::from_ptr((*result).error_message).to_string_lossy();

                Err(string_error::new_err(&*error_message))
            } else {
                Ok(ResolvedResult::new(result))
            }
        }
    }
}
impl Drop for Connection {
    fn drop(&mut self) {
        println!("Dropping {:?}", self);
        unsafe { duckdb_disconnect(self.connection) };
    }
}
