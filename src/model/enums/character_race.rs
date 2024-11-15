use std::str::FromStr;

use async_graphql::Enum;
use error::InvalidCharacterRaceError;

#[derive(Debug, Enum, Copy, Clone, Eq, PartialEq)]
pub enum CharacterRace {
    Human,
    Android,
    Cyborg,
}

impl std::fmt::Display for CharacterRace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<&str>::into(*self))
    }
}

impl FromStr for CharacterRace {
    type Err = InvalidCharacterRaceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Android" => Ok(CharacterRace::Android),
            "Cyborg" => Ok(CharacterRace::Cyborg),
            "Human" => Ok(CharacterRace::Human),
            _ => Err(InvalidCharacterRaceError(s.into())),
        }
    }
}

impl From<CharacterRace> for &str {
    fn from(value: CharacterRace) -> Self {
        match value {
            CharacterRace::Android => "Android",
            CharacterRace::Cyborg => "Cyborg",
            CharacterRace::Human => "Human",
        }
    }
}

impl From<CharacterRace> for String {
    fn from(value: CharacterRace) -> Self {
        <&str>::from(value).into()
    }
}
pub mod error {
    #[derive(Clone, Debug, thiserror::Error)]
    #[error("InvalidCharacterRaceError: '{0}'")]
    pub struct InvalidCharacterRaceError(pub String);
}
