use crate::jse;
use crate::{c_char, duckdb_timestamp, duckdb_time, duckdb_date, emscripten_asm_const_int, hook, main};
use speculate::speculate;
use std::ffi::{CStr, CString};

fn parse(html: String) -> kuchiki::NodeRef {
    use kuchiki::traits::TendrilSink;

    let mut v = Vec::from(unsafe { html.clone().as_bytes_mut() });

    let resultant = kuchiki::parse_html()
        .from_utf8()
        .read_from(&mut std::io::Cursor::new(&mut v))
        .expect("parsing failed");

    resultant.first_child().expect("first_child")
}

fn get_document_html() -> String {
    let ptr = jse!(b"return allocateUTF8OnStack(document.body.innerHTML);\x00");

    return unsafe { CStr::from_ptr(ptr as *const c_char) }.to_string_lossy().to_string();
}

speculate! {
    before {
        std::panic::set_hook(Box::new(hook));

        jse!(b"global.document = {body: {}};\x00");
    }

    after {
        jse!(b"delete global.document;\x00");
    }

    test "works" {
        main().unwrap();
    }
    
    test "timestamp" {
        duckdb_timestamp::new(
            duckdb_date::new(2021, 1, 1),
            duckdb_time::new(11, 13, 0, 0)
        );
    }

    test "to_string_works" {
        let value = duckdb_timestamp::new(duckdb_date::new(1996, 8, 7), duckdb_time::new(12, 10, 0, 0));

        assert_eq!(value.to_string(), "1996-08-07T12:10:00.0");
    }

    test "multi args works" {
        fn addition(a: i32, b: i32) -> i32 {
            jse!(b"return $0 + $1;\x00", a, b)
        }

        assert_eq!(addition(10, 12), 22);
    }

    test "html" {
        use render::{component, rsx, html};

        #[component]
        fn Heading<'title>(title: &'title str) {
              rsx! { <h1 class={"title"}>{title}</h1> }
        }

        let rendered_html = html! {
              <Heading title={"Hello world!"} />
        };

        assert_eq!(rendered_html, r#"<h1 class="title">Hello world!</h1>"#);
    }
}
