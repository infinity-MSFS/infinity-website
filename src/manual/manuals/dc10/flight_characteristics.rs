use crate::manual::types::{ManualChapter, ManualSection};

pub const FLIGHT_CHARACTERISTICS: ManualChapter =
    ManualChapter::new("flight-characteristics", "Flight Characteristics")
        .summary("How the aircraft handles across its flight envelope.")
        .sections(&[
            ManualSection::at("general", "General"),
            ManualSection::at("stalls", "Stalls"),
            ManualSection::at("engine-out", "Engine-out handling"),
            ManualSection::at("high-altitude", "High-altitude flight"),
            ManualSection::at("heavy-weight", "Handling at heavy weights"),
        ]);
