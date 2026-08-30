use infinity_ui::prelude::*;

use super::prose::Term;

pub fn doc() -> Doc {
    Doc::new("io", "Async IO", "Module · io", "Modules")
        .scope("infinity-rs")
        .tagline(
            "Sandbox-friendly file IO that runs through the MSFS host. Open, read, and write panel-relative files without blocking the sim thread.",
        )
        .tags(["File", "Async", "Sandbox", "OpenFlags"])
        .section(model())
        .section(open_flags())
        .section(open_and_read())
        .section(writing())
        .section(polling())
}

fn model() -> DocSection {
    DocSection::new("model", "Model", || {
        view! {
            <Lead>
                "MSFS gauges run in a sandboxed WASM context. The host exposes its own IO layer scoped to the package work directory; "
                <InlineCode>"infinity_rs::io"</InlineCode>
                " is a thin safe wrapper around it. Calls are non-blocking — you provide a callback that fires when the operation completes."
            </Lead>

            <List>
                <ListItem>
                    <Term>"Paths"</Term>
                    " are panel-relative (e.g. "
                    <InlineCode>"\"data/state.json\""</InlineCode>
                    "). Absolute / parent-traversal paths are rejected by the host."
                </ListItem>
                <ListItem>
                    <Term>"Handles"</Term>
                    " are owned: dropping a "
                    <InlineCode>"File"</InlineCode>
                    " closes it and aborts any in-flight async op."
                </ListItem>
                <ListItem>
                    <Term>"No threads"</Term>
                    " — completion callbacks run on the sim thread between frames."
                </ListItem>
            </List>
        }
    })
    .numbered("1.0")
    .summary("Why MSFS provides its own IO layer instead of letting wasi-libc do it.")
}

fn open_flags() -> DocSection {
    DocSection::new("open-flags", "OpenFlags & errors", || {
        view! {
            <CodeBlock
                language="rust"
                title="flags & errors"
                code=r#"pub struct OpenFlags;
impl OpenFlags {
    pub const RDONLY: Self;
    pub const WRONLY: Self;
    pub const RDWR:   Self;
    pub const CREATE: Self;
    pub const TRUNC:  Self;
    pub const APPEND: Self;
}

pub enum IoError {
    Success,                // typically not surfaced
    BadParams,
    FileNotFound,
    AccessNotAllowed,
    FileNotOpened,
    ReadNotAllowed,
    PartialReadImpossible,
    OperationImpossible,
    NulByteInPath,
    Other(i32),
}"#
            />
        }
    })
    .numbered("2.0")
    .summary("Standard read/write/create/truncate matrix and a typed error enum.")
}

fn open_and_read() -> DocSection {
    DocSection::new("open-and-read", "Open + read", || {
        view! {
            <CodeBlock
                language="rust"
                title="one-shot read"
                code=r#"use infinity_rs::io::{self, OpenFlags};

let req = io::open_read("data/state.json", |result| {
    match result {
        Ok(buf)   => println!("got {} bytes", buf.len()),
        Err(err)  => eprintln!("read failed: {err}"),
    }
})?;
// hold req (or _req) until the callback fires"#
            />

            <CodeBlock
                language="rust"
                title="open then read"
                code=r#"let file = io::open("data/state.json", OpenFlags::RDONLY, |file| {
    let size = file.file_size();
    let _ = file.read(0, size as u32, |buf| {
        let _ = std::str::from_utf8(buf).map(|s| println!("loaded: {s}"));
    });
})?;"#
            />
        }
    })
    .numbered("2.1")
    .summary("Two flavours: callback-style `open_read` (one-shot, you get the bytes), or open-then-issue-read on a `File` handle when you want to interleave operations.")
}

fn writing() -> DocSection {
    DocSection::new("writing", "Writing", || {
        view! {
            <CodeBlock
                language="rust"
                title="write"
                code=r#"use infinity_rs::io::{self, OpenFlags};

let req = io::open(
    "data/state.json",
    OpenFlags::WRONLY | OpenFlags::CREATE | OpenFlags::TRUNC,
    |file| {
        let _ = file.write(0, b"{\"hello\":1}", |result| {
            if let Err(err) = result { eprintln!("write failed: {err}") }
        });
    },
)?;"#
            />

            <Callout
                tone=Tone::Warning
                icon=icons::TRIANGLE_ALERT
                title="Hold the request handle"
            >
                "The returned "
                <InlineCode>"File"</InlineCode>
                " / request handle keeps your closures alive. If you let it drop the host aborts the operation."
            </Callout>
        }
    })
    .numbered("2.2")
    .summary("Open with `WRONLY | CREATE | TRUNC`, then write a buffer; the callback fires on completion.")
}

fn polling() -> DocSection {
    DocSection::new("polling", "Polling vs callbacks", || {
        view! {
            <P>
                <InlineCode>"File::is_done()"</InlineCode>
                ", "
                <InlineCode>"has_error()"</InlineCode>
                ", and "
                <InlineCode>"last_error()"</InlineCode>
                " let you poll a request from your "
                <InlineCode>"System::update"</InlineCode>
                " rather than handling completion through a callback. Useful when you need to chain dependent operations through your tick loop instead of nested closures."
            </P>
        }
    })
    .numbered("3.0")
    .summary("Most callers want callbacks; polling helpers are available when you need them.")
}
