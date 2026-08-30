use infinity_ui::prelude::*;

#[component]
pub fn Grid2(children: Children) -> impl IntoView {
    view! { <div class="grid gap-4 md:grid-cols-2">{children()}</div> }
}

#[component]
pub fn DocCard(
    #[prop(into)] title: String,
    #[prop(into, optional)] eyebrow: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="rounded-2xl border border-white/10 bg-white/[0.04] p-5 backdrop-blur-sm">
            {eyebrow
                .map(|e| {
                    view! {
                        <div class="mb-1 text-[11px] tracking-[0.16em] text-accent-300/80 uppercase">
                            {e}
                        </div>
                    }
                })}
            <div class="mb-2 text-base font-semibold text-white">{title}</div>
            <div class="text-sm leading-6 text-white/65">{children()}</div>
        </div>
    }
}

#[component]
pub fn Term(children: Children) -> impl IntoView {
    view! { <strong class="text-white/85">{children()}</strong> }
}
