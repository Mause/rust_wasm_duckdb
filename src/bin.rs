#![feature(extern_types)]
#![feature(try_trait)]
#![feature(static_nobundle)]
#![feature(proc_macro_hygiene)]

#[cfg(test)]
use speculate::speculate;

use crate::state::DuckDBState;
use count_tts::count_tts;
use libc::c_void;
#[allow(non_camel_case_types)]
pub type c_char = i8;
use crate::rendering::Table;
use crate::types::duckdb_date;
use render::html;
use std::convert::{TryFrom, TryInto};
use std::ffi::{CStr, CString};
use strum_macros::IntoStaticStr;

mod rendering;
mod state;
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
    Integer(i64),
    Float(f32),
    Date(*const duckdb_date),
    Double(f64),
    String(String),
    Unknown(String),
}
impl ToString for DbType {
    fn to_string(&self) -> String {
        use crate::DbType::*;

        if let Date(s) = self {
            return unsafe { s.as_ref().expect("date resolved") }.to_string();
        }

        let value: &dyn ToString = match self {
            Integer(i) => i,
            Float(f) => f,
            Double(f) => f,
            String(s) => s,
            Unknown(s) => s,
            Date(_) => panic!("Should not get here"),
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
    fn duckdb_value_float(result: *const DuckDBResult, col: i64, row: i64) -> f32;
    /// Converts the specified value to a double. Returns 0.0 on failure or NULL.
    fn duckdb_value_double(result: *const DuckDBResult, col: i64, row: i64) -> f64;
    /// Converts the specified value to a string. Returns nullptr on failure or NULL. The result must be freed with free.
    fn duckdb_value_varchar(result: *const DuckDBResult, col: i64, row: i64) -> *const c_char;
    /// Fetches a blob from a result set column. Returns a blob with blob.data set to nullptr on failure or NULL. The
    /// resulting "blob.data" must be freed with free.
    fn duckdb_value_blob(result: *const DuckDBResult, col: i64, row: i64) -> *const DuckDBBlob;

    fn duckdb_value_date(result: *const DuckDBResult, col: i64, row: i64) -> *const duckdb_date;

    fn query(db: *const Database, query: *const c_char, result: *const DuckDBResult)
        -> DuckDBState;

    pub fn emscripten_asm_const_int(
        code: *const u8,
        sigPtr: *const u8,
        argBuf: *const u8,
    ) -> *mut u8;

    pub fn mallocy() -> *const c_void;
}

fn malloc<T: Sized>(size: usize) -> *const T {
    unsafe { mallocy() as *const T }
}

static PTR: usize = core::mem::size_of::<i32>();

macro_rules! jse {
    ($js_expr:expr, $( $i:ident ),*) => {
        {
            const LEN: usize = count_tts!($($i)*);

            #[repr(C, align(16))]
            struct AlignToSixteen([i32; LEN]);

            let array = &AlignToSixteen([$($i,)*]);
            let sig = CString::new("i".repeat(LEN)).expect("sig");
            const SNIPPET: &'static [u8] = $js_expr;

            assert_eq!(SNIPPET[..].last().expect("empty snippet?"), &b"\x00"[0]);

            unsafe {
                emscripten_asm_const_int(
                    SNIPPET as *const _ as *const u8,
                    sig.as_ptr() as *const _ as *const u8,
                    array as *const _ as *const u8,
                ) as i32
            }
        }
    };
    ($js_expr:expr) => {
        {
            let sig = CString::new("").expect("sig");
            const SNIPPET: &'static [u8] = $js_expr;

            unsafe {
                emscripten_asm_const_int(
                    SNIPPET as *const _ as *const u8,
                    sig.as_ptr() as *const _ as *const u8,
                    std::ptr::null() as *const _ as *const u8,
                ) as i32
            }
        }
    };
}

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
                DuckDBTypeInteger => DbType::Integer(duckdb_value_int64(result, col, row)),
                DuckDBTypeDate => DbType::Date(duckdb_value_date(result, col, row)),
                DuckDBTypeFloat => DbType::Float(duckdb_value_float(result, col, row)),
                DuckDBTypeDouble => DbType::Double(duckdb_value_double(result, col, row)),
                DuckDBTypeVarchar => DbType::String(
                    CStr::from_ptr(duckdb_value_varchar(result, col, row))
                        .to_string_lossy()
                        .to_string(),
                ),
                _ => DbType::Unknown("unknown".to_string()),
            }
        })
    }
}

unsafe fn run_async() -> Result<(), Box<dyn std::error::Error>> {
    set_page_title("DuckDB Test".to_string());

    let database = malloc(PTR);
    let path = CString::new("db.db").unwrap();
    duckdb_open(path.as_ptr(), database)?;

    println!("DB open");

    let s = CString::new("SELECT 4,2").expect("string");

    let result = malloc(PTR);
    let status = query(database, s.as_ptr(), result);
    println!("status: {}", status);
    status?;

    let resolved = ResolvedResult::new(result);

    println!("columns: {:?}", resolved.columns);

    let table = Table {
        resolved: &resolved,
    };
    let string = html! { <>{table}</> };
    println!("{}", string);

    set_body_html(string);

    duckdb_destroy_result(result);

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

/// Dummy __gxx_personality_v0 hook
extern "C" fn ___gxx_personality_v0() {}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::panic::set_hook(Box::new(hook));

    unsafe {
        run_async().expect("Ooops");
    }

    Ok(())
}

#[cfg(test)]
speculate! {
    before {
        std::panic::set_hook(Box::new(hook));

        jse!(b"global.document = {body: {}};\x00");
    }

    after {
        jse!(b"delete global.document;\x00");
    }

    test "works" {
        main().unwrap();
    }

    test "to_string_works" {
        use crate::types::*;
        let value = duckdb_timestamp::new(duckdb_date::new(1996, 8, 7), duckdb_time::new(12, 10, 0, 0));

        assert_eq!(value.to_string(), "1996-08-07T12:10:00.0");
    }

    test "multi args works" {
        fn addition(a: i32, b: i32) -> i32 {
            jse!(b"return $0 + $1;\x00", a, b)
        }

        assert_eq!(addition(10, 12), 22);
    }

    test "html" {
        use render::{component, rsx, html};

        #[component]
        fn Heading<'title>(title: &'title str) {
              rsx! { <h1 class={"title"}>{title}</h1> }
        }

        let rendered_html = html! {
              <Heading title={"Hello world!"} />
        };

        assert_eq!(rendered_html, r#"<h1 class="title">Hello world!</h1>"#);
    }
}
