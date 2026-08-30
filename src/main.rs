mod api;
mod app;
mod auth;
mod brand;
mod components;
mod docs;
mod nav;
mod pages;
mod platform;
mod state;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(app::App);
}
