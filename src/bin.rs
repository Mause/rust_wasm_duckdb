#![feature(extern_types)]
#![feature(try_trait)]
#![feature(static_nobundle)]

use crate::state::DuckDBState;
use libc::c_void;
pub type c_char = i8;
use std::alloc::{alloc, Layout};
use std::convert::{TryFrom, TryInto};
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

#[repr(C)]
#[derive(Debug)]
struct DuckDBBlob {
    data: *const c_void,
    size: i64,
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
    // fn duckdb_value_float(result: *const DuckDBResult, col: i64, row: i64) -> float;
    /// Converts the specified value to a double. Returns 0.0 on failure or NULL.
    // fn duckdb_value_double(result: *const DuckDBResult, col: i64, row: i64) -> double;
    /// Converts the specified value to a string. Returns nullptr on failure or NULL. The result must be freed with free.
    fn duckdb_value_varchar(result: *const DuckDBResult, col: i64, row: i64) -> *const c_char;
    /// Fetches a blob from a result set column. Returns a blob with blob.data set to nullptr on failure or NULL. The
    /// resulting "blob.data" must be freed with free.
    fn duckdb_value_blob(result: *const DuckDBResult, col: i64, row: i64) -> *const DuckDBBlob;

    fn query(db: *const Database, query: *const c_char) -> *mut DuckDBResult;
}

fn malloc<T: Sized>(size: usize) -> *const T {
    unsafe { alloc(Layout::from_size_align(size, 8).expect("FUck")) as *const T }
}

static PTR: usize = core::mem::size_of::<i32>();

extern "C" {
    pub fn emscripten_asm_const_int(
        code: *const u8,
        sigPtr: *const u8,
        argBuf: *const u8,
    ) -> *mut u8;
}

fn call(input: i32) -> i32 {
    const SNIPPET: &'static [u8] =
        b"let i = arguments[0]; document.body.innerText = UTF8ToString(i, 1000); return i;\x00";

    let sig = "i\x00";

    unsafe {
        emscripten_asm_const_int(
            SNIPPET as *const _ as *const u8,
            sig as *const _ as *const u8,
            &[input] as *const _ as *const u8,
        ) as i32
    }
}

unsafe fn run_async() -> Result<(), Box<dyn std::error::Error>> {
    let database = malloc(PTR);
    duckdb_open(std::ptr::null(), database)?;

    println!("DB open");

    let s = CString::new("SELECT 42").expect("string");
    let resolved: &DuckDBResult = &*query(database, s.as_ptr());
    println!("result: {:?}", resolved);

    let length = resolved.column_count.try_into().unwrap();
    let columns: Vec<DuckDBColumn> = Vec::from_raw_parts(resolved.columns, length, length);

    println!("columns: {:?}", columns);

    for row_idx in 0..resolved.row_count {
        for col_idx in 0..resolved.column_count {
            let rval = duckdb_value_int32(resolved, col_idx, row_idx);

            let column: &DuckDBColumn =
                &columns[<usize as TryFrom<i64>>::try_from(col_idx).unwrap()];

            let string = format!(
                "val: {:?} {:?} {:?}",
                column,
                rval,
                std::ffi::CStr::from_ptr(column.name)
            );
            println!("{}", string);
            let cstring = CString::new(string).unwrap();
            call(cstring.as_ptr() as *const _ as i32);
        }
        println!("\n");
    }

    duckdb_destroy_result(resolved);

    duckdb_close(database);

    Ok(())
}

unsafe fn other() -> Result<(), Box<dyn std::error::Error>> {
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
            println!("val: {:?}", rval);
            // _emscripten_builtin_free(rval);
        }
        println!("\n");
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
fn setup_test() {
    const SNIPPET: &'static [u8] = b"global.document = {body: {}};\x00";

    let sig = "\x00";

    unsafe {
        emscripten_asm_const_int(
            SNIPPET as *const _ as *const u8,
            sig as *const _ as *const u8,
            std::ptr::null() as *const _ as *const u8,
        );
    }
}

#[test]
fn it_works() {
    setup_test();

    main().unwrap();
}
