use std::sync::Arc;

use async_graphql::Object;
use http::HeaderMap;
use parking_lot::RwLock;

use crate::model::scalars::{
    header_name_scalar::HeaderNameScalar, header_value_scalar::HeaderValueScalar,
};

use super::header::Header;

#[derive(Debug)]
pub enum Headers {
    RwLockHeaderMap {
        header_map: Arc<RwLock<HeaderMap>>,
        header_vec: Vec<Header>,
    },
    HeaderMap {
        header_map: HeaderMap,
        header_vec: Vec<Header>,
    },
}

impl Headers {
    pub fn from_header_map(header_map: HeaderMap) -> Self {
        let header_vec = Self::collect_headers_to_vec(&header_map);

        Self::HeaderMap {
            header_map,
            header_vec,
        }
    }

    pub fn from_rw_lock_header_map(header_map: Arc<RwLock<HeaderMap>>) -> Self {
        let header_vec = Self::collect_headers_to_vec(&header_map.read());

        Self::RwLockHeaderMap {
            header_map,
            header_vec,
        }
    }

    fn collect_headers_to_vec(header_map: &HeaderMap) -> Vec<Header> {
        header_map
            .iter()
            .map(|(name, value)| Header {
                name: name.clone().into(),
                value: value.clone().into(),
            })
            .collect()
    }
}

#[Object]
impl Headers {
    pub async fn by_name(&self, name: HeaderNameScalar) -> Option<HeaderValueScalar> {
        let header_map = match self {
            Headers::RwLockHeaderMap {
                header_map,
                header_vec: _header_vec,
            } => &*header_map.read(),
            Headers::HeaderMap {
                header_map,
                header_vec: _header_vec,
            } => header_map,
        };

        header_map
            .get(name.as_header_name())
            .cloned()
            .map(|item| item.into())
    }

    pub async fn all(&self) -> &Vec<Header> {
        match self {
            Headers::RwLockHeaderMap {
                header_map: _header_map,
                header_vec,
            } => &header_vec,
            Headers::HeaderMap {
                header_map: _header_map,
                header_vec,
            } => &header_vec,
        }
    }
}
