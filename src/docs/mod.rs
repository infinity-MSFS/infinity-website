use infinity_ui::prelude::*;

mod api_overview;
mod build_tools;
mod charts;
mod comm_bus;
mod events;
mod flow;
mod gauges_systems;
mod getting_started;
mod io;
mod map_view;
mod network;
mod nvg;
mod page;
mod planned_route;
mod prose;
mod simconnect;
mod vars;
mod vfx;

pub use page::DocArticle;

const CATEGORY_ORDER: [&str; 3] = ["Guide", "Reference", "Modules"];

pub fn library() -> Vec<Doc> {
    vec![
        getting_started::doc(),
        build_tools::doc(),
        api_overview::doc(),
        gauges_systems::doc(),
        vars::doc(),
        comm_bus::doc(),
        flow::doc(),
        events::doc(),
        io::doc(),
        network::doc(),
        charts::doc(),
        planned_route::doc(),
        map_view::doc(),
        vfx::doc(),
        nvg::doc(),
        simconnect::doc(),
    ]
}

pub fn groups(docs: &[Doc]) -> Vec<SelectGroup> {
    CATEGORY_ORDER
        .iter()
        .filter_map(|category| {
            let options: Vec<SelectOption> = docs
                .iter()
                .filter(|doc| doc.category == *category)
                .map(|doc| SelectOption::new(doc.slug.clone(), doc.title.clone()))
                .collect();
            (!options.is_empty()).then(|| SelectGroup::new(*category, options))
        })
        .collect()
}
