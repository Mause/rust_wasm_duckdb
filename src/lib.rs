#![feature(try_trait)]
use crate::state::DuckDBState;
use js_sys::{Function, Object, Reflect, WebAssembly};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

mod state;

// lifted from the `console_log` example
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(a: &str);
}

#[wasm_bindgen]
pub enum DuckDBType {
    string,
}

#[wasm_bindgen]
struct DuckDBColumn {
    data: i32,
    nullmask: i32,
    // #[wasm_bindgen(js_name = "type")]
    type_: DuckDBType,
    name: String,
}

#[wasm_bindgen]
struct Connection {}

#[wasm_bindgen]
struct Database {}

#[wasm_bindgen(raw_module = "./duckdb.js")]
extern "C" {
    #[derive(Debug)]
    type JsString;

    type DuckDBResult;

    #[wasm_bindgen(method, getter)]
    fn row_count(this: &DuckDBResult) -> i32;

    #[wasm_bindgen(method, getter)]
    fn column_count(this: &DuckDBResult) -> i32;

    #[wasm_bindgen(js_name = _duckdb_open)]
    fn duckdb_open(path: Option<String>, database: *mut Database) -> DuckDBState;

    #[wasm_bindgen(js_name = _duckdb_connect)]
    fn duckdb_connect(db: *mut Database, con: *mut Connection) -> DuckDBState;

    #[wasm_bindgen(js_name = _duckdb_disconnect)]
    fn duckdb_disconnect(con: *mut Connection);

    #[wasm_bindgen(js_name = _duckdb_close)]
    fn duckdb_close(db: *mut Database);

    #[wasm_bindgen(js_name = _duckdb_query)]
    fn duckdb_query(
        con: *mut Connection,
        query: JsString,
        result: *mut DuckDBResult,
    ) -> DuckDBState;

    #[wasm_bindgen(js_name = _duckdb_destroy_result)]
    fn duckdb_destroy_result(result: *mut DuckDBResult);

    #[wasm_bindgen(js_name = stringToNewUTF8)]
    fn stringToNewUTF8(string: &str) -> JsString;

    fn maybeCStringToJsString(string: i32) -> JsString;

    #[wasm_bindgen(js_name = _duckdb_value_varchar)]
    fn duckdb_value_varchar(result: *mut DuckDBResult, row: i32, column: i32) -> i32;

    fn cwrap(function: &str, return_type: JsValue, argument_types: Vec<JsValue>) -> Function;

    fn _emscripten_builtin_free(ptr: i32);

    fn _emscripten_builtin_malloc(size: i32) -> *mut Object;
}

fn malloc<T>(size: i32) -> *mut T {
    _emscripten_builtin_malloc(size) as *mut T
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

static PTR: i32 = 4;

async fn run_async() -> Result<(), Box<dyn std::error::Error>> {
    let database = malloc(PTR);
    duckdb_open(None, database)?;

    let connection: *mut Connection = malloc(PTR);
    duckdb_connect(database, connection)?;

    let query = stringToNewUTF8("select 1");
    console_log!("query: {:?}", query);
    let result = malloc(PTR);
    duckdb_query(connection, query, result)?;

    duckdb_disconnect(connection);

    duckdb_close(database);

    Ok(())
}

#[wasm_bindgen(start)]
pub fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    spawn_local(async {
        run_async().await.expect_throw("Something went wrong");
    });
}
