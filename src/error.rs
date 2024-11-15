#[derive(Clone, Debug, thiserror::Error)]
#[error("UnimplementedError: '{0}'")]
pub struct UnimplementedError(pub String);

#[derive(Debug, thiserror::Error)]
#[error("UnspecifiedGraphQLEndpointError")]
pub struct UnspecifiedGraphQLEndpointError;

#[derive(Debug, thiserror::Error)]
#[error("UnspecifiedGraphQLWsEndpointError")]
pub struct UnspecifiedGraphQLWsEndpointError;
