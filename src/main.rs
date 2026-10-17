mod api;
mod app;
mod auth;
mod background;
mod brand;
mod components;
mod docs;
mod manual;
mod nav;
mod pages;
mod platform;
mod state;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(app::App);
}
