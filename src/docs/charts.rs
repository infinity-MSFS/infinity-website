use infinity_ui::prelude::*;

use super::prose::Term;

pub fn doc() -> Doc {
    Doc::new("charts", "Charts", "Module · charts", "Modules")
        .scope("infinity-rs")
        .tagline(
            "Index, page, and image accessors for the Navigraph (and other) chart providers wired into MSFS. Owned RAII handles automatically free host allocations on drop.",
        )
        .tags(["Navigraph", "ICAO", "ChartIndex", "ChartImage"])
        .section(icao())
        .section(index())
        .section(pages())
        .section(errors())
}

fn icao() -> DocSection {
    DocSection::new("icao", "Building an ICAO", || {
        view! {
            <Lead>
                <InlineCode>"FsIcao"</InlineCode>
                " is a fixed-layout struct with "
                <InlineCode>"type"</InlineCode>
                ", "
                <InlineCode>"region"</InlineCode>
                ", "
                <InlineCode>"airport"</InlineCode>
                ", and "
                <InlineCode>"ident"</InlineCode>
                " fields, each with strict size limits. "
                <InlineCode>"make_icao"</InlineCode>
                " truncates and NUL-terminates for you."
            </Lead>

            <CodeBlock
                language="rust"
                title="make_icao"
                code=r#"use infinity_rs::charts;

let icao = charts::make_icao(
    'A',           // type — single char
    "K1",          // region — 2 chars + NUL
    "KSEA",        // airport — up to 8 chars + NUL
    "KSEA",        // ident   — up to 8 chars + NUL
);"#
            />
        }
    })
    .numbered("1.0")
    .summary("`make_icao` packs the four-field FsIcao the SDK expects.")
}

fn index() -> DocSection {
    DocSection::new("index", "Chart index", || {
        view! {
            <CodeBlock
                language="rust"
                title="get_index"
                code=r#"use infinity_rs::charts::{self, ChartIndex};

let icao = charts::make_icao('A', "K1", "KSEA", "KSEA");
let index: ChartIndex = charts::get_index(&icao)?;

for category in index.categories() {
    println!("category: {}", category.name());
    for meta in category.charts() {
        println!(
            "  {} — {} ({} pages)",
            meta.identifier(),
            meta.title(),
            meta.page_count(),
        );
    }
}"#
            />

            <List>
                <ListItem>
                    <Term>
                        <InlineCode>"ChartIndex"</InlineCode>
                    </Term>
                    " owns the host allocation; "
                    <InlineCode>"Drop"</InlineCode>
                    " calls "
                    <InlineCode>"fsChartsFreeChartIndex"</InlineCode>
                    "."
                </ListItem>
                <ListItem>
                    <Term>
                        <InlineCode>"ChartCategoryRef<'a>"</InlineCode>
                        " / "
                        <InlineCode>"ChartMetadataRef<'a>"</InlineCode>
                    </Term>
                    " are borrowed views — don't outlive the index."
                </ListItem>
            </List>
        }
    })
    .numbered("2.0")
    .summary("Get the list of chart categories and their charts for an airport.")
}

fn pages() -> DocSection {
    DocSection::new("pages", "Chart pages", || {
        view! {
            <CodeBlock
                language="rust"
                title="get_pages + page image"
                code=r#"use infinity_rs::charts::{self, ChartPages, ChartImage};

let pages: ChartPages = charts::get_pages(meta.guid())?;
for page in pages.pages() {
    let (name, url) = page.name_url();
    println!("page {name} → {url}");
}

// Load a single page image (returns an NVG-friendly handle)
let img: ChartImage = charts::get_page_image(pages.pages().next().unwrap())?;"#
            />

            <Callout
                tone=Tone::Warning
                icon=icons::TRIANGLE_ALERT
                title="ChartImage cleanup"
            >
                "The SDK does not expose a dedicated free for chart-page images. Treat it as an NVG image — call "
                <InlineCode>"img.delete_with_nvg(nvg)"</InlineCode>
                " when you're done, or let the gauge tear-down release the underlying NVG context."
            </Callout>
        }
    })
    .numbered("2.1")
    .summary("Each chart has one or more pages identified by `(name, url)` pairs.")
}

fn errors() -> DocSection {
    DocSection::new("errors", "Errors", || {
        view! {
            <CodeBlock
                language="rust"
                title="ChartError"
                code=r#"pub enum ChartError {
    GuidUnknown,    // bad guid
    BadProvider,    // unrecognised provider
    Network,        // transient — retry ok
    Internal,       // host failure — do not retry
    Unknown(i32),
}"#
            />
        }
    })
    .numbered("3.0")
    .summary("`ChartError` distinguishes transient failures (safe to retry) from permanent ones.")
}
