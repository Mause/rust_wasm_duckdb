import run from './rust_wasm_duckdb.js';
import { addOnPostRun } from './duckdb.js';

addOnPostRun(() => {
    run("./rust_wasm_duckdb_bg.wasm");
});
