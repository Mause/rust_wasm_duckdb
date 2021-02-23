#[test]
fn it_works() {
    rust_wasm_duckdb::run();
    assert_eq!(2 + 2, 4);
}
