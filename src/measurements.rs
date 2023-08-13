use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
#[repr(u16)]
pub enum UnitInfo {
    Kilogram,
    Gram,
    Miligram,
    Microgram,
    Nanogram,
    Picogram,

    Pound,

    Meter,
    Inch,
    Foot,

    Percent,
    PerMille,

    Millilitre,
    Microlitre,
    Nanolitre,
    Picolitre,
    Femtolitre,

    /// thousands per microliter
    KpMicrolitre,
    /// millions per microliter
    MpMicrolitre,
    /// units per liter
    UpLitre,
    /// unites per mililiter
    UpMillilitre,

    GrampDecilitre,
    MilligrampDecilitre,
    MilligrampLitre,
    NanogrampMillilitre,

    /// Acidity, "potential hydgrogen"
    Ph,

    /// ESR
    MillimeterpHour,
}
/*
impl UnitInfo {
    fn simple_desc(&self) -> &'static str {
        match self {
            UnitInfo::Kilogram => "kg",
            UnitInfo::Gram => "g",
            UnitInfo::Miligram => "mg",
            UnitInfo::Microgram => "µg",
            UnitInfo::Nanogram => "ng",
            UnitInfo::Picogram => "pg",
            UnitInfo::Pound => "lb.",
            UnitInfo::Meter => "m",
            UnitInfo::Inch => "in.",
            UnitInfo::Foot => "ft.",
            UnitInfo::Percent => "%",
            UnitInfo::PerMille => "‰",
            UnitInfo::Millilitre => "ml",
            UnitInfo::Microlitre => "µl",
            UnitInfo::Nanolitre => "nl",
            UnitInfo::Picolitre => "pl",
            UnitInfo::Femtolitre => "fl",
            UnitInfo::KpMicrolitre => "k/µl",
            UnitInfo::MpMicrolitre => "M/µl",
            UnitInfo::UpLitre => "1/l",
            UnitInfo::UpMillilitre => "1/ml",
            UnitInfo::GrampDecilitre => "g/dl",
            UnitInfo::MilligrampDecilitre => "mg/dl",
            UnitInfo::MilligrampLitre => "mg/l",
            UnitInfo::NanogrampMillilitre => "ng/ml",
            UnitInfo::Ph => "pH",
            UnitInfo::MillimeterpHour => "mm/h",
        }
    }
}
*/

#[derive(Serialize, Deserialize, Clone)]
pub struct MeasurementDescription {
    id: u32,
    name: String,
    description: String,
    units: Vec<UnitInfo>,
}
impl PartialEq for MeasurementDescription {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for MeasurementDescription {}
impl std::hash::Hash for MeasurementDescription {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Measurement {
    datetime: u64,
    measurement: u32,
    value: f64,
    unit: UnitInfo,
    upper_bound: Option<f64>,
    lower_bound: Option<f64>,
}
