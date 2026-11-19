//! Content model for web manuals. `load` builds it from a manual's JSON once
//! per visit and keeps it for good, so everything here borrows `'static`
//! data and the reader can pass it around freely.

/// An aircraft name exactly as it appears in [`Manual::aircraft`], e.g. "DC-10".
pub type AircraftName = &'static str;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalloutTone {
    Note,
    Caution,
    Warning,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChecklistItem {
    /// The control or item, e.g. "Parking brake".
    pub item: &'static str,
    /// Its required state, e.g. "SET".
    pub action: &'static str,
    /// Who does it, e.g. "FE", "P" or "PM". Shown after the action.
    pub crew: Option<&'static str>,
    /// A memory item: done without the checklist, so it's boxed and bold.
    pub memory: bool,
}

/// What a block shows. Text, callouts, list items, table cells, checklist rows
/// and captions take inline markup: `**bold**`, `*italic*`, `` `code` `` and
/// `[label](target)` links (see the module docs for link targets).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockBody {
    /// A blank line starts a new paragraph; a single newline is a line break.
    Text(&'static str),
    Callout {
        tone: CalloutTone,
        /// Defaults to the tone ("Note", "Caution", "Warning").
        title: Option<&'static str>,
        content: &'static str,
    },
    List {
        /// Numbered steps instead of bullets.
        ordered: bool,
        items: &'static [&'static str],
    },
    Checklist {
        title: Option<&'static str>,
        items: &'static [ChecklistItem],
    },
    Table {
        title: Option<&'static str>,
        columns: &'static [&'static str],
        rows: &'static [&'static [&'static str]],
        /// Smaller type for wide reference tables.
        dense: bool,
    },
    Figure {
        /// Absolute URL, or a path resolved against the reader's `asset_base`.
        src: &'static str,
        alt: &'static str,
        title: Option<&'static str>,
        caption: Option<&'static str>,
        width: Option<u32>,
        height: Option<u32>,
        /// The image isn't made yet. Shows a box of the figure's proportions
        /// with `alt` (what the image should show) and `src` (where to put
        /// it), instead of loading `src`.
        placeholder: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManualBlock {
    /// Limits the block to some of the manual's aircraft. Empty when the block
    /// applies to every aircraft the manual covers.
    pub applies_to: &'static [AircraftName],
    pub body: BlockBody,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManualSection {
    /// Fragment used in deep links (`…/chapter#slug`). Defaults to a slug of
    /// the title. Pin it once anything links to the section, so a retitle
    /// doesn't break those links.
    pub slug: Option<&'static str>,
    pub title: &'static str,
    pub summary: Option<&'static str>,
    pub applies_to: &'static [AircraftName],
    pub blocks: &'static [ManualBlock],
    pub children: &'static [ManualSection],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManualChapter {
    /// URL segment, e.g. "normal-procedures". Unique within the manual,
    /// parts included.
    pub slug: &'static str,
    pub title: &'static str,
    pub summary: Option<&'static str>,
    pub applies_to: &'static [AircraftName],
    /// The chapter's own topics, shown above its parts.
    pub sections: &'static [ManualSection],
    /// Sub-chapters, each on its own page and numbered under this one: part 4
    /// of chapter 1 is "1.4". For a long chapter such as a systems
    /// description, where one page per system reads better than one page for
    /// them all.
    pub parts: &'static [ManualChapter],
}

#[derive(Debug, PartialEq, Eq)]
pub struct Manual {
    /// Stable id, e.g. "dc10".
    pub id: &'static str,
    pub title: &'static str,
    /// Aircraft the manual covers, in the order of the aircraft buttons. With
    /// more than one, the reader shows only what applies to the one picked.
    /// Every `applies_to` value must come from here.
    pub aircraft: &'static [AircraftName],
    /// What readers call the top-level chapters, e.g. "Section". Defaults to
    /// "Chapter".
    pub chapter_label: Option<&'static str>,
    /// Shown on the main page, e.g. "Revision 1.2" or "Draft".
    pub revision: Option<&'static str>,
    pub summary: Option<&'static str>,
    /// Shown on the main page, after the list of chapters.
    pub introduction: &'static [ManualSection],
    pub chapters: &'static [ManualChapter],
}
