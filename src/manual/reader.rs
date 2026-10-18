use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use infinity_ui::prelude::*;
use leptos::html::{Button, Div, Nav};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::ScrollBehavior;

use super::dom::{
    at_scroll_bottom, hash_slug, scroll_margin, scroll_offset, scroll_parent, scroll_to_element,
    scroll_to_offset, smooth_scroll,
};
use super::model::{
    build_intro, build_outline, build_search_index, find_aircraft, find_problems, search_manual,
    search_terms, slugify,
};
use super::pages::{ChapterNotFound, ChapterPage, ChapterUnavailable, MainPage};
use super::shared::{ManualContext, Panel, contains, use_dismiss};
use super::sidebar::Sidebar;
use super::toolbar::Toolbar;
use super::types::{AircraftName, Manual};

/// URL query parameter holding the aircraft being shown, e.g. `?aircraft=kc-10`.
const AIRCRAFT_PARAM: &str = "aircraft";

thread_local! {
    static WARNED: RefCell<HashSet<&'static str>> = RefCell::new(HashSet::new());
    // Where the reader was on each page they left by a link, keyed by URL. Back
    // and forward return there, as they would on an ordinary site. The browser
    // can't do it itself when the host scrolls an inner container rather than
    // the window, as this site does.
    static SAVED_SCROLL: RefCell<HashMap<String, f64>> = RefCell::new(HashMap::new());
}

fn storage_key(manual: &Manual) -> String {
    format!("manual-aircraft:{}", manual.id)
}

fn location() -> web_sys::Location {
    window().location()
}

fn aircraft_from_url(manual: &Manual) -> Option<AircraftName> {
    let search = location().search().ok()?;
    let params = web_sys::UrlSearchParams::new_with_str(&search).ok()?;
    find_aircraft(manual, params.get(AIRCRAFT_PARAM).as_deref())
}

/// The aircraft in the URL, else the one this browser last showed, else the first.
fn initial_aircraft(manual: &Manual) -> Option<AircraftName> {
    aircraft_from_url(manual)
        .or_else(|| {
            // Storage can be unavailable (private windows, blocked cookies).
            let storage = window().local_storage().ok().flatten()?;
            let stored = storage.get_item(&storage_key(manual)).ok().flatten();
            find_aircraft(manual, stored.as_deref())
        })
        .or_else(|| manual.aircraft.first().copied())
}

fn set_url(url: &str, push: bool) {
    let Ok(history) = window().history() else {
        return;
    };
    let state = history.state().unwrap_or(JsValue::NULL);
    let _ = if push {
        history.push_state_with_url(&state, "", Some(url))
    } else {
        history.replace_state_with_url(&state, "", Some(url))
    };
}

/// Calls `f` on `target`'s `event` until the returned function runs.
fn listen(
    target: &web_sys::EventTarget,
    event: &'static str,
    passive: bool,
    f: impl FnMut() + 'static,
) -> Box<dyn FnOnce()> {
    let closure = Closure::<dyn FnMut()>::new(f);
    let options = web_sys::AddEventListenerOptions::new();
    options.set_passive(passive);
    let _ = target.add_event_listener_with_callback_and_add_event_listener_options(
        event,
        closure.as_ref().unchecked_ref(),
        &options,
    );
    let target = target.clone();
    Box::new(move || {
        let _ = target.remove_event_listener_with_callback(event, closure.as_ref().unchecked_ref());
    })
}

/// Renders a manual: contents page, chapters, search and deep links. The host
/// owns routing. It passes the chapter from its URL and, optionally, a
/// navigate function. The reader handles everything inside a chapter itself,
/// reading and writing the URL query and hash.
#[component]
pub fn ManualReader(
    manual: &'static Manual,
    /// Where the manual lives, e.g. "/manual/dc10". Chapters are at `{base_path}/{chapter.slug}`.
    #[prop(into)]
    base_path: String,
    /// The chapter in the URL. `None` for the manual's main page.
    #[prop(into)]
    chapter_slug: Signal<Option<String>>,
    /// Hands navigation to the host's router. Without it, moving between
    /// chapters is a full page load.
    #[prop(into, optional)]
    on_navigate: Option<Callback<String>>,
    /// Base URL for relative figure paths, e.g. "/manual-assets/dc10".
    #[prop(into, optional)]
    asset_base: String,
) -> impl IntoView {
    let root = NodeRef::<Div>::new();
    let sidebar = NodeRef::<Nav>::new();
    let sections_toggle = NodeRef::<Button>::new();
    let toc_scroll = NodeRef::<Div>::new();
    let sections_id = use_id("manual-sections");
    // The section the reader last jumped to. Near the end of a chapter it can't
    // reach the top of the viewport, so the scroll spy defers to it there.
    let pinned = StoredValue::new(None::<String>);
    // The section a link to another chapter points at. The router is handed
    // the path without it, so it doesn't `scrollIntoView()` on arrival.
    let pending = StoredValue::new(None::<String>);
    // Set from a back/forward until the reader has restored its place there.
    let restoring = StoredValue::new(false);
    // A section to keep in view while the content changes to another aircraft.
    let anchor = StoredValue::new(None::<String>);
    // Frames scheduled below can land after the reader is gone.
    let alive = Arc::new(AtomicBool::new(true));
    let active_slug = RwSignal::new(None::<String>);
    let query = RwSignal::new(String::new());
    let panel = RwSignal::new(None::<Panel>);
    let picked = RwSignal::new(initial_aircraft(manual));

    {
        let alive = Arc::clone(&alive);
        on_cleanup(move || alive.store(false, Ordering::Relaxed));
    }
    let next_frame = {
        let alive = Arc::clone(&alive);
        move |f: Box<dyn FnOnce()>| {
            let alive = Arc::clone(&alive);
            request_animation_frame(move || {
                if alive.load(Ordering::Relaxed) {
                    f();
                }
            });
        }
    };

    let aircraft = Memo::new(move |_| {
        picked
            .get()
            .filter(|p| manual.aircraft.contains(p))
            .or_else(|| manual.aircraft.first().copied())
    });
    let outline = Memo::new(move |_| build_outline(manual, aircraft.get()));
    let intro = Memo::new(move |_| build_intro(manual, aircraft.get()));
    let index =
        Memo::new(move |_| outline.with(|o| intro.with(|i| build_search_index(o, &i.flat))));
    let terms = Memo::new(move |_| query.with(|q| search_terms(q)));
    let hits = Memo::new(move |_| index.with(|i| terms.with(|t| search_manual(i, t))));
    let current = Memo::new(move |_| {
        let slug = chapter_slug.get()?;
        outline.with(|o| o.iter().find(|e| e.chapter.slug == slug).cloned())
    });

    if WARNED.with_borrow_mut(|warned| warned.insert(manual.id)) {
        for problem in find_problems(manual) {
            web_sys::console::warn_1(&format!("[manual {}] {problem}", manual.id).into());
        }
    }

    // Keep the aircraft in the URL, so copied and shared links show the same
    // content, and remember it for the next visit. Also re-checks after each
    // chapter change.
    Effect::new(move |_| {
        chapter_slug.track();
        let Some(aircraft) = aircraft.get() else {
            return;
        };
        if let Ok(Some(storage)) = window().local_storage() {
            // Remembering is optional.
            let _ = storage.set_item(&storage_key(manual), aircraft);
        }
        let Ok(url) = web_sys::Url::new(&location().href().unwrap_or_default()) else {
            return;
        };
        let value = slugify(aircraft);
        if url.search_params().get(AIRCRAFT_PARAM).as_deref() == Some(value.as_str()) {
            return;
        }
        url.search_params().set(AIRCRAFT_PARAM, &value);
        set_url(&url.href(), false);
    });

    let change_aircraft = Callback::new(move |next: AircraftName| {
        // Hold the reader's place: after the switch, return to the section
        // they were reading if this aircraft has it.
        let reading = current.with_untracked(Option::is_some);
        anchor.set_value(if reading {
            active_slug.get_untracked()
        } else {
            None
        });
        picked.set(Some(next));
    });

    let go_to_section = move |slug: &str, behavior: ScrollBehavior| -> bool {
        let Some(el) = document().get_element_by_id(slug) else {
            return false;
        };
        if !el.has_attribute("data-manual-section") || !contains(root.get_untracked(), &el) {
            return false;
        }
        pinned.set_value(Some(slug.to_string()));
        active_slug.set(Some(slug.to_string()));
        scroll_to_element(&el, behavior, false);
        true
    };

    let reveal_top = move |behavior: ScrollBehavior| {
        pinned.set_value(None);
        if let Some(root) = root.get_untracked() {
            scroll_to_element(&root, behavior, true);
        }
    };

    // The section in the URL hash, or else the top of the manual.
    let go_to_hash = move |behavior: ScrollBehavior| {
        let slug = hash_slug(&location().hash().unwrap_or_default());
        if slug.is_empty() || !go_to_section(&slug, behavior) {
            reveal_top(behavior);
        }
    };

    let restore_saved = move || -> bool {
        let href = location().href().unwrap_or_default();
        let Some(top) = SAVED_SCROLL.with_borrow(|saved| saved.get(&href).copied()) else {
            return false;
        };
        let Some(root) = root.get_untracked() else {
            return false;
        };
        pinned.set_value(None);
        scroll_to_offset(&root, top);
        true
    };

    // After a chapter renders, jump to the section the link or URL hash names,
    // or else make sure the top of the manual is in view.
    Effect::new({
        let next_frame = next_frame.clone();
        move |_| {
            chapter_slug.track();
            if restoring.get_value() {
                return;
            }
            next_frame(Box::new(move || {
                let Some(slug) = pending.get_value() else {
                    go_to_hash(ScrollBehavior::Instant);
                    return;
                };
                pending.set_value(None);
                if let Ok(url) = web_sys::Url::new(&location().href().unwrap_or_default()) {
                    url.set_hash(&slug);
                    set_url(&url.href(), false);
                }
                if !go_to_section(&slug, ScrollBehavior::Instant) {
                    reveal_top(ScrollBehavior::Instant);
                }
            }));
        }
    });

    // After switching aircraft, return to the section the reader was in. If
    // the new aircraft doesn't have it, show the top of the chapter.
    Effect::watch(
        move || aircraft.get(),
        {
            let next_frame = next_frame.clone();
            move |_, _, _| {
                let Some(slug) = anchor.get_value() else {
                    return;
                };
                anchor.set_value(None);
                next_frame(Box::new(move || {
                    if !go_to_section(&slug, ScrollBehavior::Instant) {
                        reveal_top(ScrollBehavior::Instant);
                    }
                }));
            }
        },
        false,
    );

    // Back and forward return to where the reader was when they left. This
    // waits a frame: by then the router has rendered the entry.
    let popstate = window_event_listener(leptos::ev::popstate, {
        let next_frame = next_frame.clone();
        move |_| {
            if let Some(fresh) = aircraft_from_url(manual) {
                picked.set(Some(fresh));
            }
            restoring.set_value(true);
            next_frame(Box::new(move || {
                restoring.set_value(false);
                if !restore_saved() {
                    go_to_hash(ScrollBehavior::Instant);
                }
            }));
        }
    });
    on_cleanup(move || popstate.remove());

    let follow = {
        let base_path = base_path.clone();
        Callback::new(move |(ev, href): (web_sys::MouseEvent, String)| {
            if ev.default_prevented()
                || ev.button() != 0
                || ev.meta_key()
                || ev.ctrl_key()
                || ev.shift_key()
                || ev.alt_key()
            {
                return;
            }
            let here = location();
            let current_href = here.href().unwrap_or_default();
            let Ok(url) = web_sys::Url::new_with_base(&href, &current_href) else {
                return;
            };
            if url.origin() != here.origin().unwrap_or_default() {
                return;
            }
            ev.prevent_default();
            if let Some(root) = root.get_untracked() {
                let top = scroll_offset(&root);
                SAVED_SCROLL.with_borrow_mut(|saved| saved.insert(current_href.clone(), top));
            }
            // Close any open menu or panel.
            panel.set(None);

            let same_page = url.pathname() == here.pathname().unwrap_or_default()
                && url.search() == here.search().unwrap_or_default();
            if same_page {
                // Move within the page rather than asking the router to re-render.
                if url.href() != current_href {
                    set_url(&url.href(), true);
                }
                let slug = hash_slug(&url.hash());
                if slug.is_empty() || !go_to_section(&slug, smooth_scroll()) {
                    reveal_top(smooth_scroll());
                }
                return;
            }
            let Some(navigate) = on_navigate else {
                let _ = here.assign(&url.href());
                return;
            };
            let path = format!("{}{}", url.pathname(), url.search());
            if url.pathname().starts_with(base_path.as_str()) {
                let slug = hash_slug(&url.hash());
                pending.set_value((!slug.is_empty()).then_some(slug));
                navigate.run(path);
            } else {
                navigate.run(format!("{path}{}", url.hash()));
            }
        })
    };

    // Scroll spy: the active section is the last one whose top has passed the
    // sticky-header line.
    let teardown = StoredValue::new_local(None::<Box<dyn FnOnce()>>);
    let stop_spy = move || {
        if let Some(stop) = teardown.try_update_value(Option::take).flatten() {
            stop();
        }
    };
    Effect::new({
        let next_frame = next_frame.clone();
        move |_| {
            outline.track();
            let reading = current.with(Option::is_some);
            stop_spy();
            if !reading {
                return;
            }
            next_frame(Box::new(move || {
                let Some(root_el) = root.get_untracked() else {
                    return;
                };
                let Ok(nodes) = root_el.query_selector_all("[data-manual-section]") else {
                    return;
                };
                let sections: Vec<web_sys::Element> = (0..nodes.length())
                    .filter_map(|i| nodes.item(i))
                    .filter_map(|n| n.dyn_into().ok())
                    .collect();
                let Some(first) = sections.first().cloned() else {
                    return;
                };
                let scroller = scroll_parent(&first);
                let frame = Rc::new(Cell::new(None::<AnimationFrameRequestHandle>));

                let update = Rc::new({
                    let frame = Rc::clone(&frame);
                    let scroller = scroller.clone();
                    move || {
                        frame.set(None);
                        let view_top = scroller
                            .as_ref()
                            .map_or(0.0, |s| s.get_bounding_client_rect().top());
                        let line = view_top + scroll_margin(&first) + 8.0;
                        let mut active = 0;
                        for (i, el) in sections.iter().enumerate() {
                            if el.get_bounding_client_rect().top() > line {
                                break;
                            }
                            active = i;
                        }
                        if at_scroll_bottom(scroller.as_ref()) {
                            let pin = pinned.get_value();
                            let pinned_at = sections.iter().position(|el| Some(el.id()) == pin);
                            active = match pinned_at {
                                Some(p) if p >= active => p,
                                _ => sections.len() - 1,
                            };
                        }
                        let id = sections[active].id();
                        if active_slug.with_untracked(|a| a.as_deref() != Some(id.as_str())) {
                            active_slug.set(Some(id));
                        }
                    }
                });
                let on_scroll = {
                    let frame = Rc::clone(&frame);
                    let update = Rc::clone(&update);
                    move || {
                        if frame.get().is_none() {
                            let update = Rc::clone(&update);
                            frame.set(request_animation_frame_with_handle(move || update()).ok());
                        }
                    }
                };
                // Only the reader's own scrolling releases the pinned section.
                let release = move || pinned.set_value(None);

                update();
                let win: web_sys::EventTarget = window().into();
                let target: web_sys::EventTarget = scroller.map_or_else(|| win.clone(), Into::into);
                let stops = [
                    listen(&target, "scroll", true, on_scroll.clone()),
                    listen(&win, "resize", false, on_scroll),
                    listen(&win, "wheel", true, release),
                    listen(&win, "touchmove", true, release),
                    listen(&win, "keydown", false, release),
                ];
                teardown.set_value(Some(Box::new(move || {
                    for stop in stops {
                        stop();
                    }
                    if let Some(handle) = frame.take() {
                        handle.cancel();
                    }
                })));
            }));
        }
    });
    on_cleanup(stop_spy);

    // The narrow-screen sections panel closes on a press outside it or Escape.
    let sections_open = move || panel.get_untracked() == Some(Panel::Sections);
    use_dismiss(
        sections_open,
        move || panel.set(None),
        move |node| {
            contains(sidebar.get_untracked(), node)
                || contains(sections_toggle.get_untracked(), node)
        },
    );
    let escape = window_event_listener(leptos::ev::keydown, move |ev| {
        if ev.key() != "Escape" || !sections_open() {
            return;
        }
        panel.set(None);
        if let Some(toggle) = sections_toggle.get_untracked() {
            let _ = toggle.focus();
        }
    });
    on_cleanup(move || escape.remove());

    // Keep the active entry visible in a sections list taller than its panel.
    Effect::new(move |_| {
        let Some(slug) = active_slug.get() else {
            return;
        };
        let Some(list) = toc_scroll.get_untracked() else {
            return;
        };
        if list.scroll_height() <= list.client_height() {
            return;
        }
        let Ok(items) = list.query_selector_all("[data-toc]") else {
            return;
        };
        let item = (0..items.length())
            .filter_map(|i| items.item(i))
            .filter_map(|n| n.dyn_into::<web_sys::Element>().ok())
            .find(|el| el.get_attribute("data-toc").as_deref() == Some(slug.as_str()));
        let Some(item) = item else {
            return;
        };
        let outer = list.get_bounding_client_rect();
        let inner = item.get_bounding_client_rect();
        if inner.top() < outer.top() || inner.bottom() > outer.bottom() {
            let shift = inner.top() - outer.top() - (outer.height() - inner.height()) / 2.0;
            list.set_scroll_top(list.scroll_top() + shift as i32);
        }
    });

    provide_context(ManualContext {
        manual,
        aircraft,
        chapter_label: manual.chapter_label.unwrap_or("Chapter"),
        base_path: StoredValue::new(base_path),
        asset_base: StoredValue::new(asset_base),
        chapter_slug,
        follow,
    });

    // A chapter can exist but not apply to the aircraft picked.
    let hidden = move || {
        let slug = chapter_slug.get()?;
        manual.chapters.iter().find(|c| c.slug == slug)
    };
    let menu_label = Signal::derive(move || {
        if chapter_slug.with(Option::is_none) {
            return "Overview".to_string();
        }
        if let Some(entry) = current.get() {
            return format!("{}. {}", entry.number, entry.chapter.title);
        }
        hidden().map_or_else(|| "Not found".to_string(), |c| c.title.to_string())
    });

    let page = move || {
        if chapter_slug.with(Option::is_none) {
            return view! { <MainPage outline=outline.get() intro=intro.get().sections /> }
                .into_any();
        }
        if let Some(entry) = current.get() {
            return view! { <ChapterPage outline=outline.get() entry=entry /> }.into_any();
        }
        match hidden() {
            Some(chapter) => view! { <ChapterUnavailable chapter=chapter /> }.into_any(),
            None => view! { <ChapterNotFound /> }.into_any(),
        }
    };

    let contents_id = sections_id.clone();
    let contents = move || {
        current
            .get()
            .filter(|c| !c.sections.is_empty())
            .map(|chapter| {
                view! {
                    <Sidebar
                        id=contents_id.clone()
                        chapter=chapter
                        active_slug=active_slug
                        open=Signal::derive(move || panel.get() == Some(Panel::Sections))
                        nav_ref=sidebar
                        scroll_ref=toc_scroll
                    />
                }
            })
    };

    view! {
        <div
            node_ref=root
            class=cn!(
                "manual relative text-white/70",
                (manual.aircraft.len() > 1).then_some("manual-aircraft"),
            )
        >
            <Toolbar
                outline=outline
                current=current
                menu_label=menu_label
                on_main_page=Signal::derive(move || chapter_slug.with(Option::is_none))
                on_aircraft_change=change_aircraft
                panel=panel
                query=query
                terms=terms
                hits=hits
                sections_id=sections_id.clone()
                sections_toggle=sections_toggle
            />
            <div class="mx-auto flex w-full max-w-7xl flex-col gap-10 px-4 pt-10 pb-16 md:px-8 md:pt-14 md:pb-24 min-[1100px]:flex-row min-[1100px]:items-start min-[1100px]:gap-14">
                {contents}
                <div class="flex min-w-0 flex-1 flex-col gap-16 md:gap-20">{page}</div>
            </div>
        </div>
    }
}
