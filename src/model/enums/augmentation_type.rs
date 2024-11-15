use std::str::FromStr;

use async_graphql::Enum;
use error::InvalidAugmentationTypeError;

#[derive(Debug, Enum, Copy, Clone, Eq, PartialEq)]
pub enum AugmentationType {
    Mechanical,
    BioMechanical,
    GeneticModification,
}

impl std::fmt::Display for AugmentationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<&str>::into(*self))
    }
}

impl FromStr for AugmentationType {
    type Err = InvalidAugmentationTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Mechanical" => Ok(AugmentationType::Mechanical),
            "BioMechanical" => Ok(AugmentationType::BioMechanical),
            "GeneticModification" => Ok(AugmentationType::GeneticModification),
            _ => Err(InvalidAugmentationTypeError(s.into())),
        }
    }
}

impl From<AugmentationType> for &str {
    fn from(value: AugmentationType) -> Self {
        match value {
            AugmentationType::Mechanical => "Mechanical",
            AugmentationType::BioMechanical => "BioMechanical",
            AugmentationType::GeneticModification => "GeneticModification",
        }
    }
}

impl From<AugmentationType> for String {
    fn from(value: AugmentationType) -> Self {
        <&str>::from(value).into()
    }
}
pub mod error {
    #[derive(Clone, Debug, thiserror::Error)]
    #[error("InvalidAugmentationTypeError: '{0}'")]
    pub struct InvalidAugmentationTypeError(pub String);
}
