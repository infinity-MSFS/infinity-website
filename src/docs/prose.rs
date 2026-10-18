use infinity_ui::prelude::*;

#[component]
pub fn Grid2(children: Children) -> impl IntoView {
    view! { <div class="grid gap-x-10 md:grid-cols-2">{children()}</div> }
}

#[component]
pub fn DocCard(
    #[prop(into)] title: String,
    #[prop(into, optional)] eyebrow: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="border-t border-white/10 pt-4 pb-6">
            {eyebrow
                .map(|e| {
                    view! {
                        <div class="mb-2 font-mono text-[11px] tracking-[0.2em] text-accent-400/80 uppercase">
                            {e}
                        </div>
                    }
                })}
            <div class="mb-2 text-base font-medium text-white">{title}</div>
            <div class="text-sm leading-6 font-light text-white/65">{children()}</div>
        </div>
    }
}

#[component]
pub fn Term(children: Children) -> impl IntoView {
    view! { <strong class="font-medium text-white/85">{children()}</strong> }
}
