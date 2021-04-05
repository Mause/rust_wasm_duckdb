use crate::jse;
use crate::{duckdb_timestamp, emscripten_asm_const_int, hook, main};
use speculate::speculate;
use std::ffi::CString;

#[cfg(test)]
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

    test "to_string_works" {
        use crate::types::*;
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
