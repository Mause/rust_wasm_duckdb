#![feature(try_trait)]

fn main() -> Result<(), Box<dyn std::error::Error>> {
    cc::Build::new()
        .flag("-fvisibility=default")
        .flag("-fPIC")
        .flag("-DDUCKDB_NO_THREADS=1")
        .flag("-sWASM=1")
        .flag("-DDUCKDB_BUILD_LIBRARY=1")
        .flag("-sWARN_ON_UNDEFINED_SYMBOLS=1")
        .flag("-sALLOW_MEMORY_GROWTH=1")
        .flag("-sUSE_PTHREADS=0")
        .flag("-sDISABLE_EXCEPTION_CATCHING=0")
        .flag("-fexceptions")
        .flag("-Wno-unused-parameter")
        .flag("--no-entry")
        .flag("-shared")
        .file("src/reexporter.cpp")
        .include("target")
        // .flag("target/duckdb.wasm")
        .cpp_link_stdlib("stdc++ target/duckdb.wasm")
        .no_default_flags(true)
        .compile("libduckdb.a");

    Ok(())
}
