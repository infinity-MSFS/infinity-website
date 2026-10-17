use crate::manual::types::{ManualChapter, ManualSection};

pub const LIMITATIONS: ManualChapter = ManualChapter::new("limitations", "Limitations")
    .summary("The operating limits of the aircraft and its systems.")
    .sections(&[
        ManualSection::at("general", "General"),
        ManualSection::at("weights", "Weights"),
        ManualSection::at("airspeeds", "Airspeeds"),
        ManualSection::at("engines", "Engines"),
        ManualSection::at("systems", "Systems"),
        ManualSection::at("aerial-refueling", "Aerial refueling").only(&["KC-10"]),
    ]);
