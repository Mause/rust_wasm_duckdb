#[macro_export]
macro_rules! jse {
    ($js_expr:expr, $( $i:ident ),*) => {
        {
            const LEN: usize = count_tts!($($i)*);

            #[repr(C, align(16))]
            struct AlignToSixteen([i32; LEN]);

            let array = &AlignToSixteen([$($i,)*]);
            let sig = CString::new("i".repeat(LEN)).expect("sig");
            const SNIPPET: &'static [u8] = $js_expr;

            assert_eq!(SNIPPET[..].last().expect("empty snippet?"), &0);

            unsafe {
                emscripten_asm_const_int(
                    SNIPPET as *const _ as *const u8,
                    sig.as_ptr() as *const _ as *const u8,
                    array as *const _ as *const u8,
                ) as i32
            }
        }
    };
    ($js_expr:expr) => {
        {
            let sig = CString::new("").expect("sig");
            const SNIPPET: &'static [u8] = $js_expr;

            unsafe {
                emscripten_asm_const_int(
                    SNIPPET as *const _ as *const u8,
                    sig.as_ptr() as *const _ as *const u8,
                    std::ptr::null() as *const _ as *const u8,
                ) as i32
            }
        }
    };
}
