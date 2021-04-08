#![feature(debug_non_exhaustive)]
#![feature(extern_types)]
#![feature(try_trait)]
#![feature(static_nobundle)]
#![feature(proc_macro_hygiene)]

use crate::state::DuckDBState;
use libc::c_void;
#[allow(non_camel_case_types)]
pub type c_char = i8;
use crate::db::DB;
use crate::rendering::Table;
use crate::types::{
    duckdb_blob, duckdb_date, duckdb_hugeint, duckdb_interval, duckdb_time, duckdb_timestamp,
};
use render::html;
use std::convert::{TryFrom, TryInto};
use std::ffi::{CStr, CString};
use strum_macros::IntoStaticStr;

mod db;
mod jse;
mod rendering;
mod state;
#[cfg(test)]
mod tests;
mod types;

#[repr(C)]
#[derive(Debug, IntoStaticStr, Clone, Copy)]
pub enum DuckDBType {
    DuckDBTypeInvalid = 0,
    // bool
    DuckDBTypeBoolean = 1,
    // int8_t
    DuckDBTypeTinyint = 2,
    // int16_t
    DuckDBTypeSmallint = 3,
    // int32_t
    DuckDBTypeInteger = 4,
    // int64_t
    DuckDBTypeBigint = 5,
    // float
    DuckDBTypeFloat = 6,
    // double
    DuckDBTypeDouble = 7,
    // duckdb_timestamp
    DuckDBTypeTimestamp = 8,
    // duckdb_date
    DuckDBTypeDate = 9,
    // duckdb_time
    DuckDBTypeTime = 10,
    // duckdb_interval
    DuckDBTypeInterval = 11,
    // duckdb_hugeint
    DuckDBTypeHugeint = 12,
    // const char*
    DuckDBTypeVarchar = 13,
    // duckdb_blob
    DuckDBTypeBlob = 14,
}

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

#[repr(C)]
#[derive(Debug)]
struct DuckDBColumn {
    data: i32,
    nullmask: i32,
    type_: DuckDBType,
    name: *const c_char,
}

type Connection = i32;
type Database = i32;

#[repr(C)]
#[derive(Debug)]
struct DuckDBResult {
    column_count: i64,
    row_count: i64,
    columns: *mut DuckDBColumn,
    error_message: *const c_char,
}

extern "C" {
    fn duckdb_open(path: *const c_char, database: *const Database) -> DuckDBState;

    fn duckdb_connect(db: *const Database, con: *const Connection) -> DuckDBState;

    fn duckdb_disconnect(con: *const Connection);

    fn ext_duckdb_close(db: *const Database);

    fn duckdb_query(
        con: *const Connection,
        query: *const c_char,
        result: *const DuckDBResult,
    ) -> DuckDBState;

    fn duckdb_destroy_result(result: *const DuckDBResult);

    /// Converts the specified value to a bool. Returns false on failure or NULL.
    fn duckdb_value_boolean(result: *const DuckDBResult, col: i64, row: i64) -> bool;
    /// Converts the specified value to an int8_t. Returns 0 on failure or NULL.
    fn duckdb_value_int8(result: *const DuckDBResult, col: i64, row: i64) -> i8;
    /// Converts the specified value to an int16_t. Returns 0 on failure or NULL.
    fn duckdb_value_int16(result: *const DuckDBResult, col: i64, row: i64) -> i16;
    /// Converts the specified value to an int64_t. Returns 0 on failure or NULL.
    fn duckdb_value_int32(result: *const DuckDBResult, col: i64, row: i64) -> i32;
    /// Converts the specified value to an int64_t. Returns 0 on failure or NULL.
    fn duckdb_value_int64(result: *const DuckDBResult, col: i64, row: i64) -> i64;
    /// Converts the specified value to an uint8_t. Returns 0 on failure or NULL.
    fn duckdb_value_uint8(result: *const DuckDBResult, col: i64, row: i64) -> u8;
    /// Converts the specified value to an uint16_t. Returns 0 on failure or NULL.
    fn duckdb_value_uint16(result: *const DuckDBResult, col: i64, row: i64) -> u16;
    /// Converts the specified value to an uint64_t. Returns 0 on failure or NULL.
    fn duckdb_value_uint32(result: *const DuckDBResult, col: i64, row: i64) -> u32;
    /// Converts the specified value to an uint64_t. Returns 0 on failure or NULL.
    fn duckdb_value_uint64(result: *const DuckDBResult, col: i64, row: i64) -> u64;
    /// Converts the specified value to a float. Returns 0.0 on failure or NULL.
    fn duckdb_value_float(result: *const DuckDBResult, col: i64, row: i64) -> f32;
    /// Converts the specified value to a double. Returns 0.0 on failure or NULL.
    fn duckdb_value_double(result: *const DuckDBResult, col: i64, row: i64) -> f64;
    /// Converts the specified value to a string. Returns nullptr on failure or NULL. The result must be freed with free.
    fn duckdb_value_varchar(result: *const DuckDBResult, col: i64, row: i64) -> *const c_char;
    /// Fetches a blob from a result set column. Returns a blob with blob.data set to nullptr on failure or NULL. The
    /// resulting "blob.data" must be freed with free.
    fn duckdb_value_blob(result: *const DuckDBResult, blob: *const duckdb_blob, col: i64, row: i64);

    fn duckdb_value_date(result: *const DuckDBResult, col: i64, row: i64) -> *const duckdb_date;
    fn duckdb_value_time(result: *const DuckDBResult, col: i64, row: i64) -> *const duckdb_time;
    fn duckdb_value_timestamp(
        result: *const DuckDBResult,
        col: i64,
        row: i64,
    ) -> *const duckdb_timestamp;

    fn duckdb_value_hugeint(
        result: *const DuckDBResult,
        col: i64,
        row: i64,
    ) -> *const duckdb_hugeint;
    fn duckdb_value_interval(
        result: *const DuckDBResult,
        col: i64,
        row: i64,
    ) -> *const duckdb_interval;

    fn query(db: *const Database, query: *const c_char, result: *const DuckDBResult)
        -> DuckDBState;

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

    fn column(&self, col: i64) -> &DuckDBColumn {
        &self.columns[<usize as TryFrom<i64>>::try_from(col).expect("Too big")]
    }

    fn consume(&self, col: i64, row: i64) -> Result<DbType, Box<dyn std::error::Error>> {
        use crate::DuckDBType::*;

        let column: &DuckDBColumn = self.column(col);
        let result = self.result;

        Ok(unsafe {
            match &column.type_ {
                DuckDBTypeBoolean => DbType::Boolean(duckdb_value_boolean(result, col, row)),
                DuckDBTypeTinyint => DbType::Tinyint(duckdb_value_int8(result, col, row)),
                DuckDBTypeSmallint => DbType::Smallint(duckdb_value_int16(result, col, row)),
                DuckDBTypeInteger => DbType::Integer(duckdb_value_int32(result, col, row)),
                DuckDBTypeBigint => DbType::Bigint(duckdb_value_int64(result, col, row)),
                DuckDBTypeTime => {
                    DbType::Time(*duckdb_value_time(result, col, row).as_ref().expect("Time"))
                }
                DuckDBTypeTimestamp => DbType::Timestamp(
                    *duckdb_value_timestamp(result, col, row)
                        .as_ref()
                        .expect("Timestamp"),
                ),
                DuckDBTypeDate => {
                    DbType::Date(*duckdb_value_date(result, col, row).as_ref().expect("Date"))
                }
                DuckDBTypeFloat => DbType::Float(duckdb_value_float(result, col, row)),
                DuckDBTypeDouble => DbType::Double(duckdb_value_double(result, col, row)),
                DuckDBTypeVarchar => DbType::String(
                    CStr::from_ptr(duckdb_value_varchar(result, col, row))
                        .to_string_lossy()
                        .to_string(),
                ),
                DuckDBTypeHugeint => DbType::Hugeint(
                    *duckdb_value_hugeint(result, col, row)
                        .as_ref()
                        .expect("Hugeint"),
                ),
                DuckDBTypeBlob => {
                    let ptr: *const duckdb_blob = malloc(PTR);
                    duckdb_value_blob(result, ptr, col, row);
                    DbType::Blob(*ptr.as_ref().expect("Blob"))
                }
                DuckDBTypeInterval => DbType::Interval(
                    *duckdb_value_interval(result, col, row)
                        .as_ref()
                        .expect("Interval"),
                ),
                _ => DbType::Unknown(column.type_),
            }
        })
    }
}

use render::{rsx, SimpleElement};
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::thread_local;

thread_local! {
    static database: RefCell<Option<DB>> = RefCell::new(None);
}

fn form() -> SimpleElement<'static, SimpleElement<'static, ()>> {
    rsx! {
        <form onsubmit={"event.preventDefault(); Module.ccall('callback', 'void', ['string'], [document.forms[0].query.value])"}>
            <input placeholder={"select random()"} autofocus={"true"} name={"query"}></input>
        </form>
    }
}

unsafe fn run_async() -> Result<(), Box<dyn std::error::Error>> {
    set_page_title("DuckDB Test".to_string());

    let db = Some(DB::new(Some("db.db"))?);
    database.with(|f| f.replace(db));

    println!("DB open");

    let string = html! { <>{form()}</> };
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

    database.with(|borrowed| {
        let yo = borrowed.borrow();

        let string = match yo.as_ref().expect("no db?").query(&query) {
            Ok(resolved) => {
                println!("columns: {:?}", resolved.columns);

                let table = Table {
                    resolved: &resolved,
                };
                html! {
                    <div>
                        {form()}
                        {table}
                    </div>
                }
            }
            Err(error) => {
                let e = error.to_string();
                html! {
                    <div>
                        {form()}
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
