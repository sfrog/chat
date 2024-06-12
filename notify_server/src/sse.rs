use std::{convert::Infallible, time::Duration};

use axum::response::sse::{Event, Sse};
use axum_extra::{headers, TypedHeader};
use futures::{stream, Stream};
use tokio_stream::StreamExt;

pub(crate) async fn sse_handler(
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("User-Agent: {}", user_agent);

    let stream = stream::repeat_with(|| Event::default().data("hello"))
        .map(Ok)
        .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("ping"),
    )
}
