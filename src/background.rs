//! Background palette presets and the `?bg=` preview switch.
//!
//! `BackgroundGradientAnimation` reads every colour off a `BlobPalette`, so a
//! variant is nothing but new values in the same five-blob design. Append
//! `?bg=<name>` to any URL to preview one; the choice is remembered in
//! `localStorage` and a picker drops in so the palettes can be compared against
//! real pages without a rebuild. Neither the picker nor the override exists for
//! a visitor who never passes the parameter.
//!
//! Every preset is dark by design: the page paints white text over a
//! `bg-black/80` sheet, so a light base would leave the copy unreadable.

use infinity_ui::BlobPalette;
use infinity_ui::prelude::*;
use leptos::ev;

const STORAGE_KEY: &str = "infinity.background";
const QUERY_KEY: &str = "bg";

/// Palette shipped to visitors.
pub const DEFAULT: &str = "carbon";

/// Picker sections, in display order. A preset whose `family` is not listed
/// here simply never shows up in the picker, so keep the two in sync.
pub const FAMILIES: &[&str] = &["Neutral", "Blue", "Green", "Purple", "Warm", "Aviation"];

pub struct Preset {
    pub name: &'static str,
    pub label: &'static str,
    /// Picker section this preset is listed under.
    pub family: &'static str,
    /// Shown as the button's title in the picker.
    pub blurb: &'static str,
    pub palette: BlobPalette,
}

/// Geometry is deliberately identical across presets — only the base ramp and
/// the five blob colours move.
const fn palette(
    background_start: &'static str,
    background_end: &'static str,
    colors: [&'static str; 5],
) -> BlobPalette {
    BlobPalette {
        background_start,
        background_end,
        colors,
        size: "80%",
        blend: "hard-light",
    }
}

/// Shorthand so a preset is one readable block rather than fifteen lines.
const fn preset_of(
    name: &'static str,
    label: &'static str,
    family: &'static str,
    blurb: &'static str,
    start: &'static str,
    end: &'static str,
    colors: [&'static str; 5],
) -> Preset {
    Preset {
        name,
        label,
        family,
        blurb,
        palette: palette(start, end, colors),
    }
}

pub const PRESETS: &[Preset] = &[
    // ---- Neutral ---------------------------------------------------------
    preset_of(
        "carbon",
        "Carbon",
        "Neutral",
        "Shipping look — neutral graphite, two emerald blobs for lift.",
        "rgb(24, 24, 27)",
        "rgb(3, 3, 5)",
        [
            "82, 82, 91",
            "16, 185, 129",
            "63, 63, 70",
            "113, 113, 122",
            "45, 212, 191",
        ],
    ),
    preset_of(
        "graphite",
        "Graphite",
        "Neutral",
        "Pure grey, no colour cast at all.",
        "rgb(38, 38, 38)",
        "rgb(10, 10, 10)",
        [
            "115, 115, 115",
            "82, 82, 82",
            "163, 163, 163",
            "64, 64, 64",
            "212, 212, 212",
        ],
    ),
    preset_of(
        "slate",
        "Slate",
        "Neutral",
        "Blue-grey neutrals, a touch warmer than graphite.",
        "rgb(30, 41, 59)",
        "rgb(8, 11, 20)",
        [
            "100, 116, 139",
            "71, 85, 105",
            "148, 163, 184",
            "51, 65, 85",
            "203, 213, 225",
        ],
    ),
    preset_of(
        "ink",
        "Ink",
        "Neutral",
        "Almost pure black with a faint blue bloom.",
        "rgb(9, 12, 24)",
        "rgb(1, 2, 4)",
        [
            "51, 65, 85",
            "30, 64, 175",
            "71, 85, 105",
            "15, 23, 42",
            "100, 116, 139",
        ],
    ),
    preset_of(
        "abyss",
        "Abyss",
        "Neutral",
        "Near-black slate, teal and indigo barely surfacing.",
        "rgb(12, 20, 34)",
        "rgb(2, 6, 14)",
        [
            "45, 212, 191",
            "59, 130, 246",
            "16, 185, 129",
            "99, 102, 241",
            "148, 163, 184",
        ],
    ),
    preset_of(
        "storm",
        "Storm",
        "Neutral",
        "Gunmetal with steel-blue highlights.",
        "rgb(30, 41, 59)",
        "rgb(2, 6, 18)",
        [
            "100, 116, 139",
            "148, 163, 184",
            "56, 189, 248",
            "71, 85, 105",
            "203, 213, 225",
        ],
    ),
    // ---- Blue ------------------------------------------------------------
    preset_of(
        "cobalt",
        "Cobalt",
        "Blue",
        "Saturated blue base, azure blobs.",
        "rgb(12, 34, 98)",
        "rgb(2, 6, 26)",
        [
            "37, 99, 235",
            "56, 189, 248",
            "129, 140, 248",
            "14, 165, 233",
            "191, 219, 254",
        ],
    ),
    preset_of(
        "sapphire",
        "Sapphire",
        "Blue",
        "Royal blue leaning into violet.",
        "rgb(23, 21, 92)",
        "rgb(2, 4, 24)",
        [
            "59, 130, 246",
            "99, 102, 241",
            "139, 92, 246",
            "37, 99, 235",
            "165, 180, 252",
        ],
    ),
    preset_of(
        "glacier",
        "Glacier",
        "Blue",
        "Cold blue and cyan over deep navy.",
        "rgb(8, 47, 73)",
        "rgb(2, 6, 23)",
        [
            "56, 189, 248",
            "125, 211, 252",
            "45, 212, 191",
            "96, 165, 250",
            "226, 232, 240",
        ],
    ),
    preset_of(
        "arctic",
        "Arctic",
        "Blue",
        "Pale ice — the brightest palette here.",
        "rgb(30, 58, 80)",
        "rgb(4, 12, 24)",
        [
            "125, 211, 252",
            "224, 242, 254",
            "94, 234, 212",
            "147, 197, 253",
            "241, 245, 249",
        ],
    ),
    preset_of(
        "deepsea",
        "Deep Sea",
        "Blue",
        "Teal over navy, bioluminescent cyan.",
        "rgb(3, 42, 56)",
        "rgb(1, 8, 18)",
        [
            "6, 182, 212",
            "20, 184, 166",
            "34, 211, 238",
            "13, 148, 136",
            "165, 243, 252",
        ],
    ),
    preset_of(
        "contrail",
        "Contrail",
        "Blue",
        "High-altitude blue with white streaks.",
        "rgb(15, 44, 78)",
        "rgb(2, 6, 20)",
        [
            "226, 232, 240",
            "125, 211, 252",
            "96, 165, 250",
            "241, 245, 249",
            "56, 189, 248",
        ],
    ),
    // ---- Green -----------------------------------------------------------
    preset_of(
        "aurora",
        "Aurora",
        "Green",
        "Emerald base that sits under the accent ramp instead of fighting it.",
        "rgb(6, 78, 59)",
        "rgb(2, 15, 42)",
        [
            "16, 185, 129",
            "45, 212, 191",
            "100, 220, 255",
            "52, 211, 153",
            "125, 211, 252",
        ],
    ),
    preset_of(
        "infinity",
        "Infinity",
        "Green",
        "Brand emerald and white glow on true black.",
        "rgb(4, 32, 26)",
        "rgb(0, 0, 0)",
        [
            "52, 211, 153",
            "255, 255, 255",
            "16, 185, 129",
            "45, 212, 191",
            "110, 231, 183",
        ],
    ),
    preset_of(
        "jade",
        "Jade",
        "Green",
        "Jade and teal with a gold blob.",
        "rgb(6, 61, 51)",
        "rgb(2, 10, 14)",
        [
            "16, 185, 129",
            "45, 212, 191",
            "163, 230, 53",
            "20, 184, 166",
            "250, 204, 21",
        ],
    ),
    preset_of(
        "moss",
        "Moss",
        "Green",
        "Forest green and olive, no blue at all.",
        "rgb(20, 45, 24)",
        "rgb(3, 8, 6)",
        [
            "74, 222, 128",
            "132, 204, 22",
            "22, 163, 74",
            "163, 230, 53",
            "134, 239, 172",
        ],
    ),
    preset_of(
        "spruce",
        "Spruce",
        "Green",
        "Pine green cooled with slate blue.",
        "rgb(12, 45, 45)",
        "rgb(2, 8, 20)",
        [
            "13, 148, 136",
            "56, 189, 248",
            "34, 197, 94",
            "15, 118, 110",
            "148, 163, 184",
        ],
    ),
    preset_of(
        "matrix",
        "Matrix",
        "Green",
        "Black with phosphor green — heavy contrast.",
        "rgb(4, 20, 10)",
        "rgb(0, 3, 1)",
        [
            "34, 197, 94",
            "22, 163, 74",
            "134, 239, 172",
            "16, 185, 129",
            "74, 222, 128",
        ],
    ),
    // ---- Purple ----------------------------------------------------------
    preset_of(
        "violet",
        "Violet",
        "Purple",
        "The original — purple base, warm blobs.",
        "rgb(108, 0, 162)",
        "rgb(0, 17, 82)",
        [
            "18, 113, 255",
            "212, 72, 245",
            "100, 220, 255",
            "235, 115, 55",
            "235, 200, 80",
        ],
    ),
    preset_of(
        "nebula",
        "Nebula",
        "Purple",
        "Cooler, deeper take on the violet — no warm blobs.",
        "rgb(46, 16, 101)",
        "rgb(3, 7, 32)",
        [
            "129, 140, 248",
            "232, 121, 249",
            "56, 189, 248",
            "167, 139, 250",
            "244, 114, 182",
        ],
    ),
    preset_of(
        "orchid",
        "Orchid",
        "Purple",
        "Magenta and violet, high saturation.",
        "rgb(76, 5, 96)",
        "rgb(8, 3, 26)",
        [
            "217, 70, 239",
            "232, 121, 249",
            "168, 85, 247",
            "236, 72, 153",
            "192, 132, 252",
        ],
    ),
    preset_of(
        "plum",
        "Plum",
        "Purple",
        "Deep wine purple, muted.",
        "rgb(59, 7, 46)",
        "rgb(8, 2, 14)",
        [
            "192, 38, 211",
            "168, 85, 247",
            "236, 72, 153",
            "124, 58, 237",
            "244, 114, 182",
        ],
    ),
    preset_of(
        "twilight",
        "Twilight",
        "Purple",
        "Indigo with rose and one warm blob.",
        "rgb(35, 24, 84)",
        "rgb(6, 5, 20)",
        [
            "99, 102, 241",
            "244, 114, 182",
            "56, 189, 248",
            "167, 139, 250",
            "251, 146, 60",
        ],
    ),
    preset_of(
        "synthwave",
        "Synthwave",
        "Purple",
        "Retro magenta and cyan over indigo.",
        "rgb(45, 8, 74)",
        "rgb(6, 2, 20)",
        [
            "236, 72, 153",
            "34, 211, 238",
            "168, 85, 247",
            "244, 63, 94",
            "56, 189, 248",
        ],
    ),
    // ---- Warm ------------------------------------------------------------
    preset_of(
        "ember",
        "Ember",
        "Warm",
        "Warm amber and rose — the emerald accent reads as contrast.",
        "rgb(69, 26, 3)",
        "rgb(12, 10, 20)",
        [
            "251, 146, 60",
            "244, 63, 94",
            "250, 204, 21",
            "217, 70, 239",
            "249, 115, 22",
        ],
    ),
    preset_of(
        "sunset",
        "Sunset",
        "Warm",
        "Orange and pink over deep magenta.",
        "rgb(88, 18, 62)",
        "rgb(10, 6, 24)",
        [
            "251, 113, 133",
            "251, 146, 60",
            "217, 70, 239",
            "250, 204, 21",
            "244, 63, 94",
        ],
    ),
    preset_of(
        "rust",
        "Rust",
        "Warm",
        "Burnt sienna and copper, low saturation.",
        "rgb(62, 26, 12)",
        "rgb(10, 6, 8)",
        [
            "234, 88, 12",
            "180, 83, 9",
            "251, 146, 60",
            "217, 119, 6",
            "245, 158, 11",
        ],
    ),
    preset_of(
        "gold",
        "Gold",
        "Warm",
        "Amber and bronze on near-black.",
        "rgb(48, 33, 4)",
        "rgb(6, 5, 2)",
        [
            "250, 204, 21",
            "245, 158, 11",
            "234, 179, 8",
            "253, 224, 71",
            "217, 119, 6",
        ],
    ),
    preset_of(
        "crimson",
        "Crimson",
        "Warm",
        "Deep red and rose.",
        "rgb(76, 5, 25)",
        "rgb(10, 2, 8)",
        [
            "244, 63, 94",
            "225, 29, 72",
            "251, 113, 133",
            "190, 18, 60",
            "249, 115, 22",
        ],
    ),
    preset_of(
        "magma",
        "Magma",
        "Warm",
        "Black rock with lava breaking through.",
        "rgb(28, 8, 6)",
        "rgb(3, 2, 2)",
        [
            "239, 68, 68",
            "249, 115, 22",
            "250, 204, 21",
            "185, 28, 28",
            "251, 146, 60",
        ],
    ),
    // ---- Aviation --------------------------------------------------------
    preset_of(
        "runway",
        "Runway",
        "Aviation",
        "Asphalt grey with edge-light blue and centreline amber.",
        "rgb(28, 30, 34)",
        "rgb(4, 5, 7)",
        [
            "148, 163, 184",
            "56, 189, 248",
            "250, 204, 21",
            "226, 232, 240",
            "59, 130, 246",
        ],
    ),
    preset_of(
        "avionics",
        "Avionics",
        "Aviation",
        "Dark cockpit lit by PFD cyan, magenta and green.",
        "rgb(6, 20, 26)",
        "rgb(1, 4, 6)",
        [
            "34, 211, 238",
            "217, 70, 239",
            "163, 230, 53",
            "56, 189, 248",
            "250, 204, 21",
        ],
    ),
    preset_of(
        "nightflight",
        "Night Flight",
        "Aviation",
        "Navy with red and green nav lights, white strobe.",
        "rgb(6, 16, 44)",
        "rgb(1, 2, 10)",
        [
            "239, 68, 68",
            "34, 197, 94",
            "59, 130, 246",
            "226, 232, 240",
            "148, 163, 184",
        ],
    ),
    preset_of(
        "stratosphere",
        "Stratosphere",
        "Aviation",
        "Black-blue at altitude with a horizon glow.",
        "rgb(12, 22, 66)",
        "rgb(2, 3, 12)",
        [
            "251, 146, 60",
            "59, 130, 246",
            "129, 140, 248",
            "250, 204, 21",
            "30, 64, 175",
        ],
    ),
];

fn preset(name: &str) -> &'static Preset {
    PRESETS
        .iter()
        .find(|p| p.name == name)
        .or_else(|| PRESETS.iter().find(|p| p.name == DEFAULT))
        .unwrap_or(&PRESETS[0])
}

pub fn palette_for(name: &str) -> BlobPalette {
    preset(name).palette
}

/// Flat two-stop version of a palette, for the low-performance fallback and
/// the picker swatches.
pub fn ramp_for(name: &str) -> String {
    let p = preset(name).palette;
    format!(
        "linear-gradient(40deg, {}, {})",
        p.background_start, p.background_end
    )
}

/// Three blob colours as a strip, so a swatch shows more than the base ramp.
fn blobs_for(name: &str) -> String {
    let [c1, c2, _, c4, _] = preset(name).palette.colors;
    format!("linear-gradient(90deg, rgb({c1}) 0%, rgb({c2}) 50%, rgb({c4}) 100%)")
}

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

/// `?bg=name`, or `Some("")` for a bare `?bg`, which only opens the picker.
fn query_override() -> Option<String> {
    let search = web_sys::window()?.location().search().ok()?;
    search
        .trim_start_matches('?')
        .split('&')
        .filter(|pair| !pair.is_empty())
        .find_map(|pair| match pair.split_once('=') {
            Some((QUERY_KEY, value)) => Some(value.to_string()),
            Some(_) => None,
            None => (pair == QUERY_KEY).then(String::new),
        })
}

#[derive(Clone, Copy)]
pub struct Background {
    pub name: RwSignal<&'static str>,
    /// The picker only mounts for a session that asked for it with `?bg`.
    pub picker: bool,
}

impl Background {
    pub fn select(&self, name: &'static str) {
        self.name.set(name);
        if let Some(store) = storage() {
            let _ = store.set_item(STORAGE_KEY, name);
        }
        // Keeps the URL shareable without pushing a history entry per click.
        if let Some(history) = web_sys::window().and_then(|w| w.history().ok()) {
            let _ = history.replace_state_with_url(
                &wasm_bindgen::JsValue::NULL,
                "",
                Some(&format!("?{QUERY_KEY}={name}")),
            );
        }
    }

    /// Step `delta` presets along the list, wrapping at both ends.
    pub fn step(&self, delta: isize) {
        let current = self.name.get_untracked();
        let index = PRESETS
            .iter()
            .position(|p| p.name == current)
            .unwrap_or_default() as isize;
        let len = PRESETS.len() as isize;
        let next = (index + delta).rem_euclid(len) as usize;
        self.select(PRESETS[next].name);
    }
}

pub fn provide_background() -> Background {
    let requested = query_override();
    let picker = requested.is_some();
    let stored = storage().and_then(|s| s.get_item(STORAGE_KEY).ok().flatten());

    let chosen = requested
        .filter(|name| !name.is_empty())
        .or(stored)
        .unwrap_or_else(|| DEFAULT.to_string());

    let background = Background {
        name: RwSignal::new(preset(&chosen).name),
        picker,
    };
    provide_context(background);
    background
}

pub fn use_background() -> Background {
    use_context::<Background>().expect("Background context is provided by App")
}

/// Floating palette switcher. Mounted only when the URL carried `?bg`.
#[component]
pub fn BackgroundPicker() -> impl IntoView {
    let background = use_background();
    let dismissed = RwSignal::new(false);
    let open = RwSignal::new(true);

    // Arrow keys walk the list, which is the fastest way to judge palettes
    // against a page. Typing in a field still wins.
    let keys = window_event_listener(ev::keydown, move |ev| {
        if dismissed.get_untracked() || ev.ctrl_key() || ev.meta_key() || ev.alt_key() {
            return;
        }
        let typing = document()
            .active_element()
            .map(|el| el.tag_name())
            .is_some_and(|tag| matches!(tag.as_str(), "INPUT" | "TEXTAREA" | "SELECT"));
        if typing {
            return;
        }
        match ev.key().as_str() {
            "ArrowRight" => background.step(1),
            "ArrowLeft" => background.step(-1),
            _ => {}
        }
    });
    on_cleanup(move || keys.remove());

    let current = move || preset(background.name.get());

    view! {
        <Show when=move || !dismissed.get()>
            <div class="pointer-events-none fixed inset-x-0 bottom-4 z-[60] flex justify-center px-4">
                <div class="pointer-events-auto flex w-full max-w-3xl flex-col gap-2 rounded-2xl border border-white/10 bg-black/75 p-3 shadow-lg shadow-black/50 backdrop-blur-xl">
                    <div class="flex items-center gap-3">
                        <span
                            class="h-6 w-10 shrink-0 rounded-md border border-white/15"
                            style=move || {
                                format!("background: {};", ramp_for(background.name.get()))
                            }
                        ></span>
                        <div class="min-w-0 flex-1">
                            <p class="truncate text-sm font-semibold text-white">
                                {move || current().label}
                            </p>
                            <p class="truncate text-[11px] text-white/50">
                                {move || current().blurb}
                            </p>
                        </div>
                        <span class="hidden shrink-0 text-[11px] text-white/35 sm:inline">
                            "← → to cycle"
                        </span>
                        <button
                            type="button"
                            aria-label=move || {
                                if open.get() { "Collapse palettes" } else { "Expand palettes" }
                            }
                            on:click=move |_| open.update(|o| *o = !*o)
                            class="shrink-0 rounded-lg border border-white/10 p-1.5 text-white/50 transition-colors hover:border-white/25 hover:text-white"
                        >
                            <span class=move || {
                                cn!(
                                    "inline-flex transition-transform",
                                    open.get().then_some("rotate-180"),
                                )
                            }>
                                <Icon icon=icons::CHEVRON_DOWN class="h-4 w-4" />
                            </span>
                        </button>
                        <button
                            type="button"
                            aria-label="Hide palette picker"
                            on:click=move |_| dismissed.set(true)
                            class="shrink-0 rounded-lg border border-white/10 p-1.5 text-white/50 transition-colors hover:border-white/25 hover:text-white"
                        >
                            <Icon icon=icons::X class="h-4 w-4" />
                        </button>
                    </div>

                    <Show when=move || open.get()>
                        <div class="flex max-h-[52vh] flex-col gap-3 overflow-y-auto border-t border-white/10 pt-3">
                            {FAMILIES
                                .iter()
                                .map(|family| {
                                    view! {
                                        <div class="flex flex-col gap-1.5">
                                            <span class="text-[10px] font-semibold tracking-[0.16em] text-white/35 uppercase">
                                                {*family}
                                            </span>
                                            <div class="flex flex-wrap gap-1.5">
                                                {PRESETS
                                                    .iter()
                                                    .filter(|p| p.family == *family)
                                                    .map(|preset| {
                                                        let active = Signal::derive(move || {
                                                            background.name.get() == preset.name
                                                        });
                                                        view! {
                                                            <button
                                                                type="button"
                                                                title=preset.blurb
                                                                on:click=move |_| background.select(preset.name)
                                                                class=move || {
                                                                    cn!(
                                                                        "flex shrink-0 items-center gap-2 rounded-xl border px-2 py-1.5 transition-colors",
                                                                        if active.get() {
                                                                            "border-accent-400/60 bg-white/[0.08]"
                                                                        } else {
                                                                            "border-white/10 hover:border-white/25 hover:bg-white/[0.05]"
                                                                        },
                                                                    )
                                                                }
                                                            >
                                                                <span
                                                                    class="h-5 w-5 shrink-0 rounded-md border border-white/15"
                                                                    style=format!(
                                                                        "background: {};",
                                                                        ramp_for(preset.name),
                                                                    )
                                                                ></span>
                                                                <span
                                                                    class="h-5 w-2.5 shrink-0 rounded-sm border border-white/10"
                                                                    style=format!(
                                                                        "background: {};",
                                                                        blobs_for(preset.name),
                                                                    )
                                                                ></span>
                                                                <span class=move || {
                                                                    cn!(
                                                                        "text-[11px] font-medium whitespace-nowrap",
                                                                        if active.get() {
                                                                            "text-white"
                                                                        } else {
                                                                            "text-white/60"
                                                                        },
                                                                    )
                                                                }>{preset.label}</span>
                                                            </button>
                                                        }
                                                    })
                                                    .collect_view()}
                                            </div>
                                        </div>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </Show>
                </div>
            </div>
        </Show>
    }
}
