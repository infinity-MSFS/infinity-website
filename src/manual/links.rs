//! Which web manuals the site offers, and which catalog products link to
//! them. The Flight Manual picker lists these, and the aircraft page reads
//! them to show its Flight Manual card.

use crate::state::Catalog;

pub const MANUAL_BASE_PATH: &str = "/manual";

pub struct SiteManual {
    /// `Manual::id`, which names its file in `public/manual-data/`.
    pub id: &'static str,
    /// The manual's name on the picker, e.g. "DC-10 Series".
    pub name: &'static str,
    /// One line under the name on the picker.
    pub description: &'static str,
    /// The aircraft the manual covers, as the picker lists them.
    pub covers: &'static [&'static str],
    /// Catalog products whose detail page links to this manual, each with the
    /// aircraft the manual opens on. The manual is public once any of them is
    /// on the storefront, and developer-only until then.
    pub products: &'static [ManualProduct],
    /// Still being written. Developer-only even once its aircraft is on the
    /// storefront, "coming soon" on the picker for everyone else, and not
    /// linked from the product page, which keeps pointing at the in-sim EFB.
    pub draft: bool,
}

pub struct ManualProduct {
    pub slug: &'static str,
    pub aircraft: &'static str,
}

pub const SITE_MANUALS: &[SiteManual] = &[
    SiteManual {
        id: "t38",
        name: "T-38A/C Talon Professional",
        description: "The Air Force's supersonic trainer, in its original and avionics-modernized forms.",
        covers: &["T-38A", "T-38C"],
        products: &[ManualProduct {
            slug: "t-38",
            aircraft: "T-38A",
        }],
        draft: true,
    },
    SiteManual {
        id: "t38n",
        name: "T-38N / F-5B Professional",
        description: "NASA's glass-cockpit Talon and the two-seat Freedom Fighter trainer.",
        covers: &["T-38N", "F-5B"],
        products: &[ManualProduct {
            slug: "t-38n",
            aircraft: "T-38N",
        }],
        draft: true,
    },
    SiteManual {
        id: "dc10",
        name: "DC-10 Series",
        description: "One study-level manual for the three-crew DC-10, the KC-10A tanker and the two-crew MD-10.",
        covers: &["DC-10", "KC-10", "MD-10"],
        products: &[
            ManualProduct {
                slug: "dc-10",
                aircraft: "DC-10",
            },
            // Not in the catalog yet; its page links here once it is.
            ManualProduct {
                slug: "kc-10",
                aircraft: "KC-10",
            },
            ManualProduct {
                slug: "md-10",
                aircraft: "MD-10",
            },
        ],
        draft: false,
    },
];

pub fn find_site_manual(id: &str) -> Option<&'static SiteManual> {
    SITE_MANUALS.iter().find(|manual| manual.id == id)
}

pub fn manual_path(id: &str) -> String {
    format!("{MANUAL_BASE_PATH}/{id}")
}

/// `?aircraft=…` opening a manual on one of its aircraft.
pub fn aircraft_query(aircraft: &str) -> String {
    let value = String::from(js_sys::encode_uri_component(&aircraft.to_lowercase()));
    format!("?aircraft={value}")
}

/// Whether one of the manual's aircraft is on the storefront.
pub fn aircraft_on_sale(manual: &SiteManual, catalog: &Catalog) -> bool {
    manual.products.iter().any(|product| {
        catalog
            .by_slug(product.slug)
            .is_some_and(|p| p.storefront && !p.dev_only)
    })
}

/// Whether a manual is public: it isn't a draft, and one of its aircraft is
/// on the storefront (the same rule as the aircraft detail page). Until then
/// only developers may read it. False while the catalog loads.
pub fn is_released(manual: &SiteManual, catalog: &Catalog) -> bool {
    !manual.draft && aircraft_on_sale(manual, catalog)
}

/// The link from a product's page: its manual, opened on its aircraft. None
/// for a draft manual, so the page keeps pointing at the in-sim EFB.
pub fn manual_link_for_product(product_slug: &str) -> Option<String> {
    SITE_MANUALS
        .iter()
        .filter(|manual| !manual.draft)
        .find_map(|manual| {
            let product = manual.products.iter().find(|p| p.slug == product_slug)?;
            Some(format!(
                "{}{}",
                manual_path(manual.id),
                aircraft_query(product.aircraft)
            ))
        })
}
