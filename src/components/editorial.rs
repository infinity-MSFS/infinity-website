//! Shared pieces of the editorial page style: mono eyebrows, numbered section
//! labels, and the accent hairline that draws across a row on hover.

use infinity_ui::prelude::*;

pub const EYEBROW: &str = "font-mono text-[11px] tracking-[0.25em] text-white/50 uppercase";

/// Mono label followed by a hairline that runs to the edge of the column.
#[component]
pub fn SectionLabel(num: &'static str, label: &'static str) -> impl IntoView {
    view! {
        <div class=cn!("mb-8 flex items-center gap-4", EYEBROW)>
            <span class="text-accent-400/80">{num}</span>
            <span>{label}</span>
            <span class="h-px flex-1 bg-white/10"></span>
        </div>
    }
}

/// Accent line along the top edge that grows on hover. The parent needs
/// `group relative`.
#[component]
pub fn HoverRule() -> impl IntoView {
    view! {
        <span class="absolute -top-px left-0 h-px w-0 bg-accent-400 transition-all duration-300 group-hover:w-full"></span>
    }
}
