//! Which catalog products link to which web manual. The aircraft page reads
//! this to show its Flight Manual card.

pub const MANUAL_BASE_PATH: &str = "/manual";

pub struct SiteManual {
    /// `Manual::id` of a manual in `manuals/`.
    pub id: &'static str,
    /// Catalog products whose detail page links to this manual, each with the
    /// aircraft the manual opens on. The manual is public once any of them is
    /// on the storefront, and developer-only until then.
    pub products: &'static [ManualProduct],
}

pub struct ManualProduct {
    pub slug: &'static str,
    pub aircraft: &'static str,
}

pub const SITE_MANUALS: &[SiteManual] = &[SiteManual {
    id: "dc10",
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
}];

pub fn manual_path(id: &str) -> String {
    format!("{MANUAL_BASE_PATH}/{id}")
}

/// The link from a product's page: its manual, opened on its aircraft.
pub fn manual_link_for_product(product_slug: &str) -> Option<String> {
    SITE_MANUALS.iter().find_map(|manual| {
        let product = manual.products.iter().find(|p| p.slug == product_slug)?;
        let aircraft = String::from(js_sys::encode_uri_component(
            &product.aircraft.to_lowercase(),
        ));
        Some(format!("{}?aircraft={aircraft}", manual_path(manual.id)))
    })
}
