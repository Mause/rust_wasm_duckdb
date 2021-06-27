#![feature(try_trait)]

use which::which;

fn eat(command: &mut std::process::Command) {
    let res = command.output().expect("Compile");

    if !res.status.success() {
        panic!("{}", String::from_utf8(res.stderr).expect("String"));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: reenable
    // println!("cargo:rustc-link-lib=static-nobundle=stdc++");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=target/duckdb.hpp");
    println!("cargo:rerun-if-changed=target/duckdb.cpp");
    println!("cargo:rerun-if-changed=target/duckdb.h");
    println!("cargo:rerun-if-changed=src/reexporter.cpp");
    println!("cargo:rerun-if-changed=src/interface.cpp");

    let emar_path =
        which("emar").expect("Couldn't find emar, is the emscripten environment activated?");

    cc::Build::new()
        .flag("-fvisibility=default")
        .flag("-fPIC")
        .define("DUCKDB_NO_THREADS", "1")
        .flag("-sWASM=1")
        .define("DUCKDB_BUILD_LIBRARY", "1")
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
        .file("target/duckdb.cpp")
        .compile("duckdb.o");

    #[cfg(windows)]
    {
        let p = emar_path.parent().unwrap().parent().unwrap().join("bin");

        println!("{:?}", p);
        std::env::set_var("LIBCLANG_PATH", &p);
    }

    bindgen::builder()
        .header("target/duckdb.h")
        .header("src/interface.hpp")
        // .detect_include_paths(true)
        .clang_arg(format!(
            "-I{}",
            emar_path
                .join("../cache/sysroot/include")
                .to_str()
                .expect("include path")
        ))
        .generate_block(true)
        .rustified_enum(".*")
        // .clang_arg("-DDUCKDB_BUILD_LIBRARY")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("failed?")
        .write_to_file(std::env::var("OUT_DIR")? + "/bindings.rs")?;

    Ok(())
}
