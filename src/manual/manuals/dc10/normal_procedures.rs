use crate::manual::types::{ManualChapter, ManualSection};

pub const NORMAL_PROCEDURES: ManualChapter =
    ManualChapter::new("normal-procedures", "Normal Procedures")
        .summary("A complete flight, from a cold and dark aircraft to shutdown.")
        .sections(&[
            ManualSection::at("cockpit-preparation", "Cockpit preparation"),
            ManualSection::at("engine-start", "Engine start"),
            ManualSection::at("taxi-and-takeoff", "Taxi and takeoff"),
            ManualSection::at("climb-and-cruise", "Climb and cruise"),
            ManualSection::at("aerial-refueling", "Aerial refueling").only(&["KC-10"]),
            ManualSection::at("descent-and-approach", "Descent and approach"),
            ManualSection::at("landing", "Landing"),
            ManualSection::at("shutdown", "Shutdown"),
        ]);
