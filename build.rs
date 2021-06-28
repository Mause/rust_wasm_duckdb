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

    assert_eq!(
        String::from_utf8(
            cc::Build::new()
                .get_compiler()
                .to_command()
                .arg("--version")
                .output()
                .expect("Couldnt check emcc version")
                .stdout
        )
        .expect("Couldnt check emcc version")
        .contains("2.0.15"),
        true
    );

    let emar_path =
        which("emar").expect("Couldn't find emar, is the emscripten environment activated?");

    eat(cc::Build::new()
        .get_compiler()
        .to_command()
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
        .arg("src/reexporter.cpp")
        .arg("-Itarget")
        // .arg("target/duckdb.wasm")
        .arg("target/duckdb.cpp")
        .arg("-o")
        .arg("duckdb.o"));

    println!("{:?}", &emar_path);
    eat(std::process::Command::new(&emar_path)
        .arg("rcs")
        .arg("target/libduckdb.a")
        .arg("duckdb.o"));

    println!("cargo:rustc-link-lib=static-nobundle=duckdb");
    println!(
        "cargo:rustc-link-search={}",
        std::env::current_dir()?
            .join("target")
            .to_str()
            .expect("aaaaa")
    );

    #[cfg(windows)]
    {
        let p = emar_path.parent().unwrap().parent().unwrap().join("bin");

        println!("{:?}", p);
        std::env::set_var("LIBCLANG_PATH", &p);
    }

    bindgen::builder()
        .header("target/duckdb.h")
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
