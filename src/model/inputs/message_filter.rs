use async_graphql::InputObject;
use clap::{builder::TypedValueParser, error::ErrorKind, ValueEnum};

use crate::model::{
    enums::{
        connection_type::ConnectionType, filter_type::FilterType,
        message_direction::MessageDirection, payload_type::PayloadType,
    },
    types::message::Message,
};

#[derive(Debug, Clone, InputObject)]
pub struct MessageFilter {
    pub filter_type: FilterType,
    pub connection_type: Option<ConnectionType>,
    pub message_direction: Option<MessageDirection>,
    pub payload_type: Option<PayloadType>,
}

impl MessageFilter {
    pub fn is_message_allowed(&self, message: &Message) -> Option<bool> {
        let is_matching = self.is_message_matching(message);

        if is_matching {
            match self.filter_type {
                FilterType::Allow => Some(is_matching),
                FilterType::Prohibit => Some(!is_matching),
            }
        } else {
            None
        }
    }

    pub fn is_message_matching(&self, message: &Message) -> bool {
        if let Some(connection_type) = self.connection_type {
            if message.connection_type != connection_type {
                return false;
            }
        }

        if let Some(message_direction) = self.message_direction {
            if message.message_direction != message_direction {
                return false;
            }
        }

        let payload = match message.connection_type {
            ConnectionType::Http => &message.message,
            ConnectionType::Ws => {
                if let serde_json::Value::Object(map) = &*message.message {
                    &map["payload"]
                } else {
                    &message.message
                }
            }
        };

        if let Some(payload_type) = self.payload_type {
            if PayloadType::from_json(payload) != payload_type {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Clone)]
pub struct MessageFilterCliParser;

impl MessageFilterCliParser {
    fn create_filter_type_variants_message(&self) -> String {
        let mut message = String::new();

        message += "<filter-type> variants: [";
        let mut first = true;
        FilterType::value_variants().iter().for_each(|variant| {
            if let Some(variant) = variant.to_possible_value() {
                if first {
                    first = false;
                } else {
                    message += ", ";
                }
                message += variant.get_name();
            }
        });
        message += "]";

        message
    }

    fn create_connection_type_variants_message(&self) -> String {
        let mut message = String::new();

        message += "<connection-type> variants: [";
        let mut first = true;
        ConnectionType::value_variants().iter().for_each(|variant| {
            if let Some(variant) = variant.to_possible_value() {
                if first {
                    first = false;
                } else {
                    message += ", ";
                }
                message += variant.get_name();
            }
        });
        message += "]";

        message
    }

    fn create_message_direction_variants_message(&self) -> String {
        let mut message = String::new();

        message += "<message-direction> variants: [";
        let mut first = true;
        MessageDirection::value_variants()
            .iter()
            .for_each(|variant| {
                if let Some(variant) = variant.to_possible_value() {
                    if first {
                        first = false;
                    } else {
                        message += ", ";
                    }
                    message += variant.get_name();
                }
            });
        message += "]";

        message
    }

    fn create_payload_type_variants_message(&self) -> String {
        let mut message = String::new();

        message += "<payload-type> variants: [";
        let mut first = true;
        PayloadType::value_variants().iter().for_each(|variant| {
            if let Some(variant) = variant.to_possible_value() {
                if first {
                    first = false;
                } else {
                    message += ", ";
                }
                message += variant.get_name();
            }
        });
        message += "]";

        message
    }

    fn create_error_message(&self) -> String {
        let mut message = "Invalid message filter format. Expected <filter_type>:<connection-type>,<message-direction>,<payload-type>".to_string();

        message += "; ";
        message += &self.create_filter_type_variants_message();

        message += "; ";
        message += &self.create_connection_type_variants_message();

        message += "; ";
        message += &self.create_message_direction_variants_message();

        message += "; ";
        message += &self.create_payload_type_variants_message();

        message
    }

    fn try_parse(&self, value: &str) -> Result<MessageFilter, String> {
        let (filter_type_str, remaining) = value
            .split_once(":")
            .ok_or_else(|| self.create_error_message())?;

        let filter_type = FilterType::from_str(filter_type_str, true)
            .map_err(|e| format!("{e}, {}", self.create_filter_type_variants_message()))?;

        let mut collection = remaining.split(",");

        let connection_type = {
            let connection_type_str = collection
                .next()
                .ok_or_else(|| self.create_error_message())?;

            if connection_type_str.to_ascii_lowercase() == "any" {
                None
            } else {
                Some(
                    ConnectionType::from_str(connection_type_str, true).map_err(|e| {
                        format!("{e}, {}", self.create_connection_type_variants_message())
                    })?,
                )
            }
        };

        let message_direction = {
            let message_direction_str = collection
                .next()
                .ok_or_else(|| self.create_error_message())?;

            if message_direction_str.to_ascii_lowercase() == "any" {
                None
            } else {
                Some(
                    MessageDirection::from_str(message_direction_str, true).map_err(|e| {
                        format!("{e}, {}", self.create_message_direction_variants_message())
                    })?,
                )
            }
        };

        let payload_type =
            {
                let payload_type_str = collection
                    .next()
                    .ok_or_else(|| self.create_error_message())?;

                if payload_type_str.to_ascii_lowercase() == "any" {
                    None
                } else {
                    Some(PayloadType::from_str(payload_type_str, true).map_err(|e| {
                        format!("{e}, {}", self.create_payload_type_variants_message())
                    })?)
                }
            };

        if collection.next().is_some() {
            return Err(self.create_error_message());
        }

        Ok(MessageFilter {
            filter_type,
            connection_type,
            message_direction,
            payload_type,
        })
    }
}

impl TypedValueParser for MessageFilterCliParser {
    type Value = MessageFilter;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let value = value.to_string_lossy();
        self.try_parse(&value)
            .map_err(|e| cmd.clone().error(ErrorKind::ValueValidation, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::enums::connection_type::ConnectionType;
    use crate::model::enums::filter_type::FilterType;
    use crate::model::enums::message_direction::MessageDirection;
    use crate::model::enums::payload_type::PayloadType;

    #[test]
    fn test_message_filter_cli_parser() {
        let parser = MessageFilterCliParser;

        assert!(matches!(parser.try_parse("allow:any,any,any,any"), Err(_)));

        // FilterType
        {
            assert!(matches!(
                parser.try_parse("allow:any,any,any"),
                Ok(MessageFilter {
                    filter_type: FilterType::Allow,
                    connection_type: None,
                    message_direction: None,
                    payload_type: None,
                })
            ));

            assert!(matches!(
                parser.try_parse("prohibit:any,any,any"),
                Ok(MessageFilter {
                    filter_type: FilterType::Prohibit,
                    connection_type: None,
                    message_direction: None,
                    payload_type: None,
                })
            ));
        }

        // ConnectionType
        {
            assert!(matches!(
                parser.try_parse("allow:http,any,any"),
                Ok(MessageFilter {
                    filter_type: FilterType::Allow,
                    connection_type: Some(ConnectionType::Http),
                    message_direction: None,
                    payload_type: None,
                })
            ));

            assert!(matches!(
                parser.try_parse("allow:ws,any,any"),
                Ok(MessageFilter {
                    filter_type: FilterType::Allow,
                    connection_type: Some(ConnectionType::Ws),
                    message_direction: None,
                    payload_type: None,
                })
            ));
        }

        // MessageDirection
        {
            assert!(matches!(
                parser.try_parse("allow:any,request,any"),
                Ok(MessageFilter {
                    filter_type: FilterType::Allow,
                    connection_type: None,
                    message_direction: Some(MessageDirection::Request),
                    payload_type: None,
                })
            ));

            assert!(matches!(
                parser.try_parse("allow:any,response,any"),
                Ok(MessageFilter {
                    filter_type: FilterType::Allow,
                    connection_type: None,
                    message_direction: Some(MessageDirection::Response),
                    payload_type: None,
                })
            ));
        }

        // PayloadType
        {
            assert!(matches!(
                parser.try_parse("allow:any,any,request"),
                Ok(MessageFilter {
                    filter_type: FilterType::Allow,
                    connection_type: None,
                    message_direction: None,
                    payload_type: Some(PayloadType::Request),
                })
            ));
            assert!(matches!(
                parser.try_parse("allow:any,any,only-data"),
                Ok(MessageFilter {
                    filter_type: FilterType::Allow,
                    connection_type: None,
                    message_direction: None,
                    payload_type: Some(PayloadType::OnlyData),
                })
            ));
            assert!(matches!(
                parser.try_parse("allow:any,any,only-error"),
                Ok(MessageFilter {
                    filter_type: FilterType::Allow,
                    connection_type: None,
                    message_direction: None,
                    payload_type: Some(PayloadType::OnlyError),
                })
            ));
            assert!(matches!(
                parser.try_parse("allow:any,any,partial-data-and-error"),
                Ok(MessageFilter {
                    filter_type: FilterType::Allow,
                    connection_type: None,
                    message_direction: None,
                    payload_type: Some(PayloadType::PartialDataAndError),
                })
            ));
            assert!(matches!(
                parser.try_parse("allow:any,any,non-graph-ql"),
                Ok(MessageFilter {
                    filter_type: FilterType::Allow,
                    connection_type: None,
                    message_direction: None,
                    payload_type: Some(PayloadType::NonGraphQl),
                })
            ));
        }
    }
}
