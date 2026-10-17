//! Scrolling helpers. The reader can't use `scrollIntoView()`: it scrolls every
//! scrollable ancestor, including overflow-hidden wrappers a host may have
//! around the page (this site's animated background is one). These scroll only
//! the element's real scroll container, which on this site is the app shell's
//! div rather than the window.

use web_sys::{Element, ScrollBehavior, ScrollToOptions};

fn computed(el: &Element, property: &str) -> String {
    window()
        .get_computed_style(el)
        .ok()
        .flatten()
        .and_then(|style| style.get_property_value(property).ok())
        .unwrap_or_default()
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no window")
}

pub fn scroll_parent(el: &Element) -> Option<Element> {
    let document = window().document()?;
    let body = document.body();
    let root = document.document_element();
    let mut node = el.parent_element();
    while let Some(current) = node {
        let is_body = body
            .as_ref()
            .is_some_and(|b| b.is_same_node(Some(&current)));
        let is_root = root
            .as_ref()
            .is_some_and(|r| r.is_same_node(Some(&current)));
        if is_body || is_root {
            break;
        }
        let overflow = computed(&current, "overflow-y");
        if (overflow == "auto" || overflow == "scroll")
            && current.scroll_height() > current.client_height()
        {
            return Some(current);
        }
        node = current.parent_element();
    }
    None
}

/// How far the element's scroll container (or the window) is scrolled.
pub fn scroll_offset(el: &Element) -> f64 {
    scroll_parent(el).map_or_else(
        || window().scroll_y().unwrap_or_default(),
        |parent| f64::from(parent.scroll_top()),
    )
}

fn scroll_to(container: Option<&Element>, top: f64, behavior: ScrollBehavior) {
    let options = ScrollToOptions::new();
    options.set_top(top);
    options.set_behavior(behavior);
    match container {
        Some(container) => container.scroll_to_with_scroll_to_options(&options),
        None => window().scroll_to_with_scroll_to_options(&options),
    }
}

pub fn scroll_to_offset(el: &Element, top: f64) {
    scroll_to(scroll_parent(el).as_ref(), top, ScrollBehavior::Instant);
}

/// The element's CSS `scroll-margin-top`, i.e. the space a sticky header needs.
pub fn scroll_margin(el: &Element) -> f64 {
    computed(el, "scroll-margin-top")
        .trim_end_matches("px")
        .parse()
        .unwrap_or_default()
}

/// Scrolls so `el` sits `scroll-margin-top` below the top of its scroll
/// container. With `only_up`, does nothing unless that means scrolling up,
/// which is how "show the top of the manual" avoids nudging a fresh page.
pub fn scroll_to_element(el: &Element, behavior: ScrollBehavior, only_up: bool) {
    let parent = scroll_parent(el);
    let current = parent.as_ref().map_or_else(
        || window().scroll_y().unwrap_or_default(),
        |p| f64::from(p.scroll_top()),
    );
    let offset = el.get_bounding_client_rect().top()
        - parent
            .as_ref()
            .map_or(0.0, |p| p.get_bounding_client_rect().top());
    let top = (current + offset - scroll_margin(el)).max(0.0);
    if only_up && top >= current {
        return;
    }
    scroll_to(parent.as_ref(), top, behavior);
}

pub fn at_scroll_bottom(scroller: Option<&Element>) -> bool {
    let fallback = window()
        .document()
        .and_then(|d| d.scrolling_element().or_else(|| d.document_element()));
    let Some(el) = scroller.cloned().or(fallback) else {
        return false;
    };
    el.scroll_height() > el.client_height() + 2
        && el.scroll_top() + el.client_height() >= el.scroll_height() - 2
}

pub fn smooth_scroll() -> ScrollBehavior {
    let reduced = window()
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()
        .flatten()
        .is_some_and(|query| query.matches());
    if reduced {
        ScrollBehavior::Instant
    } else {
        ScrollBehavior::Smooth
    }
}

/// A URL hash (`#engine-start`) as a section slug.
pub fn hash_slug(hash: &str) -> String {
    let raw = hash.strip_prefix('#').unwrap_or(hash);
    js_sys::decode_uri_component(raw)
        .ok()
        .and_then(|decoded| decoded.as_string())
        .unwrap_or_else(|| raw.to_string())
}
