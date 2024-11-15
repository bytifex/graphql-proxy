#![allow(clippy::enum_variant_names)]
#![allow(clippy::duplicated_attributes)]

use async_graphql::Interface;

use crate::model::{
    enums::character_race::CharacterRace,
    scalars::id::Id,
    types::{cyborg::Cyborg, human::Human},
};

use super::augmented_character::AugmentedCharacter;

#[derive(Interface)]
#[graphql(
    field(name = "id", ty = "&Id"),
    field(name = "name", ty = "&Option<String>",),
    field(name = "nickname", ty = "&String"),
    field(name = "race", ty = "CharacterRace")
)]
pub enum Character {
    // derived interfaces
    AugmentedCharacter(AugmentedCharacter),

    // types
    Cyborg(Cyborg),
    Human(Human),
}
