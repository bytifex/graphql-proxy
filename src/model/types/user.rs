use async_graphql::Object;

use crate::{
    error::UnimplementedError,
    model::{interfaces::character::Character, scalars::id::Id},
};

pub struct User {
    pub id: Id,
    pub display_name: String,
    pub email_address: Option<String>,
}

#[Object]
impl User {
    pub async fn display_name(&self) -> &String {
        &self.display_name
    }

    pub async fn email_address(&self) -> &Option<String> {
        &self.email_address
    }

    pub async fn id(&self) -> &Id {
        &self.id
    }

    pub async fn character_by_id(&self, _id: Id) -> Result<Option<Character>, UnimplementedError> {
        Err(UnimplementedError("User::character_by_id".into()))
    }

    pub async fn characters(&self) -> Result<Vec<Character>, UnimplementedError> {
        Err(UnimplementedError("User::character_by_id".into()))
    }
}
