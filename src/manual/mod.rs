//! Web manuals for Infinity aircraft.
//!
//! The content isn't written here. It lives as TypeScript in the React site's
//! repo (`src/manual/manuals/<id>/`, with its own README), and that repo's
//! `node src/manual/export-manuals.mjs <this repo>/public/manual-data` writes
//! one JSON file per manual into `public/manual-data/`. `load` fetches a
//! manual's file the first time someone opens it. Re-export after changing
//! the content, and commit the JSON with the change that needs it.
//!
//! A manual has an introduction and a list of chapters. A chapter holds
//! sections (nested through `children`), sections hold blocks, and a long
//! chapter can split into `parts`: sub-chapters on pages of their own,
//! numbered under it ("1.4"). Chapters, parts and sections are numbered from
//! their order, so reordering renumbers everything. `chapter_label` sets what
//! readers see chapters called: the DC-10 manual uses "Section". The main
//! page lists the top-level chapters, then shows the introduction; a chapter
//! with parts lists them after its own sections.
//!
//! `aircraft` lists the aircraft a manual covers. With more than one, the bar
//! has a button for each, and the reader shows only what applies to the one
//! picked: numbers close up, and search only finds what's shown. Content
//! limited to some aircraft carries `applies_to` and a badge such as
//! "DC-10 / KC-10 only". The aircraft shown is in the URL as
//! `?aircraft=kc-10`, so a link opens on the same aircraft. Without it, the
//! reader uses the aircraft this browser last showed, then the first in the
//! list. A link to content the shown aircraft doesn't have opens it on the
//! first aircraft that does, tagged with that aircraft's name.
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
//! title unless the content pins one. Figures take a path relative to
//! `public/manual-assets/<manual id>/`, with `width` and `height` so the page
//! doesn't jump while images load; a `placeholder` figure shows a box of its
//! proportions describing the image to come. Checklist rows can name the
//! crew member who does them and mark memory items, which show boxed.
//! Duplicate or non-URL-safe slugs and unknown `applies_to` names show up as
//! `[manual <id>]` warnings in the browser console. A topic with no blocks
//! renders as "This topic hasn't been written yet."
//!
//! The site owns routing: it serves the picker at `/manual` and
//! `/manual/:manual/:chapter?` for the manuals themselves, passing the
//! chapter in. The reader handles the rest itself through the URL's query
//! and hash: the aircraft shown, deep links to sections, copy-link buttons,
//! and back returning to where the reader left off. Its sticky bar sits
//! `--manual-top-offset` below the top of the scroll container.

mod blocks;
mod dom;
pub mod links;
mod load;
mod model;
mod page;
mod pages;
mod picker;
mod reader;
mod shared;
mod sidebar;
mod toolbar;
mod types;

pub use links::manual_link_for_product;
pub use page::ManualPage;
pub use picker::ManualIndex;
