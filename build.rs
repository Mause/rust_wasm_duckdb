#![feature(try_trait)]

use which::which;

fn eat(command: &mut std::process::Command) {
    let res = command.output().expect("Compile");

    if !res.status.success() {
        panic!("{}", String::from_utf8(res.stderr).expect("String"));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rustc-link-lib=static-nobundle=stdc++");

    let emcc_path =
        which("em++").expect("Couldn't find em++, is the emscripten environment activated?");

    let emar_path =
        which("emar").expect("Couldn't find emar, is the emscripten environment activated?");

    eat(std::process::Command::new(emcc_path)
        .arg("-fvisibility=default")
        .arg("-fPIC")
        .arg("-DDUCKDB_NO_THREADS=1")
        .arg("-sWASM=1")
        .arg("-DDUCKDB_BUILD_LIBRARY=1")
        .arg("-sWARN_ON_UNDEFINED_SYMBOLS=1")
        .arg("-sALLOW_MEMORY_GROWTH=1")
        .arg("-sUSE_PTHREADS=0")
        .arg("-sDISABLE_EXCEPTION_CATCHING=0")
        .arg("-fexceptions")
        .arg("-Wno-unused-parameter")
        .arg("--no-entry")
        .arg("-shared")
        .arg("reexporter.cpp")
        .arg("-IC:\\Users\\me\\duckdb\\src\\amalgamation")
        .arg(
            std::env::current_dir()?
                .join("src\\duckdb.wasm")
                .to_str()
                .expect("oops"),
        )
        .arg("-o")
        .arg("duckdb.o"));

    eat(std::process::Command::new(emar_path)
        .arg("rcs")
        .arg("libduckdb.a")
        .arg("duckdb.o"));

    println!("cargo:rustc-link-lib=static-nobundle=duckdb");
    println!(
        "cargo:rustc-link-search={}",
        std::env::current_dir()?.to_str().expect("aaaaa")
    );

    Ok(())
}
