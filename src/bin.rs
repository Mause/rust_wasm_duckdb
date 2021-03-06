#![feature(debug_non_exhaustive)]
#![feature(extern_types)]
#![feature(try_trait)]
#![feature(static_nobundle)]
#![feature(proc_macro_hygiene)]
#![allow(unused_parens)]
#![allow(unused_braces)]

use crate::state::DuckDBState;
use libc::c_void;
#[allow(non_camel_case_types)]
pub type c_char = i8;
use crate::db::DB;
use crate::rendering::{Form, Table};
use crate::types::{
    duckdb_blob, duckdb_connection, duckdb_database, duckdb_date, duckdb_hugeint, duckdb_interval,
    duckdb_time, duckdb_timestamp, duckdb_type as DuckDBType, DuckDBColumn, DuckDBResult,
};
use render::html;
use std::cell::RefCell;
use std::convert::{TryFrom, TryInto};
use std::ffi::{CStr, CString};
use std::thread_local;
use strum_macros::IntoStaticStr;

mod bindings;
mod db;
mod jse;
mod rendering;
mod state;
#[cfg(test)]
mod tests;
mod types;

#[derive(Debug, IntoStaticStr)]
enum DbType {
    Boolean(bool),
    Tinyint(i8),
    Smallint(i16),
    Integer(i32),
    Bigint(i64),
    Float(f32),
    Date(duckdb_date),
    Time(duckdb_time),
    Timestamp(duckdb_timestamp),
    Double(f64),
    String(String),
    Interval(duckdb_interval),
    Hugeint(duckdb_hugeint),
    Blob(duckdb_blob),
    Unknown(DuckDBType),
}
impl ToString for DbType {
    fn to_string(&self) -> String {
        use crate::DbType::*;

        let value: &dyn ToString = match self {
            Boolean(s) => s,
            Tinyint(s) => s,
            Smallint(s) => s,
            Integer(i) => i,
            Bigint(s) => s,
            Float(f) => f,
            Double(f) => f,
            String(s) => s,
            Time(s) => s,
            Timestamp(s) => s,
            Date(s) => s,
            Blob(s) => s,
            Hugeint(s) => s,
            Interval(s) => s,
            Unknown(_) => &"unknown",
        };

        value.to_string()
    }
}

extern "C" {
    fn duckdb_open(path: *const c_char, database: *const duckdb_database) -> DuckDBState;

    fn duckdb_connect(db: *const duckdb_database, con: *const duckdb_connection) -> DuckDBState;

    fn duckdb_disconnect(con: *const duckdb_connection);

    fn ext_duckdb_close(db: *const duckdb_database);

    fn duckdb_query(
        con: *const duckdb_connection,
        query: *const c_char,
        result: *const DuckDBResult,
    ) -> DuckDBState;

    fn duckdb_destroy_result(result: *const DuckDBResult);

    /// Converts the specified value to a bool. Returns false on failure or NULL.
    fn duckdb_value_boolean(result: *const DuckDBResult, col: u64, row: u64) -> bool;
    /// Converts the specified value to an int8_t. Returns 0 on failure or NULL.
    fn duckdb_value_int8(result: *const DuckDBResult, col: u64, row: u64) -> i8;
    /// Converts the specified value to an int16_t. Returns 0 on failure or NULL.
    fn duckdb_value_int16(result: *const DuckDBResult, col: u64, row: u64) -> i16;
    /// Converts the specified value to an int64_t. Returns 0 on failure or NULL.
    fn duckdb_value_int32(result: *const DuckDBResult, col: u64, row: u64) -> i32;
    /// Converts the specified value to an int64_t. Returns 0 on failure or NULL.
    fn duckdb_value_int64(result: *const DuckDBResult, col: u64, row: u64) -> i64;
    /// Converts the specified value to an uint8_t. Returns 0 on failure or NULL.
    fn duckdb_value_uint8(result: *const DuckDBResult, col: u64, row: u64) -> u8;
    /// Converts the specified value to an uint16_t. Returns 0 on failure or NULL.
    fn duckdb_value_uint16(result: *const DuckDBResult, col: u64, row: u64) -> u16;
    /// Converts the specified value to an uint64_t. Returns 0 on failure or NULL.
    fn duckdb_value_uint32(result: *const DuckDBResult, col: u64, row: u64) -> u32;
    /// Converts the specified value to an uint64_t. Returns 0 on failure or NULL.
    fn duckdb_value_uint64(result: *const DuckDBResult, col: u64, row: u64) -> u64;
    /// Converts the specified value to a float. Returns 0.0 on failure or NULL.
    fn duckdb_value_float(result: *const DuckDBResult, col: u64, row: u64) -> f32;
    /// Converts the specified value to a double. Returns 0.0 on failure or NULL.
    fn duckdb_value_double(result: *const DuckDBResult, col: u64, row: u64) -> f64;
    /// Converts the specified value to a string. Returns nullptr on failure or NULL. The result must be freed with free.
    fn duckdb_value_varchar(result: *const DuckDBResult, col: u64, row: u64) -> *const c_char;
    /// Fetches a blob from a result set column. Returns a blob with blob.data set to nullptr on failure or NULL. The
    /// resulting "blob.data" must be freed with free.
    fn duckdb_value_blob(result: *const DuckDBResult, blob: *const duckdb_blob, col: u64, row: u64);

    fn duckdb_value_date(result: *const DuckDBResult, col: u64, row: u64) -> *const duckdb_date;
    fn duckdb_value_time(result: *const DuckDBResult, col: u64, row: u64) -> *const duckdb_time;
    fn duckdb_value_timestamp(
        result: *const DuckDBResult,
        col: u64,
        row: u64,
    ) -> *const duckdb_timestamp;

    fn duckdb_value_hugeint(
        result: *const DuckDBResult,
        col: u64,
        row: u64,
    ) -> *const duckdb_hugeint;
    fn duckdb_value_interval(
        result: *const DuckDBResult,
        col: u64,
        row: u64,
    ) -> *const duckdb_interval;

    pub fn emscripten_asm_const_int(
        code: *const u8,
        sigPtr: *const u8,
        argBuf: *const u8,
    ) -> *mut u8;

    pub fn mallocy() -> *const c_void;
}

fn malloc<T: Sized>(_size: usize) -> *const T {
    unsafe { mallocy() as *const T }
}

static PTR: usize = core::mem::size_of::<i32>();

fn set_body_html(string: String) -> i32 {
    let cstring = CString::new(string).expect("string");
    let input = cstring.as_ptr() as *const _ as i32;

    jse!(
        b"document.body.innerHTML = UTF8ToString($0, 1000);\x00",
        input
    )
}

fn set_page_title(string: String) -> i32 {
    let cstring = CString::new(string).expect("string");
    let input = cstring.as_ptr() as *const _ as i32;

    jse!(b"document.title = UTF8ToString($0, 1000);\x00", input)
}

#[derive(Debug)]
pub struct ResolvedResult<'a> {
    result: *const DuckDBResult,
    resolved: &'a DuckDBResult,
    columns: Vec<DuckDBColumn>,
    length: usize,
}
impl<'a> Clone for ResolvedResult<'a> {
    fn clone(&self) -> ResolvedResult<'a> {
        unsafe { ResolvedResult::new(self.result) }
    }
}
impl<'a> Drop for ResolvedResult<'a> {
    fn drop(&mut self) {
        println!("Dropping {:?}", self);
        unsafe { duckdb_destroy_result(self.result) };
    }
}
impl<'a> ResolvedResult<'a> {
    unsafe fn new(result: *const DuckDBResult) -> Self {
        let resolved = &*result;

        let length = resolved.column_count.try_into().expect("Too many columns");
        let columns: Vec<DuckDBColumn> = Vec::from_raw_parts(resolved.columns, length, length);

        Self {
            result,
            resolved,
            columns,
            length,
        }
    }

    fn column(&self, col: u64) -> &DuckDBColumn {
        &self.columns[<usize as TryFrom<u64>>::try_from(col).expect("Too big")]
    }

    fn consume(&self, col: u64, row: u64) -> Result<DbType, Box<dyn std::error::Error>> {
        let column: &DuckDBColumn = self.column(col);
        let result = self.result;

        Ok(unsafe {
            match &column.type_ {
                DuckDBType::DUCKDB_TYPE_BOOLEAN => {
                    DbType::Boolean(duckdb_value_boolean(result, col, row))
                }
                DuckDBType::DUCKDB_TYPE_TINYINT => {
                    DbType::Tinyint(duckdb_value_int8(result, col, row))
                }
                DuckDBType::DUCKDB_TYPE_SMALLINT => {
                    DbType::Smallint(duckdb_value_int16(result, col, row))
                }
                DuckDBType::DUCKDB_TYPE_INTEGER => {
                    DbType::Integer(duckdb_value_int32(result, col, row))
                }
                DuckDBType::DUCKDB_TYPE_BIGINT => {
                    DbType::Bigint(duckdb_value_int64(result, col, row))
                }
                DuckDBType::DUCKDB_TYPE_TIME => {
                    DbType::Time(*duckdb_value_time(result, col, row).as_ref().expect("Time"))
                }
                DuckDBType::DUCKDB_TYPE_TIMESTAMP => DbType::Timestamp(
                    *duckdb_value_timestamp(result, col, row)
                        .as_ref()
                        .expect("Timestamp"),
                ),
                DuckDBType::DUCKDB_TYPE_DATE => {
                    DbType::Date(*duckdb_value_date(result, col, row).as_ref().expect("Date"))
                }
                DuckDBType::DUCKDB_TYPE_FLOAT => {
                    DbType::Float(duckdb_value_float(result, col, row))
                }
                DuckDBType::DUCKDB_TYPE_DOUBLE => {
                    DbType::Double(duckdb_value_double(result, col, row))
                }
                DuckDBType::DUCKDB_TYPE_VARCHAR => DbType::String(
                    CStr::from_ptr(duckdb_value_varchar(result, col, row))
                        .to_string_lossy()
                        .to_string(),
                ),
                DuckDBType::DUCKDB_TYPE_HUGEINT => DbType::Hugeint(
                    *duckdb_value_hugeint(result, col, row)
                        .as_ref()
                        .expect("Hugeint"),
                ),
                DuckDBType::DUCKDB_TYPE_BLOB => {
                    let ptr: *const duckdb_blob = malloc(PTR);
                    duckdb_value_blob(result, ptr, col, row);
                    DbType::Blob(ptr.as_ref().expect("Blob").clone())
                }
                DuckDBType::DUCKDB_TYPE_INTERVAL => DbType::Interval(
                    *duckdb_value_interval(result, col, row)
                        .as_ref()
                        .expect("Interval"),
                ),
                _ => DbType::Unknown(column.type_),
            }
        })
    }
}

thread_local! {
    static DATABASE: RefCell<Option<DB>> = RefCell::new(None);
}

unsafe fn run_async() -> Result<(), Box<dyn std::error::Error>> {
    set_page_title("DuckDB Test".to_string());

    let db = Some(DB::new(Some("db.db"))?);
    println!("DB: {:?}", db);
    DATABASE.with(|f| f.replace(db));

    println!("DB open");

    let string = html! { <>{Form {}}</> };
    set_body_html(string);

    Ok(())
}

fn hook(info: &std::panic::PanicInfo) {
    let mut msg = info.to_string();

    println!("{:?}", msg);

    // Add the error stack to our message.
    //
    // This ensures that even if the `console` implementation doesn't
    // include stacks for `console.error`, the stack is still available
    // for the user. Additionally, Firefox's console tries to clean up
    // stack traces, and ruins Rust symbols in the process
    // (https://bugzilla.mozilla.org/show_bug.cgi?id=1519569) but since
    // it only touches the logged message's associated stack, and not
    // the message's contents, by including the stack in the message
    // contents we make sure it is available to the user.
    msg.push_str("\n\nStack:\n\n");
    // #[cfg(not(test))]
    // {
    //     let error = js_sys::Error::new("test1");
    //     println!("{:?}", error);
    // }
    // let stack = error.stack();
    // println!("{:?}", stack);
    // msg.push_str(stack.as_str().unwrap_or_default());

    // Safari's devtools, on the other hand, _do_ mess with logged
    // messages' contents, so we attempt to break their heuristics for
    // doing that by appending some whitespace.
    // https://github.com/rustwasm/console_error_panic_hook/issues/7
    msg.push_str("\n\n");

    // Finally, log the panic with `console.error`!
    println!("{}", msg);
}

#[no_mangle]
extern "C" fn callback(query_: *const c_char) {
    let org = unsafe { CStr::from_ptr(query_) };
    let query = org.to_string_lossy();

    println!("you called?: {} {:?} {:?}", query, org, query_);

    DATABASE.with(|borrowed| {
        println!("borrowed: {:?}", borrowed);
        let yo = borrowed.borrow();
        println!("yo: {:?}", yo);

        let conn = yo.as_ref().expect("no db?").connection().unwrap();

        let string = match conn.query(&query) {
            Ok(resolved) => {
                println!("columns: {:?}", resolved.columns);

                let table = Table {
                    resolved: &resolved,
                };
                html! {
                    <div>
                        {Form {}}
                        {table}
                    </div>
                }
            }
            Err(error) => {
                let e = error.to_string();
                html! {
                    <div>
                        {Form {}}
                        <pre><code>{e}</code></pre>
                    </div>
                }
            }
        };

        println!("{}", string);

        set_body_html(string);
    });
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::panic::set_hook(Box::new(hook));

    unsafe {
        run_async().expect("Ooops");
    }

    Ok(())
}
