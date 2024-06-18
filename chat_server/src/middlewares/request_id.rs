use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use tracing::warn;

use super::REQUEST_ID_HEADER;

pub async fn set_request_id(mut req: Request, next: Next) -> Response {
    // if x-request-id header is not present, generate a new request id
    let id = match req.headers().get(REQUEST_ID_HEADER) {
        Some(v) => Some(v.clone()),
        None => {
            let request_id = uuid::Uuid::now_v7().to_string();
            match HeaderValue::from_str(&request_id) {
                Ok(v) => {
                    req.headers_mut().insert(REQUEST_ID_HEADER, v.clone());
                    Some(v)
                }
                Err(e) => {
                    warn!("Error parsing request id for request: {}", e);
                    None
                }
            }
        }
    };

    let mut res = next.run(req).await;

    let Some(v) = id else {
        return res;
    };

    res.headers_mut().insert(REQUEST_ID_HEADER, v);
    res
}
