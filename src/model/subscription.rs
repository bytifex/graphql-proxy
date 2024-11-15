use async_graphql::Subscription;
use futures_util::Stream;
use tokio::sync::broadcast;

use crate::admin_state::AdminState;

use super::{inputs::message_filter::MessageFilter, types::message::Message};

pub struct Subscription {
    pub admin_state: AdminState,
}

#[Subscription]
impl Subscription {
    pub async fn messages(
        &self,
        #[graphql(default)] message_filters: Vec<MessageFilter>,
    ) -> impl Stream<Item = Result<Message, broadcast::error::RecvError>> {
        let mut receiver = self.admin_state.message_receiver();

        async_stream::stream! {
            loop {
                match receiver.recv().await {
                    Ok(message) => {
                        let mut is_message_allowed = true;
                        for filter in message_filters.iter() {
                            if let Some(allowed) = filter.is_message_allowed(&message) {
                                is_message_allowed = allowed;
                            }
                        }

                        if is_message_allowed {
                            yield Ok(message);
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(e @ broadcast::error::RecvError::Lagged(_skipped)) => {
                        yield Err(e)
                    }
                }
            }
        }
    }
}
