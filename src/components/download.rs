use infinity_ui::prelude::*;
use infinity_ui::{MenuAction, SplitButton};

use crate::api::{ManagerPlatform, Platforms};
use crate::platform::{detect_os, pick_platform_for, platform_label};

fn menu_action(key: &str, platform: &ManagerPlatform) -> MenuAction {
    MenuAction::new(key, platform_label(key))
        .href(platform.url.clone())
        .detail(platform.version.clone())
        .icon(icons::DOWNLOAD)
}

#[component]
pub fn DownloadButton(
    #[prop(into)] platforms: Signal<Platforms>,
    #[prop(into, default = Signal::from(false))] loading: Signal<bool>,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let os = detect_os();
    let primary_key = Signal::derive(move || platforms.with(|p| pick_platform_for(os, p)));

    let primary = Signal::derive(move || {
        let key = primary_key.get()?;
        platforms.with(|p| {
            p.get(&key).map(|platform| {
                MenuAction::new(&key, format!("Download for {}", platform_label(&key)))
                    .href(platform.url.clone())
                    .icon(icons::DOWNLOAD)
            })
        })
    });

    let rest = Signal::derive(move || {
        let chosen = primary_key.get();
        platforms.with(|p| {
            p.iter()
                .filter(|(key, _)| Some((*key).as_str()) != chosen.as_deref())
                .map(|(key, platform)| menu_action(key, platform))
                .collect::<Vec<_>>()
        })
    });

    view! {
        <SplitButton
                  primary=primary
                  actions=rest
                  loading=loading
                  loading_label="Finding the latest release…"
                  empty_label="No release published yet"
                  choose_label="Choose a download"
                  class=class
              />
    }
}
