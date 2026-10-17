//! Content model for web manuals. Plain `const` data, so a manual compiles
//! into the binary and the reader never fetches or parses anything.

// The draft manual doesn't use every block type and builder yet.
#![allow(dead_code)]

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
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManualBlock {
    /// Limits the block to some of the manual's aircraft. Empty when the block
    /// applies to every aircraft the manual covers.
    pub applies_to: &'static [AircraftName],
    pub body: BlockBody,
}

impl ManualBlock {
    pub const fn text(content: &'static str) -> Self {
        Self::new(BlockBody::Text(content))
    }

    pub const fn callout(tone: CalloutTone, content: &'static str) -> Self {
        Self::new(BlockBody::Callout {
            tone,
            title: None,
            content,
        })
    }

    pub const fn list(items: &'static [&'static str]) -> Self {
        Self::new(BlockBody::List {
            ordered: false,
            items,
        })
    }

    pub const fn steps(items: &'static [&'static str]) -> Self {
        Self::new(BlockBody::List {
            ordered: true,
            items,
        })
    }

    pub const fn checklist(items: &'static [ChecklistItem]) -> Self {
        Self::new(BlockBody::Checklist { title: None, items })
    }

    pub const fn table(
        columns: &'static [&'static str],
        rows: &'static [&'static [&'static str]],
    ) -> Self {
        Self::new(BlockBody::Table {
            title: None,
            columns,
            rows,
            dense: false,
        })
    }

    pub const fn new(body: BlockBody) -> Self {
        Self {
            applies_to: &[],
            body,
        }
    }

    /// Sets the title of a callout, checklist, table or figure.
    pub const fn titled(mut self, heading: &'static str) -> Self {
        match &mut self.body {
            BlockBody::Callout { title, .. }
            | BlockBody::Checklist { title, .. }
            | BlockBody::Table { title, .. }
            | BlockBody::Figure { title, .. } => *title = Some(heading),
            BlockBody::Text(_) | BlockBody::List { .. } => {}
        }
        self
    }

    pub const fn only(mut self, aircraft: &'static [AircraftName]) -> Self {
        self.applies_to = aircraft;
        self
    }
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

impl ManualSection {
    pub const fn new(title: &'static str) -> Self {
        Self {
            slug: None,
            title,
            summary: None,
            applies_to: &[],
            blocks: &[],
            children: &[],
        }
    }

    /// A section with a pinned slug.
    pub const fn at(slug: &'static str, title: &'static str) -> Self {
        let mut section = Self::new(title);
        section.slug = Some(slug);
        section
    }

    pub const fn summary(mut self, summary: &'static str) -> Self {
        self.summary = Some(summary);
        self
    }

    pub const fn only(mut self, aircraft: &'static [AircraftName]) -> Self {
        self.applies_to = aircraft;
        self
    }

    pub const fn blocks(mut self, blocks: &'static [ManualBlock]) -> Self {
        self.blocks = blocks;
        self
    }

    pub const fn children(mut self, children: &'static [ManualSection]) -> Self {
        self.children = children;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManualChapter {
    /// URL segment, e.g. "normal-procedures". Unique within the manual.
    pub slug: &'static str,
    pub title: &'static str,
    pub summary: Option<&'static str>,
    pub applies_to: &'static [AircraftName],
    pub sections: &'static [ManualSection],
}

impl ManualChapter {
    pub const fn new(slug: &'static str, title: &'static str) -> Self {
        Self {
            slug,
            title,
            summary: None,
            applies_to: &[],
            sections: &[],
        }
    }

    pub const fn summary(mut self, summary: &'static str) -> Self {
        self.summary = Some(summary);
        self
    }

    pub const fn only(mut self, aircraft: &'static [AircraftName]) -> Self {
        self.applies_to = aircraft;
        self
    }

    pub const fn sections(mut self, sections: &'static [ManualSection]) -> Self {
        self.sections = sections;
        self
    }
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
