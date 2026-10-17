use crate::manual::types::{ManualChapter, ManualSection};

pub const ADVERSE_WEATHER: ManualChapter =
    ManualChapter::new("adverse-weather", "Adverse Weather Conditions")
        .summary("Operating in cold, heat, icing and turbulence, and on contaminated runways.")
        .sections(&[
            ManualSection::at("cold-weather", "Cold weather"),
            ManualSection::at("hot-weather", "Hot weather"),
            ManualSection::at("icing", "Icing"),
            ManualSection::at("turbulence", "Turbulence and thunderstorms"),
            ManualSection::at("wind-shear", "Wind shear"),
            ManualSection::at("contaminated-runways", "Contaminated runways"),
        ]);
