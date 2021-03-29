use dodrio::{bumpalo, Attribute, Node, Render, RenderContext};
#[cfg(test)]
use speculate::speculate;
//  use wasm_bindgen::UnwrapThrowExt;

speculate! {
    test "sanity" {
        use bumpalo::Bump;

        let bump = Bump::new();
        let mut rc = RenderContext { bump };
        Hello.new("world").render(&mut rc);
    }
}

/// A component that greets someone.
pub struct Hello<'who> {
    who: &'who str,
}

impl<'who> Hello<'who> {
    /// Construct a new `Hello` component that greets the given `who`.
    pub fn new(who: &str) -> Hello {
        Hello { who }
    }
}

impl<'a, 'who> Render<'a> for Hello<'who> {
    fn render(&self, cx: &mut RenderContext<'a>) -> Node<'a> {
        use dodrio::builder::*;

        let id = bumpalo::format!(in cx.bump, "hello-{}", self.who);
        let who = bumpalo::collections::String::from_str_in(self.who, cx.bump).into_bump_str();

        div(&cx)
            .attr("id", id.into_bump_str())
            .on("click", |root, _vdom, _event| {
                let hello = root.unwrap_mut::<Hello>();
                /*                web_sys::window()
                .expect_throw("should have a `Window` on the Web")
                .alert_with_message(hello.who);*/
            })
            .children([
                text("Hello, "),
                strong(&cx).children([text(who), text("!")]).finish(),
            ])
            .finish()
    }
}
