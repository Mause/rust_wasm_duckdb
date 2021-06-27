#![feature(try_trait)]

use std::env::{join_paths, set_var, split_paths, var};
use std::path::PathBuf;
use which::which;

fn exists(s: String) -> String {
    let path = std::path::Path::new(&s);
    assert_eq!(path.exists(), true, "{}", &s);
    return path.to_string_lossy().to_string();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let emcc_path = which("emcc").expect("emar");
    let emscripten_path = emcc_path.parent().unwrap();

    let clang_path = emscripten_path.parent().unwrap().join("bin");

    let mut path = split_paths(&var("PATH").unwrap()).collect::<Vec<PathBuf>>();
    path.push((*clang_path.to_string_lossy()).into());
    eprintln!("{:?}", path);
    set_var("PATH", join_paths(path)?);

    cc::Build::new()
        .flag("-fvisibility=default")
        .flag("-fPIC")
        .flag("-DDUCKDB_NO_THREADS=1")
        .flag("-DDUCKDB_BUILD_LIBRARY=1")
        .flag("-fexceptions")
        .flag("-Wno-unused-parameter")
        //.cpp(true)
        .flag("-shared")
        .flag("-std=gnu++17")
        .file("src/reexporter.cpp")
        .include(exists(format!(
            "{}/system/lib/libcxx/include",
            emscripten_path.display()
        )))
        .include(exists(format!(
            "{}/system/lib/libcxxabi/include",
            emscripten_path.display()
        )))
        .include(exists(format!(
            "{}/system/include",
            emscripten_path.display()
        )))
        .include(exists(format!(
            "{}/system/lib/libc/musl/include",
            emscripten_path.display()
        )))
        .include(exists(format!(
            "{}/system/lib/libc/musl/arch/emscripten",
            emscripten_path.display()
        )))
        .include(exists(format!(
            "{}/system/lib/libc/musl/arch/generic",
            emscripten_path.display()
        )))
        .include("target")
        .file("target/duckdb.cpp")
        .compile("duckdb");

    #[cfg(windows)]
    {
        println!("{:?}", clang_path);
        std::env::set_var("LIBCLANG_PATH", &clang_path);
    }

    bindgen::builder()
        .header("target/duckdb.h")
        .detect_include_paths(true)
        /*
        .clang_arg(format!(
            "-I{}",
            emscripten_path
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
