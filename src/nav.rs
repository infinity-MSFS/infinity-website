#![allow(dead_code)]

use infinity_ui::TopNavItem;
use infinity_ui::prelude::*;

pub const HOME: &str = "/";
pub const AIRCRAFT: &str = "/aircraft";
pub const MANAGER: &str = "/about";
pub const DOCS: &str = "/docs";
pub const DEVELOPER_DOCS: &str = "/docs/developer";
pub const USER: &str = "/user";
pub const BETA: &str = "/beta";
pub const ATA: &str = "/ata";
pub const BETA_ADMIN: &str = "/beta/admin";
pub const PURCHASE_SUCCESS: &str = "/purchase-success";
pub const PURCHASE_CANCELLED: &str = "/purchase-cancelled";

/// GitHub organisation that hosts everything we open-source.
pub const GITHUB_ORG: &str = "https://github.com/infinity-MSFS";

/// Route for the e-signing page of a single beta document.
pub fn beta_sign(document_id: &str) -> String {
    format!("{BETA}/sign/{document_id}")
}

pub fn top_nav_items() -> Vec<TopNavItem> {
    vec![
        TopNavItem::new("Home", HOME),
        TopNavItem::new("Aircraft", AIRCRAFT),
        TopNavItem::new("Manager", MANAGER),
        TopNavItem::new("Docs", DOCS),
    ]
}

pub fn use_go() -> Callback<String> {
    let navigate = leptos_router::hooks::use_navigate();
    Callback::new(move |to: String| navigate(&to, Default::default()))
}

pub fn go_to(go: Callback<String>, to: impl Into<String>) -> Callback<()> {
    let to = to.into();
    Callback::new(move |()| go.run(to.clone()))
}
