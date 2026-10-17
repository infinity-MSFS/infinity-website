use super::types::Manual;

pub mod dc10;

/// Every manual the site serves, looked up by `Manual::id`.
pub fn find(id: &str) -> Option<&'static Manual> {
    [&dc10::DC10_MANUAL]
        .into_iter()
        .find(|manual| manual.id == id)
}
