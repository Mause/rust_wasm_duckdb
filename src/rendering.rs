use crate::{DbType, DuckDBColumn, ResolvedResult};
use render::{component, rsx, Render};
use std::ffi::CStr;
use std::iter::{FromIterator, Map};

pub trait Contain<I: Render> {
    fn contain(self) -> Container<I>;
}

impl<B: Render, I: Iterator, F> Contain<B> for Map<I, F>
where
    F: FnMut(I::Item) -> B,
    Vec<B>: FromIterator<B>,
{
    fn contain(self) -> Container<B> {
        Container(self.collect::<Vec<B>>())
    }
}

pub struct Container<T: Render>(pub Vec<T>);
impl<T: Render> Render for Container<T> {
    fn render_into<W>(self, writer: &mut W) -> Result<(), std::fmt::Error>
    where
        W: std::fmt::Write,
    {
        for item in self.0 {
            item.render_into(writer)?;
        }

        Ok(())
    }
}

impl Render for DbType {
    fn render_into<W: core::fmt::Write>(self, writer: &mut W) -> Result<(), std::fmt::Error> {
        writer.write_str(&self.to_string())
    }
}

#[component]
pub fn Table<'a>(resolved: &'a ResolvedResult<'a>) {
    let head = (0..resolved.resolved.column_count)
        .map(|col_idx| {
            let column: &DuckDBColumn = resolved.column(col_idx);
            let name = unsafe { CStr::from_ptr(column.name) }
                .to_string_lossy()
                .to_string();

            let type_: &str = resolved.consume(col_idx, 0).expect("consume").into();

            rsx! { <td>{name}{": "}{type_}</td> }
        })
        .contain();

    let body = (0..resolved.resolved.row_count)
        .map(|row| {
            rsx! {
                <tr>
                    {
                        (
                            (0..resolved.resolved.column_count)
                            .map(|col| {
                                let value = resolved.consume(col, row).expect("consume");

                                rsx!{<td>{value}</td>}
                            })
                            .contain()
                        )
                }
                </tr>
            }
        })
        .contain();

    rsx! {
        <table>
            <thead>{head}</thead>
            <tbody>{body}</tbody>
        </table>
    }
}

#[component]
pub fn Form() {
    let files = Container(
        std::fs::read_dir(std::path::Path::new("/"))
            .expect("files")
            .map(|f| rsx! { <li>{f.unwrap().path().to_str().unwrap().to_string()}</li> })
            .collect(),
    );

    rsx! {
        <div>
            <form onsubmit={"event.preventDefault(); Module.ccall('callback', 'void', ['string'], [document.forms[0].query.value])"}>
                <input placeholder={"select random()"} autofocus={"true"} name={"query"}></input>
            </form>
            <ul>
                {files}
            </ul>
        </div>
    }
}
