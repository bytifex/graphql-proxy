use async_graphql::Object;

use crate::{
    error::UnimplementedError,
    model::{
        enums::{augmentation_type::AugmentationType, character_race::CharacterRace},
        scalars::id::Id,
    },
};

use super::augmentation::Augmentation;

pub struct Cyborg {
    pub id: Id,
    pub name: Option<String>,
    pub nickname: String,
    pub race: CharacterRace,
}

#[Object]
impl Cyborg {
    pub async fn id(&self) -> &Id {
        &self.id
    }

    pub async fn name(&self) -> &Option<String> {
        &self.name
    }

    pub async fn nickname(&self) -> &String {
        &self.nickname
    }

    pub async fn race(&self) -> CharacterRace {
        self.race
    }

    pub async fn augmentations(
        &self,
        r#_type: AugmentationType,
    ) -> Result<Vec<Augmentation>, UnimplementedError> {
        Err(UnimplementedError("Cyborg::augmentations".into()))
    }

    pub async fn augmentations_by_type(
        &self,
        r#_type: AugmentationType,
    ) -> Result<Vec<Augmentation>, UnimplementedError> {
        Err(UnimplementedError("Cyborg::augmentations_by_type".into()))
    }
}
