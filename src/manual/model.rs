//! Pure helpers over the content model: aircraft filtering, numbering, slugs,
//! links and search. Nothing here touches the DOM.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::types::{AircraftName, BlockBody, Manual, ManualBlock, ManualChapter, ManualSection};

#[derive(Clone, Debug, PartialEq)]
pub struct OutlineSection {
    pub section: &'static ManualSection,
    pub slug: String,
    /// Display number, e.g. "3.2.1", counting only what the selected aircraft
    /// shows. Empty for the main page's introduction.
    pub number: String,
    /// 0 for a chapter's top-level sections.
    pub depth: usize,
    /// The blocks that apply to the selected aircraft.
    pub blocks: Vec<&'static ManualBlock>,
    pub children: Vec<Arc<OutlineSection>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OutlineChapter {
    pub chapter: &'static ManualChapter,
    /// Display number, e.g. "3".
    pub number: String,
    pub sections: Vec<Arc<OutlineSection>>,
    /// Every section, in reading order.
    pub flat: Vec<Arc<OutlineSection>>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OutlineIntro {
    pub sections: Vec<Arc<OutlineSection>>,
    pub flat: Vec<Arc<OutlineSection>>,
}

pub fn slugify(input: &str) -> String {
    let lower = input.trim().to_lowercase().replace('&', " and ");
    let mut out = String::with_capacity(lower.len());
    let mut gap = false;
    for c in lower.chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            if gap && !out.is_empty() {
                out.push('-');
            }
            gap = false;
            out.push(c);
        } else {
            gap = true;
        }
    }
    out
}

/// Whether content tagged `names` shows for `aircraft`. Untagged content shows for all.
pub fn is_for(aircraft: Option<AircraftName>, names: &[AircraftName]) -> bool {
    aircraft.is_none_or(|a| names.is_empty() || names.contains(&a))
}

/// The aircraft a URL value such as "kc-10" names, if the manual covers it.
pub fn find_aircraft(manual: &Manual, value: Option<&str>) -> Option<AircraftName> {
    let wanted = slugify(value.filter(|v| !v.is_empty())?);
    manual
        .aircraft
        .iter()
        .copied()
        .find(|name| slugify(name) == wanted)
}

type Slugs = HashMap<*const ManualSection, String>;

/// The chapters and sections that apply to `aircraft` (all of them when it's
/// `None`), numbered in order. Slugs come from the whole manual, so a link to
/// a section works whichever aircraft is picked.
pub fn build_outline(
    manual: &'static Manual,
    aircraft: Option<AircraftName>,
) -> Vec<OutlineChapter> {
    let groups: Vec<&'static [ManualSection]> =
        manual.chapters.iter().map(|c| c.sections).collect();
    let slugs = assign_slugs(&groups);
    manual
        .chapters
        .iter()
        .filter(|chapter| is_for(aircraft, chapter.applies_to))
        .enumerate()
        .map(|(index, chapter)| {
            let number = (index + 1).to_string();
            let sections = walk(chapter.sections, &number, 0, aircraft, &slugs);
            OutlineChapter {
                chapter,
                number,
                flat: flatten(&sections),
                sections,
            }
        })
        .collect()
}

/// The main page's introduction sections that apply to `aircraft`.
pub fn build_intro(manual: &'static Manual, aircraft: Option<AircraftName>) -> OutlineIntro {
    let slugs = assign_slugs(&[manual.introduction]);
    let sections = walk(manual.introduction, "", 0, aircraft, &slugs);
    OutlineIntro {
        flat: flatten(&sections),
        sections,
    }
}

fn walk(
    sections: &'static [ManualSection],
    prefix: &str,
    depth: usize,
    aircraft: Option<AircraftName>,
    slugs: &Slugs,
) -> Vec<Arc<OutlineSection>> {
    sections
        .iter()
        .filter(|section| is_for(aircraft, section.applies_to))
        .enumerate()
        .map(|(i, section)| {
            let number = if prefix.is_empty() {
                String::new()
            } else {
                format!("{prefix}.{}", i + 1)
            };
            let children = walk(section.children, &number, depth + 1, aircraft, slugs);
            Arc::new(OutlineSection {
                section,
                slug: slugs
                    .get(&std::ptr::from_ref(section))
                    .cloned()
                    .unwrap_or_else(|| slugify(section.title)),
                number,
                depth,
                blocks: section
                    .blocks
                    .iter()
                    .filter(|block| is_for(aircraft, block.applies_to))
                    .collect(),
                children,
            })
        })
        .collect()
}

/// Every node, parents before their children.
fn flatten(nodes: &[Arc<OutlineSection>]) -> Vec<Arc<OutlineSection>> {
    let mut flat = Vec::new();
    for node in nodes {
        flat.push(Arc::clone(node));
        flat.extend(flatten(&node.children));
    }
    flat
}

/// A slug for every section, unique within its group (a chapter, or the
/// introduction). An automatic slug that collides takes its parent's slug as
/// a prefix ("electrical-overview"); anything that still collides gets a
/// counter.
fn assign_slugs(groups: &[&'static [ManualSection]]) -> Slugs {
    fn visit(
        sections: &'static [ManualSection],
        parent: Option<&str>,
        used: &mut HashSet<String>,
        slugs: &mut Slugs,
    ) {
        for section in sections {
            let slug = unique_slug(section, parent, used);
            visit(section.children, Some(&slug), used, slugs);
            slugs.insert(std::ptr::from_ref(section), slug);
        }
    }

    let mut slugs = Slugs::new();
    for group in groups {
        let mut used = HashSet::new();
        visit(group, None, &mut used, &mut slugs);
    }
    slugs
}

fn unique_slug(
    section: &ManualSection,
    parent: Option<&str>,
    used: &mut HashSet<String>,
) -> String {
    let base = section.slug.map_or_else(
        || {
            Some(slugify(section.title))
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "section".to_string())
        },
        str::to_string,
    );
    let mut slug = base.clone();
    if let Some(parent) = parent.filter(|_| used.contains(&slug) && section.slug.is_none()) {
        slug = format!("{parent}-{base}");
    }
    let mut n = 2;
    while used.contains(&slug) {
        slug = format!("{base}-{n}");
        n += 1;
    }
    used.insert(slug.clone());
    slug
}

/// Content mistakes worth fixing: duplicate or non-URL-safe slugs, and
/// `applies_to` names that aren't in `manual.aircraft`.
pub fn find_problems(manual: &'static Manual) -> Vec<String> {
    let mut problems = Vec::new();
    let mut chapter_slugs = HashSet::new();

    if manual.aircraft.is_empty() {
        problems.push("The manual lists no aircraft".to_string());
    }

    let check_aircraft = |problems: &mut Vec<String>, place: &str, names: &[AircraftName]| {
        for name in names {
            if !manual.aircraft.contains(name) {
                problems.push(format!(
                    "{place}: applies_to \"{name}\" isn't one of {}",
                    manual.aircraft.join(", "),
                ));
            }
        }
    };

    let check_sections = |problems: &mut Vec<String>, group: &str, flat: &[Arc<OutlineSection>]| {
        for node in flat {
            let section = node.section;
            let place = format!("\"{group}\" > \"{}\"", section.title);
            if let Some(pinned) = section.slug {
                if pinned != node.slug {
                    problems.push(format!("{place}: slug \"{pinned}\" is already used here"));
                }
                if slugify(pinned) != pinned {
                    problems.push(format!(
                        "{place}: slug \"{pinned}\" should be lowercase words joined by hyphens"
                    ));
                }
            }
            check_aircraft(problems, &place, section.applies_to);
            for block in &node.blocks {
                check_aircraft(problems, &place, block.applies_to);
            }
        }
    };

    check_sections(
        &mut problems,
        "Introduction",
        &build_intro(manual, None).flat,
    );
    for entry in build_outline(manual, None) {
        let chapter = entry.chapter;
        if !chapter_slugs.insert(chapter.slug) {
            problems.push(format!("Chapter slug \"{}\" is used twice", chapter.slug));
        }
        if slugify(chapter.slug) != chapter.slug {
            problems.push(format!(
                "Chapter slug \"{}\" should be lowercase words joined by hyphens",
                chapter.slug
            ));
        }
        check_aircraft(
            &mut problems,
            &format!("\"{}\"", chapter.title),
            chapter.applies_to,
        );
        check_sections(&mut problems, chapter.title, &entry.flat);
    }
    problems
}

/// `{base_path}/{chapter}?aircraft=…#{section}`, leaving out whatever isn't
/// given.
pub fn manual_href(
    base_path: &str,
    chapter: Option<&str>,
    section: Option<&str>,
    aircraft: Option<AircraftName>,
) -> String {
    let base = base_path.trim_end_matches('/');
    let mut href = match chapter.filter(|c| !c.is_empty()) {
        Some(chapter) => format!("{base}/{chapter}"),
        None if base.is_empty() => "/".to_string(),
        None => base.to_string(),
    };
    if let Some(aircraft) = aircraft {
        href.push_str("?aircraft=");
        href.push_str(&slugify(aircraft));
    }
    if let Some(section) = section.filter(|s| !s.is_empty()) {
        href.push('#');
        href.push_str(section);
    }
    href
}

fn has_scheme(target: &str) -> bool {
    let mut chars = target.chars();
    if !chars.next().is_some_and(|c| c.is_ascii_alphabetic()) {
        return false;
    }
    for c in chars {
        if c == ':' {
            return true;
        }
        if !(c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-')) {
            return false;
        }
    }
    false
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedLink {
    pub href: String,
    pub external: bool,
}

/// Resolves a link target written in content. `#section` stays in the current
/// chapter, `chapter` or `chapter#section` points elsewhere in the manual, a
/// leading `/` is a path on the host site, and anything with a scheme
/// (`https:`, `mailto:`) is external.
pub fn resolve_link(
    target: &str,
    base_path: &str,
    chapter_slug: Option<&str>,
    aircraft: Option<AircraftName>,
) -> ResolvedLink {
    if has_scheme(target) || target.starts_with("//") {
        return ResolvedLink {
            href: target.to_string(),
            external: true,
        };
    }
    if target.starts_with('/') {
        return ResolvedLink {
            href: target.to_string(),
            external: false,
        };
    }
    let mut parts = target.split('#');
    let chapter = parts.next().filter(|c| !c.is_empty()).or(chapter_slug);
    ResolvedLink {
        href: manual_href(base_path, chapter, parts.next(), aircraft),
        external: false,
    }
}

/// Resolves a figure `src` against `asset_base` unless it's already absolute.
pub fn resolve_asset(src: &str, asset_base: &str) -> String {
    if asset_base.is_empty() || has_scheme(src) || src.starts_with('/') {
        return src.to_string();
    }
    let path = src.strip_prefix("./").unwrap_or(src);
    format!("{}/{path}", asset_base.trim_end_matches('/'))
}

/// "DC-10 only" for content limited to some aircraft; `None` when it applies to all.
pub fn applies_to_label(manual: &Manual, names: &[AircraftName]) -> Option<String> {
    if names.is_empty() || manual.aircraft.iter().all(|name| names.contains(name)) {
        return None;
    }
    Some(format!("{} only", names.join(" / ")))
}

/// One piece of inline markup: **bold**, `code`, [label](target) and *italic*.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inline<'a> {
    Text(&'a str),
    Strong(&'a str),
    Code(&'a str),
    Link { label: &'a str, target: &'a str },
    Em(&'a str),
}

impl<'a> Inline<'a> {
    /// The visible text.
    pub const fn label(self) -> &'a str {
        match self {
            Inline::Text(text)
            | Inline::Strong(text)
            | Inline::Code(text)
            | Inline::Em(text)
            | Inline::Link { label: text, .. } => text,
        }
    }
}

/// Splits text into plain runs and markup tokens, shared by the renderer and
/// search.
pub fn parse_inline(text: &str) -> Vec<Inline<'_>> {
    let mut out = Vec::new();
    let mut plain = 0;
    let mut at = 0;
    while at < text.len() {
        if let Some((len, token)) = inline_token(&text[at..]) {
            if plain < at {
                out.push(Inline::Text(&text[plain..at]));
            }
            out.push(token);
            at += len;
            plain = at;
        } else {
            at += text[at..].chars().next().map_or(1, char::len_utf8);
        }
    }
    if plain < text.len() {
        out.push(Inline::Text(&text[plain..]));
    }
    out
}

/// The token at the start of `s`, with its length in bytes.
fn inline_token(s: &str) -> Option<(usize, Inline<'_>)> {
    if let Some(rest) = s.strip_prefix("**") {
        let end = rest.find('*').filter(|&end| end > 0)?;
        return rest[end..]
            .starts_with("**")
            .then(|| (end + 4, Inline::Strong(&rest[..end])));
    }
    if let Some(rest) = s.strip_prefix('`') {
        let end = rest.find('`').filter(|&end| end > 0)?;
        return Some((end + 2, Inline::Code(&rest[..end])));
    }
    if let Some(rest) = s.strip_prefix('[') {
        let close = rest.find(']').filter(|&close| close > 0)?;
        let target = rest[close..].strip_prefix("](")?;
        let end = target
            .find(|c: char| c == ')' || c.is_whitespace())
            .filter(|&end| end > 0 && target[end..].starts_with(')'))?;
        return Some((
            1 + close + 2 + end + 1,
            Inline::Link {
                label: &rest[..close],
                target: &target[..end],
            },
        ));
    }
    if let Some(rest) = s.strip_prefix('*') {
        let first = rest
            .chars()
            .next()
            .filter(|c| *c != '*' && !c.is_whitespace())?;
        let end = first.len_utf8() + rest[first.len_utf8()..].find('*')?;
        return Some((end + 2, Inline::Em(&rest[..end])));
    }
    None
}

/// Text with inline markup reduced to what a reader sees.
pub fn plain_text(text: &str) -> String {
    parse_inline(text).into_iter().map(Inline::label).collect()
}

fn join_present<'a>(parts: impl IntoIterator<Item = Option<&'a str>>) -> String {
    parts
        .into_iter()
        .flatten()
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn block_text(block: &ManualBlock) -> String {
    match block.body {
        BlockBody::Text(content) => content.to_string(),
        BlockBody::Callout { title, content, .. } => join_present([title, Some(content)]),
        BlockBody::List { items, .. } => items.join(" "),
        BlockBody::Checklist { title, items } => {
            let rows: Vec<String> = items
                .iter()
                .map(|i| format!("{} {}", i.item, i.action))
                .collect();
            join_present(std::iter::once(title).chain(rows.iter().map(|r| Some(r.as_str()))))
        }
        BlockBody::Table {
            title,
            columns,
            rows,
            ..
        } => join_present(
            std::iter::once(title)
                .chain(columns.iter().map(|c| Some(*c)))
                .chain(rows.iter().flat_map(|row| row.iter().map(|c| Some(*c)))),
        ),
        BlockBody::Figure {
            title,
            alt,
            caption,
            ..
        } => join_present([title, Some(alt), caption]),
    }
}

/// A chapter as search results show it.
#[derive(Clone, Debug, PartialEq)]
pub struct ChapterRef {
    pub slug: &'static str,
    pub number: String,
    pub title: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchEntry {
    /// Missing for the introduction, which lives on the main page.
    pub chapter: Option<ChapterRef>,
    /// Missing for a chapter's own entry.
    pub section: Option<Arc<OutlineSection>>,
    pub title: &'static str,
    pub text: String,
    haystack: String,
}

/// Everything the selected aircraft shows: chapters, their sections, then the introduction.
pub fn build_search_index(
    outline: &[OutlineChapter],
    intro: &[Arc<OutlineSection>],
) -> Vec<SearchEntry> {
    let section_entry = |chapter: Option<ChapterRef>, node: &Arc<OutlineSection>| {
        let raw: Vec<String> =
            std::iter::once(node.section.summary.unwrap_or_default().to_string())
                .chain(node.blocks.iter().map(|b| block_text(b)))
                .collect();
        let text = plain_text(&raw.join(" "))
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        SearchEntry {
            chapter,
            section: Some(Arc::clone(node)),
            title: node.section.title,
            haystack: format!("{} {text}", node.section.title).to_lowercase(),
            text,
        }
    };

    let mut entries = Vec::new();
    for entry in outline {
        let chapter = ChapterRef {
            slug: entry.chapter.slug,
            number: entry.number.clone(),
            title: entry.chapter.title,
        };
        let summary = entry.chapter.summary.unwrap_or_default();
        entries.push(SearchEntry {
            chapter: Some(chapter.clone()),
            section: None,
            title: chapter.title,
            text: summary.to_string(),
            haystack: format!("{} {summary}", chapter.title).to_lowercase(),
        });
        for node in &entry.flat {
            entries.push(section_entry(Some(chapter.clone()), node));
        }
    }
    for node in intro {
        entries.push(section_entry(None, node));
    }
    entries
}

pub fn search_terms(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchHit {
    pub entry: SearchEntry,
    pub snippet: Option<String>,
}

/// Entries containing every term, title matches first, then reading order.
pub fn search_manual(index: &[SearchEntry], terms: &[String]) -> Vec<SearchHit> {
    if terms.is_empty() {
        return Vec::new();
    }
    let mut scored: Vec<(usize, &SearchEntry)> = index
        .iter()
        .filter(|entry| {
            terms
                .iter()
                .all(|term| entry.haystack.contains(term.as_str()))
        })
        .map(|entry| {
            let title = entry.title.to_lowercase();
            (
                terms
                    .iter()
                    .filter(|term| title.contains(term.as_str()))
                    .count(),
                entry,
            )
        })
        .collect();
    scored.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
    scored
        .into_iter()
        .map(|(_, entry)| SearchHit {
            entry: entry.clone(),
            snippet: snippet(&entry.text, terms, 70),
        })
        .collect()
}

fn snippet(text: &str, terms: &[String], radius: usize) -> Option<String> {
    if text.is_empty() {
        return None;
    }
    let lower = text.to_lowercase();
    let at = terms
        .iter()
        .filter_map(|term| lower.find(term.as_str()))
        .map(|byte| lower[..byte].chars().count())
        .min()
        .unwrap_or(0);
    let chars: Vec<char> = text.chars().collect();
    let start = at.saturating_sub(radius).min(chars.len());
    let end = (at + radius * 2).min(chars.len());
    let body: String = chars[start..end].iter().collect();
    Some(format!(
        "{}{}{}",
        if start > 0 { "…" } else { "" },
        body.trim(),
        if end < chars.len() { "…" } else { "" },
    ))
}

/// Splits `text` into runs, flagging those that match a search term.
pub fn highlight<'a>(text: &'a str, terms: &[String]) -> Vec<(bool, &'a str)> {
    let lower = text.to_lowercase();
    // Lowercasing can change byte lengths outside ASCII; don't highlight then.
    if lower.len() != text.len() || terms.is_empty() {
        return vec![(false, text)];
    }
    let mut out = Vec::new();
    let mut plain = 0;
    let mut at = 0;
    while at < text.len() {
        if !text.is_char_boundary(at) {
            at += 1;
            continue;
        }
        let hit = terms
            .iter()
            .find(|term| !term.is_empty() && lower[at..].starts_with(term.as_str()))
            .map(String::len);
        if let Some(len) = hit {
            if plain < at {
                out.push((false, &text[plain..at]));
            }
            out.push((true, &text[at..at + len]));
            at += len;
            plain = at;
        } else {
            at += 1;
        }
    }
    if plain < text.len() {
        out.push((false, &text[plain..]));
    }
    out
}
