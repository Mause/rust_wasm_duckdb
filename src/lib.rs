use js_sys::{Function, Object, Reflect, WebAssembly};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};

// lifted from the `console_log` example
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(a: &str);
}

#[wasm_bindgen(module = "@wasmer/wasi")]
extern "C" {
    #[derive(Debug)]
    type WASI;

    #[wasm_bindgen(constructor)]
    fn new(init: &Object) -> WASI;

    #[wasm_bindgen(method, getter, js_name = wasiImport)]
    fn wasi_import(this: &WASI) -> Object;

    #[wasm_bindgen(method, js_name = getImports)]
    fn get_imports(this: &WASI, module: &WebAssembly::Module) -> Object;

    #[wasm_bindgen(method, js_name = setMemory)]
    fn set_memory(this: &WASI, memory: &WebAssembly::Memory);

    #[wasm_bindgen(method, js_name = getImportNamespace)]
    fn get_import_namespace(this: &WASI, module: &WebAssembly::Module) -> Object;
}

#[wasm_bindgen(module = "@wasmer/wasi/lib/bindings/browser")]
extern "C" {
    #[wasm_bindgen(js_name = default)]
    static wasi_bindings: Object;
}

#[wasm_bindgen(raw_module = "./duckdb.js")]
extern "C" {
    #[wasm_bindgen(js_name = _duckdb_open)]
    fn duckdb_open(path: Option<&str>, database: &[i32]) -> i32;

    #[wasm_bindgen(js_name = calledRun)]
    static initialized: bool;
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// const WASM: &[u8] = include_bytes!("duckdb.wasm");

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object, js_namespace = WebAssembly, typescript_type = "WebAssembly.Global")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type Global;

    #[wasm_bindgen(constructor, js_namespace = WebAssembly, catch)]
    pub fn new(global_descriptor: &Object) -> Result<Global, JsValue>;
}

fn set(instance: &JsValue, name: &str, value: &JsValue) -> Result<bool, JsValue> {
    Reflect::set(instance, &JsValue::from_str(name), value)
}

fn make_global(import: Option<&JsValue>) -> Result<JsValue, JsValue> {
    // console_log!("{:?}", import.and_then(|f| js_sys::JSON::stringify(f).ok()));

    let global_desc = Object::new();

    let name = import
        .map(|v| {
            Reflect::get(&v, &JsValue::from_str("name"))
                .unwrap()
                .as_string()
                .unwrap()
        })
        .unwrap_or("".to_string());
    // if  {
    //     console_log!("{:?}", import);
    // }

    let valie = import
        .and_then(|v| Reflect::get(&v, &JsValue::from_str("value")).ok())
        .filter(|p| p.is_truthy())
        .unwrap_or(JsValue::from_str("i32"));

    // console_log!("{:?}", &valie);
    set(&global_desc, "value", &valie)?;
    set(
        &global_desc,
        "mutable",
        &JsValue::from(name != "__memory_base" && name != "__table_base"),
    )?;

    Ok(Global::new(&global_desc).expect("Global").into())
}

async fn make_instance() -> Result<WebAssembly::Instance, JsValue> {
    console_log!("instantiating a new wasm module directly");

    let window = web_sys::window().unwrap();
    let prom = window.fetch_with_str("http://localhost:8000/duckdb.wasm");
    let resp_value = JsFuture::from(prom).await?;
    let resp: web_sys::Response = resp_value.dyn_into().unwrap();
    let array_buffer = JsFuture::from(resp.array_buffer()?).await?;

    console_log!("loaded into array");
    let module: WebAssembly::Module = JsFuture::from(WebAssembly::compile(&array_buffer))
        .await
        .expect("Compiled failed")
        .into();
    console_log!("Compiled");

    console_log!("{:?}", module);

    let module_imports = Object::new();

    let env = Object::new();
    set(&module_imports, "env", &env)?;

    let got_mem = Object::new();
    set(&module_imports, "GOT.mem", &got_mem)?;

    let got_func = Object::new();
    set(&module_imports, "GOT.func", &got_func)?;

    let wasi_snapshot_preview1 = Object::new();
    set(
        &module_imports,
        "wasi_snapshot_preview1",
        &wasi_snapshot_preview1,
    )?;

    for import in WebAssembly::Module::imports(&module).iter() {
        let name = Reflect::get(&import, &JsValue::from_str("name"))?;
        let module_name = Reflect::get(&import, &JsValue::from_str("module"))?;
        let kind = Reflect::get(&import, &JsValue::from_str("kind"))?
            .as_string()
            .expect("not string?");
        let target = Reflect::get(&module_imports, &module_name)?;

        let value: wasm_bindgen::JsValue = if kind == "global" {
            make_global(Some(&import))?
        } else {
            let body = format!("{{ /*{:?}*/;\nconsole.log(arguments);\ndebugger;}}", import);
            Function::new_no_args(&body).into()
        };

        Reflect::set(&target, &name, &value)?;
    }

    set(
        &env,
        "emscripten_get_sbrk_ptr",
        &Function::new_no_args("{ return 0; }").into(),
    )?;

    let table_desc = Object::new();
    set(&table_desc, "element", &JsValue::from_str("anyfunc"))?;
    set(&table_desc, "initial", &JsValue::from(20000))?;
    set(
        &env,
        "__indirect_function_table",
        &WebAssembly::Table::new(&table_desc).expect("Table"),
    )?;

    // use core::alloc::{GlobalAlloc, Layout};

    // let malloc = wasm_bindgen::closure::Closure::wrap(Box::new(|| {
    //     console_log!("interval elapsed!");
    //     unsafe {
    //         wee_alloc::WeeAlloc::INIT.alloc(Layout::new::<&str>());
    //     }
    // }) as Box<dyn FnMut()>);
    // set(&env, "malloc", &malloc.as_ref().unchecked_ref())?;

    let decl = Object::new();
    set(&decl, "initial", &JsValue::from(24576))?;
    set(&decl, "maximum", &JsValue::from(24576))?;
    let memory = JsValue::from(WebAssembly::Memory::new(&decl).expect("Failed to build memory"));
    set(&env, "memory", &memory).expect("Memory");
    console_log!("Got memory");

    let wasi_args = Object::new();
    Reflect::set(&wasi_args, &"args".into(), &js_sys::Array::new())?;
    Reflect::set(&wasi_args, &"env".into(), &Object::new())?;
    Reflect::set(&wasi_args, &"bindings".into(), &wasi_bindings)?;

    let wasi = WASI::new(&wasi_args);
    wasi.set_memory(&memory.into());

    Object::assign(&wasi_snapshot_preview1, &wasi.wasi_import());

    let res = JsFuture::from(WebAssembly::instantiate_streaming(
        &window.fetch_with_str("http://localhost:8000/duckdb.wasm"),
        &module_imports,
    ))
    .await
    .expect("instantiate_streaming")
    .into();

    Ok(Reflect::get(&res, &"instance".into())
        .expect("instance")
        .into())
}

async fn run_async() -> Result<(), JsValue> {
    // console_log!("duckdb_open: {:?}", duckdb_open.to_string());
    Function::new_no_args("{debugger;}").call0(&JsValue::undefined())?;

    while !(*initialized) {
        console_log!("boop");
        wasm_timer::Delay::new(core::time::Duration::from_millis(1000))
            .await
            .expect("delay");
    }

    let database: [i32; 1] = [0];
    duckdb_open(None, &database);

    let instance = make_instance().await.expect("make_instance");
    console_log!("what");

    let c = instance.exports();

    let malloc = Reflect::get(c.as_ref(), &"emscripten_builtin_malloc".into())
        .expect("malloc")
        .dyn_into::<Function>()
        .expect("mallocmmmm");

    let duckdb_open = Reflect::get(c.as_ref(), &"duckdb_open".into())
        .expect("reflect get")
        .dyn_into::<Function>()
        .expect("duckdb_open export wasn't a function");

    let path = JsValue::null();
    console_log!("db?)");
    let database = malloc
        .call1(&JsValue::undefined(), &8.into())
        .expect("database");
    console_log!("database: {:?}", database);

    let three = duckdb_open
        .call2(&JsValue::null(), &path, &database)
        .expect("call failed");

    console_log!("duckdb status {:?}", three);
    console_log!("database: {:?}", database);

    let mem = Reflect::get(c.as_ref(), &"memory".into())?
        .dyn_into::<WebAssembly::Memory>()
        .expect("memory export wasn't a `WebAssembly.Memory`");

    console_log!("created module has {} pages of memory", mem.grow(0));
    console_log!("giving the module 4 more pages of memory");
    mem.grow(4);
    console_log!("now the module has {} pages of memory", mem.grow(0));

    Ok(())
}

#[wasm_bindgen(start)]
pub fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    spawn_local(async {
        run_async().await.expect_throw("Something went wrong");
    });
}
