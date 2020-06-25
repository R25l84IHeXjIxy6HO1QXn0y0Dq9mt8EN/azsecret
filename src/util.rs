use std::io::{Error, ErrorKind};

use bytes::buf::BufExt;
use hyper::{Body, Client, Method, Request, Response, Uri};
use hyper::body::aggregate;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{from_reader, from_value, to_string, Value};
// use zeroize::Zeroize;

pub type BoxedResult<T> = Result<T, Box<dyn std::error::Error>>;

pub async fn call(method: Method, uri: &str, payload: Option<&impl Serialize>) -> BoxedResult<Response<Body>> {
    let client = Client::new();

    let req = Request::builder()
        .uri(uri.parse::<Uri>()?)
        .method(method)
        .body(match payload {
            Some(v) => Body::from(to_string(v).unwrap()),
            None => Body::empty()
        })?;

    Ok(client.request(req).await?)
}

#[inline]
pub async fn slurp_json<T: DeserializeOwned>(res: Response<Body>) -> BoxedResult<T> {
    let status = res.status();
    let body = aggregate(res).await?;
    let val: Value = from_reader(body.reader())?;

    if status.is_success() {
        Ok(from_value(val)?)
    }
    else {
        Err(Box::new(Error::new(ErrorKind::Other, to_string(&val)?)))
    }
}
