//! Web manuals for Infinity aircraft.
//!
//! A manual (`manuals/<id>/mod.rs`) has an introduction and a list of
//! chapters, one file per chapter. Chapters hold sections (nest them with
//! `children`), and sections hold blocks. Chapters and sections are numbered
//! from their order, so reordering renumbers everything. `chapter_label` sets
//! what readers see chapters called: the DC-10 manual uses "Section", so its
//! seven chapters read "Section 1" to "Section 7". The main page lists the
//! chapters, then shows the introduction.
//!
//! `aircraft` lists the aircraft a manual covers. With more than one, the bar
//! has a button for each, and the reader shows only what applies to the one
//! picked: section numbers close up, and search only finds what's shown. Put
//! `.only(&["KC-10"])` on a chapter, section or block to limit it to some
//! aircraft; such content carries a badge such as "DC-10 / KC-10 only". The
//! aircraft shown is in the URL as `?aircraft=kc-10`, so a link opens on the
//! same aircraft. Without it, the reader uses the aircraft this browser last
//! showed, then the first in the list.
//!
//! Inline markup works in text, callouts, list items, table cells, checklist
//! rows and captions: `**bold**`, `*italic*`, `` `code` `` and
//! `[label](target)`. Link targets:
//!
//! - `#section` is a section in the same chapter.
//! - `chapter` or `chapter#section` is elsewhere in the manual.
//! - `https://…` or `mailto:…` is external and opens in a new tab.
//! - `/path` is a page on the site. Avoid it: it ties the content to one site.
//!
//! A chapter's `slug` is its URL segment. A section's slug defaults to its
//! title; pin one with `ManualSection::at` once anything links to it, so a
//! rename doesn't break those links. Figures take a path relative to
//! `public/manual-assets/<manual id>/`, with `width` and `height` so the page
//! doesn't jump while images load. Duplicate or non-URL-safe slugs and unknown
//! `applies_to` names show up as `[manual <id>]` warnings in the browser
//! console. A topic with no blocks renders as "This topic hasn't been written
//! yet."
//!
//! The site owns routing: it serves `/manual/:manual/:chapter?` and passes
//! the chapter in. The reader handles the rest itself through the URL's query
//! and hash: the aircraft shown, deep links to sections, copy-link buttons,
//! and back returning to where the reader left off. Its sticky bar sits
//! `--manual-top-offset` below the top of the scroll container.

mod blocks;
mod dom;
pub mod links;
mod manuals;
mod model;
mod page;
mod pages;
mod reader;
mod shared;
mod sidebar;
mod toolbar;
mod types;

pub use links::manual_link_for_product;
pub use page::ManualPage;
