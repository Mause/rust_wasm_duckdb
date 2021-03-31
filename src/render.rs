use render::{component, html, rsx, Render, SimpleElement};

struct Container<T: Render>(Vec<T>);
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
fn Table<'a>(resolved: &'a ResolvedResult<'a>) -> render::SimpleElement {
    let head: Vec<_> = (0..resolved.resolved.column_count)
        .map(|col_idx| {
            let column: &DuckDBColumn = resolved.column(col_idx);
            let name = unsafe { CStr::from_ptr(column.name) }
                .to_string_lossy()
                .to_string();
            let type_: &str = column.type_.into();

            rsx! { <td>{name}{": "}{type_}</td> }
        })
        .collect();

    let head = Container(head);

    let body = Container(
        (0..resolved.resolved.row_count)
            .map(|row| {
                rsx! {
                    <tr>
                        {
                            Container(
                                (0..resolved.resolved.column_count)
                                .map(|col| {
                                    let value = resolved.consume(col, row).expect("consume");

                                    rsx!{<td>{value}</td>}
                                })
                                .collect()
                            )
                    }
                    </tr>
                }
            })
            .collect(),
    );

    rsx! {
        <table>
            <thead>{head}</thead>
            <tbody>{body}</tbody>
        </table>
    }
}
