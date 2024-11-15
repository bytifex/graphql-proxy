#![allow(clippy::enum_variant_names)]
#![allow(clippy::duplicated_attributes)]

use async_graphql::Interface;

use crate::model::{
    enums::{augmentation_type::AugmentationType, character_race::CharacterRace},
    scalars::id::Id,
    types::{augmentation::Augmentation, cyborg::Cyborg},
};

#[derive(Interface)]
#[graphql(
    field(name = "id", ty = "&Id"),
    field(name = "name", ty = "&Option<String>"),
    field(name = "nickname", ty = "&String"),
    field(name = "race", ty = "CharacterRace"),
    field(
        name = "augmentations",
        ty = "Vec<Augmentation>",
        arg(name = "type", ty = "AugmentationType"),
    ),
    field(
        name = "augmentations_by_type",
        ty = "Vec<Augmentation>",
        arg(name = "type", ty = "AugmentationType"),
    )
)]
pub enum AugmentedCharacter {
    Cyborg(Cyborg),
}
