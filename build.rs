#![feature(try_trait)]

fn main() -> Result<(), Box<dyn std::error::Error>> {
    cc::Build::new()
        .flag("-fvisibility=default")
        .flag("-fPIC")
        .flag("-DDUCKDB_NO_THREADS=1")
        .flag("-DDUCKDB_BUILD_LIBRARY=1")
        .flag("-fexceptions")
        .flag("-Wno-unused-parameter")
        .flag("-shared")
        .file("src/reexporter.cpp")
        .flag("-Itarget")
        .file("target/duckdb.cpp")
        .compile("duckdb");

    #[cfg(windows)]
    {
        let p = emar_path.parent().unwrap().parent().unwrap().join("bin");

        println!("{:?}", p);
        std::env::set_var("LIBCLANG_PATH", &p);
    }

    bindgen::builder()
        .header("target/duckdb.h")
        .detect_include_paths(true)
        /*
        .clang_arg(format!(
            "-I{}",
            emar_path
                .join("../cache/sysroot/include")
                .to_str()
                .expect("include path")
        ))
        */
        .generate_block(true)
        .rustified_enum(".*")
        // .clang_arg("-DDUCKDB_BUILD_LIBRARY")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("failed?")
        .write_to_file(std::env::var("OUT_DIR")? + "/bindings.rs")?;

    Ok(())
}
