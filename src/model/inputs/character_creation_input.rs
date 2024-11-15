use async_graphql::InputObject;

use crate::model::enums::character_race::CharacterRace;

#[derive(InputObject)]
pub struct CharacterCreationInput {
    pub race: CharacterRace,
    pub nickname: String,
    pub name: Option<String>,
}
