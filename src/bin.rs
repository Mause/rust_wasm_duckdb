#![feature(extern_types)]
#![feature(try_trait)]
#![feature(static_nobundle)]

use crate::state::DuckDBState;
// use libc::c_char;
pub type c_char = i8;
use std::alloc::{alloc, Layout};
use std::convert::TryInto;
use std::ffi::CString;

mod state;

#[repr(C)]
#[derive(Debug)]
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

    fn duckdb_close(db: *const Database);

    fn duckdb_query(
        con: *const Connection,
        query: *const c_char,
        result: *const DuckDBResult,
    ) -> DuckDBState;

    fn duckdb_destroy_result(result: *const DuckDBResult);

    fn duckdb_value_varchar(result: *const DuckDBResult, row: i64, column: i64) -> *const c_char;

    fn query(query: *const c_char) -> *mut DuckDBResult;
}

fn malloc<T: Sized>(size: usize) -> *const T {
    unsafe { alloc(Layout::from_size_align(size, 8).expect("FUck")) as *const T }
}

macro_rules! console_log {
    ($($t:tt)*) => (stdweb::console!(log, (&format_args!($($t)*).to_string())))
}

static PTR: usize = core::mem::size_of::<i32>();

unsafe fn run_async() -> Result<(), Box<dyn std::error::Error>> {
    let s = CString::new("SELECT 1;").expect("string");
    let resolved: &DuckDBResult = &*query(s.as_ptr());
    println!("{:?}", resolved);

    let database = malloc(PTR);
    duckdb_open(std::ptr::null(), database)?;

    let connection: *const Connection = malloc(PTR);
    duckdb_connect(database, connection)?;
    println!("{:?} {:?}", database, connection);

    println!("building string");
    let query = CString::new("select 1")?;
    println!("query: {:?}", query);
    let result = malloc(PTR);
    duckdb_query(connection, query.as_ptr(), result)?;

    let rl_res = result.as_ref().expect("res");

    let length = rl_res.column_count.try_into().unwrap();
    let columns: Vec<DuckDBColumn> = Vec::from_raw_parts(rl_res.columns, length, length);

    println!("{:?}", columns);

    for row_idx in 0..rl_res.row_count {
        for col_idx in 0..rl_res.column_count {
            let rval = duckdb_value_varchar(result, col_idx, row_idx);
            console_log!("val: {:?}", rval);
            // _emscripten_builtin_free(rval);
        }
        console_log!("\n");
    }
    duckdb_destroy_result(result);

    duckdb_disconnect(connection);

    duckdb_close(database);

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
    let error = js_sys::Error::new("test1");
    println!("{:?}", error);
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

/// Dummy __gxx_personality_v0 hook
extern "C" fn ___gxx_personality_v0() {}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::panic::set_hook(Box::new(hook));

    unsafe {
        run_async().expect("Ooops");
    }

    Ok(())
}

#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}
