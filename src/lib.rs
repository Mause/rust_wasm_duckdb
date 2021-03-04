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
struct DuckDBResult {
    column_count: i32,
    row_count: i32,
    columns: Vec<DuckDBColumn>,
    error_message: String,
}

#[wasm_bindgen(raw_module = "./duckdb.js")]
extern "C" {
    #[wasm_bindgen(js_name = _duckdb_open)]
    fn duckdb_open(path: Option<String>, database: i32) -> DuckDBState;

    #[wasm_bindgen(js_name = _duckdb_connect)]
    fn duckdb_connect(db: i32, con: i32) -> DuckDBState;

    #[wasm_bindgen(js_name = _duckdb_disconnect)]
    fn duckdb_disconnect(con: i32);

    #[wasm_bindgen(js_name = _duckdb_close)]
    fn duckdb_close(db: i32);

    #[wasm_bindgen(js_name = _duckdb_query)]
    fn duckdb_query(con: i32, query: String, result: Option<i32>) -> DuckDBState;

    #[wasm_bindgen]
    fn _emscripten_builtin_malloc(size: i32) -> i32;
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

async fn run_async() -> Result<(), Box<dyn std::error::Error>> {
    let database = _emscripten_builtin_malloc(4);
    duckdb_open(None, database)?;

    let connection = _emscripten_builtin_malloc(4);
    duckdb_connect(database, connection)?;

    duckdb_query(connection, "select 1\0", None)?;

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
