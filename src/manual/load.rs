//! Loads a manual's content from `public/manual-data/<id>.json`. The files
//! are written from the TypeScript manuals in the React site's repo by its
//! `src/manual/export-manuals.mjs`, which is where the content is edited.
//!
//! A manual is fetched when someone first opens it and then kept for the rest
//! of the visit: parsed once and leaked into `'static` memory, so the reader
//! works on plain borrowed data. Visitors who never open a manual never
//! download one, and the wasm doesn't carry megabytes of text.

use std::cell::RefCell;
use std::collections::HashMap;

use serde::Deserialize;

use super::types::{
    AircraftName, BlockBody, CalloutTone, ChecklistItem, Manual, ManualBlock, ManualChapter,
    ManualSection,
};

/// Where the site serves the files.
const DATA_PATH: &str = "/manual-data";

/// The file format this reader understands. The exporter raises it whenever
/// an older reader would misread its files.
const FORMAT: u32 = 1;

thread_local! {
    static LOADED: RefCell<HashMap<String, &'static Manual>> = RefCell::new(HashMap::new());
}

/// The manual with this id, fetched on first use.
pub async fn load_manual(id: &str) -> Result<&'static Manual, String> {
    if let Some(manual) = LOADED.with_borrow(|loaded| loaded.get(id).copied()) {
        return Ok(manual);
    }
    let url = format!("{DATA_PATH}/{id}.json");
    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("{url}: {e}"))?;
    if !response.ok() {
        return Err(format!("{url}: HTTP {}", response.status()));
    }
    let text = response.text().await.map_err(|e| format!("{url}: {e}"))?;
    let manual = parse(&text).map_err(|e| format!("{url}: {e}"))?;
    if manual.id != id {
        return Err(format!("{url} holds the manual \"{}\"", manual.id));
    }
    // Two loads can race; keep whichever finished first.
    Ok(LOADED.with_borrow_mut(|loaded| *loaded.entry(id.to_string()).or_insert(manual)))
}

fn parse(text: &str) -> Result<&'static Manual, String> {
    #[derive(Deserialize)]
    struct Head {
        format: u32,
    }
    #[derive(Deserialize)]
    struct File {
        format: u32,
        manual: ManualData,
    }

    match serde_json::from_str::<File>(text) {
        Ok(file) if file.format == FORMAT => Ok(file.manual.build()),
        Ok(file) => Err(format!(
            "format {}, but this site reads {FORMAT}",
            file.format
        )),
        // A newer format may not parse at all; say so rather than just failing.
        Err(e) => match serde_json::from_str::<Head>(text) {
            Ok(head) if head.format != FORMAT => Err(format!(
                "format {}, but this site reads {FORMAT}",
                head.format
            )),
            _ => Err(e.to_string()),
        },
    }
}

// The file's shape: the manual exactly as the TypeScript content model
// (src/manual/types.ts) describes it. Unknown fields are ignored.

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManualData {
    id: String,
    title: String,
    aircraft: Vec<String>,
    chapter_label: Option<String>,
    revision: Option<String>,
    summary: Option<String>,
    #[serde(default)]
    introduction: Vec<SectionData>,
    chapters: Vec<ChapterData>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChapterData {
    slug: String,
    title: String,
    summary: Option<String>,
    #[serde(default)]
    applies_to: Vec<String>,
    #[serde(default)]
    sections: Vec<SectionData>,
    #[serde(default)]
    parts: Vec<ChapterData>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SectionData {
    slug: Option<String>,
    title: String,
    summary: Option<String>,
    #[serde(default)]
    applies_to: Vec<String>,
    #[serde(default)]
    blocks: Vec<BlockData>,
    #[serde(default)]
    children: Vec<SectionData>,
}

#[derive(Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum BlockData {
    Text {
        #[serde(default)]
        applies_to: Vec<String>,
        content: String,
    },
    Callout {
        #[serde(default)]
        applies_to: Vec<String>,
        tone: ToneData,
        title: Option<String>,
        content: String,
    },
    List {
        #[serde(default)]
        applies_to: Vec<String>,
        #[serde(default)]
        ordered: bool,
        items: Vec<String>,
    },
    Checklist {
        #[serde(default)]
        applies_to: Vec<String>,
        title: Option<String>,
        items: Vec<ItemData>,
    },
    Table {
        #[serde(default)]
        applies_to: Vec<String>,
        title: Option<String>,
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
        #[serde(default)]
        dense: bool,
    },
    Figure {
        #[serde(default)]
        applies_to: Vec<String>,
        src: String,
        alt: String,
        title: Option<String>,
        caption: Option<String>,
        width: Option<u32>,
        height: Option<u32>,
        #[serde(default)]
        placeholder: bool,
    },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum ToneData {
    Note,
    Caution,
    Warning,
}

#[derive(Deserialize)]
struct ItemData {
    item: String,
    action: String,
    crew: Option<String>,
    #[serde(default)]
    memory: bool,
}

fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn leak_opt(s: Option<String>) -> Option<&'static str> {
    s.map(leak)
}

fn leak_all<T, U>(items: Vec<T>, f: impl FnMut(T) -> U) -> &'static [U] {
    Box::leak(
        items
            .into_iter()
            .map(f)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    )
}

impl ManualData {
    fn build(self) -> &'static Manual {
        let aircraft = leak_all(self.aircraft, leak);
        // Content names its aircraft thousands of times; share the manual's
        // copy of each name rather than keeping every one.
        let names = |names: Vec<String>| -> &'static [AircraftName] {
            leak_all(names, |name| {
                aircraft
                    .iter()
                    .copied()
                    .find(|known| *known == name)
                    .unwrap_or_else(|| leak(name))
            })
        };
        Box::leak(Box::new(Manual {
            id: leak(self.id),
            title: leak(self.title),
            aircraft,
            chapter_label: leak_opt(self.chapter_label),
            revision: leak_opt(self.revision),
            summary: leak_opt(self.summary),
            introduction: leak_all(self.introduction, |s| s.build(&names)),
            chapters: leak_all(self.chapters, |c| c.build(&names)),
        }))
    }
}

type Names<'a> = dyn Fn(Vec<String>) -> &'static [AircraftName] + 'a;

impl ChapterData {
    fn build(self, names: &Names) -> ManualChapter {
        ManualChapter {
            slug: leak(self.slug),
            title: leak(self.title),
            summary: leak_opt(self.summary),
            applies_to: names(self.applies_to),
            sections: leak_all(self.sections, |s| s.build(names)),
            parts: leak_all(self.parts, |c| c.build(names)),
        }
    }
}

impl SectionData {
    fn build(self, names: &Names) -> ManualSection {
        ManualSection {
            slug: leak_opt(self.slug),
            title: leak(self.title),
            summary: leak_opt(self.summary),
            applies_to: names(self.applies_to),
            blocks: leak_all(self.blocks, |b| b.build(names)),
            children: leak_all(self.children, |s| s.build(names)),
        }
    }
}

impl BlockData {
    fn build(self, names: &Names) -> ManualBlock {
        let (applies_to, body) = match self {
            Self::Text {
                applies_to,
                content,
            } => (applies_to, BlockBody::Text(leak(content))),
            Self::Callout {
                applies_to,
                tone,
                title,
                content,
            } => (
                applies_to,
                BlockBody::Callout {
                    tone: match tone {
                        ToneData::Note => CalloutTone::Note,
                        ToneData::Caution => CalloutTone::Caution,
                        ToneData::Warning => CalloutTone::Warning,
                    },
                    title: leak_opt(title),
                    content: leak(content),
                },
            ),
            Self::List {
                applies_to,
                ordered,
                items,
            } => (
                applies_to,
                BlockBody::List {
                    ordered,
                    items: leak_all(items, leak),
                },
            ),
            Self::Checklist {
                applies_to,
                title,
                items,
            } => (
                applies_to,
                BlockBody::Checklist {
                    title: leak_opt(title),
                    items: leak_all(items, |i| ChecklistItem {
                        item: leak(i.item),
                        action: leak(i.action),
                        crew: leak_opt(i.crew),
                        memory: i.memory,
                    }),
                },
            ),
            Self::Table {
                applies_to,
                title,
                columns,
                rows,
                dense,
            } => (
                applies_to,
                BlockBody::Table {
                    title: leak_opt(title),
                    columns: leak_all(columns, leak),
                    rows: leak_all(rows, |row| leak_all(row, leak)),
                    dense,
                },
            ),
            Self::Figure {
                applies_to,
                src,
                alt,
                title,
                caption,
                width,
                height,
                placeholder,
            } => (
                applies_to,
                BlockBody::Figure {
                    src: leak(src),
                    alt: leak(alt),
                    title: leak_opt(title),
                    caption: leak_opt(caption),
                    width,
                    height,
                    placeholder,
                },
            ),
        };
        ManualBlock {
            applies_to: names(applies_to),
            body,
        }
    }
}
