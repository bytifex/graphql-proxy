use async_graphql::Object;

use crate::model::{enums::character_race::CharacterRace, scalars::id::Id};

pub struct Human {
    pub id: Id,
    pub name: Option<String>,
    pub nickname: String,
    pub race: CharacterRace,
}

#[Object]
impl Human {
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
}
