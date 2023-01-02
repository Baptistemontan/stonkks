use serde::Serialize;
use std::fmt::Debug;

pub enum ContentType {
    Text,
    Json,
}

pub struct Response {
    pub content_type: ContentType,
    pub content: Vec<u8>,
}

impl Response {
    fn new<C: Into<Vec<u8>>>(content_type: ContentType, content: C) -> Self {
        let content = content.into();
        Response {
            content_type,
            content,
        }
    }
}

pub trait IntoResponse {
    type Err: Debug;
    fn into_response(self) -> Result<Response, Self::Err>;
}

pub struct Json<T: Serialize>(pub T);

impl<T: Serialize> IntoResponse for Json<T> {
    type Err = serde_json::Error;
    fn into_response(self) -> Result<Response, Self::Err> {
        let serialized_value = serde_json::to_string(&self.0)?;
        Ok(Response::new(ContentType::Json, serialized_value))
    }
}
