use infinity_ui::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, AircraftProduct, ManagerLatest, Platforms};
use crate::auth::use_auth;

pub struct Remote<T: Send + Sync + 'static> {
    pub value: RwSignal<Option<T>>,
    pub error: RwSignal<Option<String>>,
}

impl<T: Send + Sync + 'static> Clone for Remote<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Send + Sync + 'static> Copy for Remote<T> {}

impl<T: Send + Sync + 'static> Remote<T> {
    fn new() -> Self {
        Self {
            value: RwSignal::new(None),
            error: RwSignal::new(None),
        }
    }

    pub fn loading(&self) -> Signal<bool> {
        let value = self.value;
        let error = self.error;
        Signal::derive(move || value.with(Option::is_none) && error.with(Option::is_none))
    }
}

#[derive(Clone, Copy)]
pub struct Catalog(pub Remote<Vec<AircraftProduct>>);

impl Catalog {
    pub fn listing(&self) -> Signal<Vec<AircraftProduct>> {
        let value = self.0.value;
        let auth = use_auth();
        Signal::derive(move || {
            let staff = auth.entitlements.get().is_developer();
            value.with(|products| {
                products
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .filter(|p| staff || (p.storefront && !p.dev_only))
                    .cloned()
                    .collect()
            })
        })
    }

    pub fn by_slug(&self, slug: &str) -> Option<AircraftProduct> {
        self.0.value.with(|products| {
            products
                .as_deref()
                .unwrap_or_default()
                .iter()
                .find(|p| p.slug == slug)
                .cloned()
        })
    }
}

#[derive(Clone, Copy)]
pub struct Manager(pub Remote<ManagerLatest>);

impl Manager {
    pub fn platforms(&self) -> Signal<Platforms> {
        let value = self.0.value;
        Signal::derive(move || {
            value.with(|m| m.as_ref().map(|m| m.platforms.clone()).unwrap_or_default())
        })
    }

    pub fn version(&self) -> Signal<Option<String>> {
        let value = self.0.value;
        Signal::derive(move || value.with(|m| m.as_ref().map(|m| m.version.clone())))
    }
}

pub fn provide_remote_state() {
    let catalog = Catalog(Remote::new());
    let manager = Manager(Remote::new());
    provide_context(catalog);
    provide_context(manager);

    let auth = use_auth();

    Effect::new(move |_| {
        if auth.loading.get() {
            return;
        }
        let authenticated = auth.is_authenticated.get();
        spawn_local(async move {
            let token = if authenticated {
                auth.token().await
            } else {
                None
            };
            match api::website_catalog(token.as_deref()).await {
                Ok(feed) => {
                    catalog.0.error.set(None);
                    catalog.0.value.set(Some(feed.products));
                }
                Err(err) => {
                    if catalog.0.value.with_untracked(Option::is_none) {
                        catalog.0.error.set(Some(err));
                    }
                }
            }
        });
    });

    spawn_local(async move {
        match api::manager_latest().await {
            Ok(release) => manager.0.value.set(Some(release)),
            Err(err) => manager.0.error.set(Some(err)),
        }
    });
}

pub fn use_catalog() -> Catalog {
    expect_context::<Catalog>()
}

pub fn use_manager() -> Manager {
    expect_context::<Manager>()
}
