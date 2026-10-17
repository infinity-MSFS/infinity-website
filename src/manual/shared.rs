use infinity_ui::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::MouseEvent;

use super::model::{self, ResolvedLink};
use super::types::{AircraftName, Manual};

/// Glass panel behind headers, sections and cards.
pub const PANEL: &str = "rounded-[1.75rem] border border-white/10 bg-white/[0.045] shadow-[0_12px_40px_rgba(0,0,0,0.28)] backdrop-blur-[16px]";
/// Opaque, so light text behind a popover doesn't show through.
pub const POPOVER: &str =
    "border border-white/10 bg-[rgb(6,14,16)] shadow-[0_16px_48px_rgba(0,0,0,0.5)]";
pub const FOCUS: &str = "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-400";
/// Bar controls: the chapter menu button and the icon buttons.
pub const CONTROL: &str =
    "h-10 rounded-2xl border text-sm text-white transition-colors duration-150";
pub const CONTROL_IDLE: &str =
    "border-white/10 bg-black/20 hover:border-white/20 hover:bg-white/[0.06]";
pub const CONTROL_OPEN: &str = "border-accent-400/30 bg-accent-400/[0.08]";

/// Lucide `list`.
pub const LIST: IconData = IconData::new(
    r#"<path d="M3 6h.01"/><path d="M3 12h.01"/><path d="M3 18h.01"/><path d="M8 6h13"/><path d="M8 12h13"/><path d="M8 18h13"/>"#,
);
/// Lucide `octagon-alert`.
pub const OCTAGON_ALERT: IconData = IconData::new(
    r#"<path d="M15.31 2a2 2 0 0 1 1.42.59l4.68 4.68A2 2 0 0 1 22 8.69v6.62a2 2 0 0 1-.59 1.42l-4.68 4.68a2 2 0 0 1-1.42.59H8.69a2 2 0 0 1-1.42-.59l-4.68-4.68A2 2 0 0 1 2 15.31V8.69a2 2 0 0 1 .59-1.42l4.68-4.68A2 2 0 0 1 8.69 2z"/><path d="M12 8v4"/><path d="M12 16h.01"/>"#,
);

/// At most one popover is open at a time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Panel {
    Chapters,
    Sections,
    Search,
}

/// What every part of the reader needs, provided by `ManualReader`.
#[derive(Clone, Copy)]
pub struct ManualContext {
    pub manual: &'static Manual,
    /// The aircraft whose content is showing.
    pub aircraft: Memo<Option<AircraftName>>,
    /// What readers call a chapter, e.g. "Section".
    pub chapter_label: &'static str,
    pub base_path: StoredValue<String>,
    pub asset_base: StoredValue<String>,
    pub chapter_slug: Signal<Option<String>>,
    pub follow: Callback<(MouseEvent, String)>,
}

impl ManualContext {
    /// Href for the main page, a chapter, or a section, for the shown aircraft.
    pub fn href(&self, chapter: Option<&str>, section: Option<&str>) -> String {
        let aircraft = self.aircraft.get();
        self.base_path
            .with_value(|base| model::manual_href(base, chapter, section, aircraft))
    }

    pub fn resolve_link(&self, target: &str) -> ResolvedLink {
        let aircraft = self.aircraft.get();
        let chapter = self.chapter_slug.get();
        self.base_path
            .with_value(|base| model::resolve_link(target, base, chapter.as_deref(), aircraft))
    }

    pub fn resolve_asset(&self, src: &str) -> String {
        self.asset_base
            .with_value(|base| model::resolve_asset(src, base))
    }

    /// Badge text for content limited to some aircraft, e.g. "DC-10 only".
    pub fn applies_to(&self, names: &[AircraftName]) -> Option<String> {
        model::applies_to_label(self.manual, names)
    }

    /// Attach to every internal link with `on:click` so navigation stays
    /// client-side. Events aren't delegated in this build, so the link's own
    /// listener runs before the router's window-level anchor handler and can
    /// claim the click.
    pub fn on_link(self, href: String) -> impl Fn(MouseEvent) + use<> {
        let follow = self.follow;
        move |ev| follow.run((ev, href.clone()))
    }
}

pub fn use_manual() -> ManualContext {
    expect_context::<ManualContext>()
}

/// While `open`, a press outside everything `inside` accepts calls `on_dismiss`.
pub fn use_dismiss(
    open: impl Fn() -> bool + 'static,
    on_dismiss: impl Fn() + 'static,
    inside: impl Fn(&web_sys::Node) -> bool + 'static,
) {
    let handle = window_event_listener(leptos::ev::pointerdown, move |ev| {
        if !open() {
            return;
        }
        let Some(target) = ev.target().and_then(|t| t.dyn_into::<web_sys::Node>().ok()) else {
            return;
        };
        if !inside(&target) {
            on_dismiss();
        }
    });
    on_cleanup(move || handle.remove());
}

/// Whether `node` sits inside the element behind `el`.
pub fn contains(el: Option<impl AsRef<web_sys::Node>>, node: &web_sys::Node) -> bool {
    el.is_some_and(|el| el.as_ref().contains(Some(node)))
}

/// Moves focus through a list's links for the arrow, Home and End keys.
pub fn move_focus(links: &[web_sys::HtmlElement], key: &str) -> bool {
    let active = document().active_element();
    let at = links
        .iter()
        .position(|link| active.as_ref().is_some_and(|a| a.is_same_node(Some(link))));
    let last = links.len().saturating_sub(1);
    let next = match key {
        "ArrowDown" => at.map_or(0, |i| (i + 1).min(last)),
        "ArrowUp" => at.map_or(0, |i| i.saturating_sub(1)),
        "Home" => 0,
        "End" => last,
        _ => return false,
    };
    if let Some(link) = links.get(next) {
        let _ = link.focus();
    }
    true
}

/// The links inside `el`, in document order.
pub fn links_in(el: Option<impl AsRef<web_sys::Element>>) -> Vec<web_sys::HtmlElement> {
    let Some(list) = el.and_then(|el| el.as_ref().query_selector_all("a").ok()) else {
        return Vec::new();
    };
    (0..list.length())
        .filter_map(|i| list.item(i))
        .filter_map(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
        .collect()
}
