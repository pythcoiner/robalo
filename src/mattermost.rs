use std::collections::HashMap;

use serde_json::{Map, Value};

#[derive(Debug)]
pub enum Error {
    MinReq(minreq::Error),
    PostFail(i32, Option<String>),
}

impl From<minreq::Error> for Error {
    fn from(value: minreq::Error) -> Self {
        Self::MinReq(value)
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Response {
    pub status: i32,
    pub request: String,
    pub headers: HashMap<String, String>,
    pub body: Value,
}

impl Response {
    pub fn from_minreq<S: Into<String>>(
        request: S,
        response: minreq::Response,
    ) -> Result<Self, Error> {
        Ok(Self {
            status: response.status_code,
            request: request.into(),
            headers: response.headers.clone(),
            body: response.json()?,
        })
    }
}

#[derive(Debug)]
pub struct Client {
    base_url: String,
    token: String,
}

impl Client {
    pub fn new<U, T>(base_url: U, token: T) -> Self
    where
        U: Into<String>,
        T: Into<String>,
    {
        Self {
            base_url: base_url.into(),
            token: token.into(),
        }
    }

    pub fn url(&self, endpoint: &str) -> String {
        format!("{}{}", self.base_url, endpoint)
    }

    pub fn get(&self, endpoint: &str) -> Result<Response, Error> {
        let response = minreq::get(self.url(endpoint))
            .with_header("Authorization", &format!("Bearer {}", self.token))
            .send()?;
        Response::from_minreq(format!("GET {}", endpoint), response)
    }

    pub fn post(&self, endpoint: &str, payload: &Value) -> Result<Response, Error> {
        let response = minreq::post(self.url(endpoint))
            .with_header("Authorization", &format!("Bearer {}", self.token))
            .with_json(payload)?
            .send()?;
        Response::from_minreq(format!("POST {}", endpoint), response)
    }

    pub fn create_post<C, M>(&self, channel_id: C, msg: M) -> Result<(), Error>
    where
        C: Into<String>,
        M: Into<String>,
    {
        let mut payload = Map::new();
        payload.insert("channel_id".into(), Value::String(channel_id.into()));
        payload.insert("message".into(), Value::String(msg.into()));
        let response = self.post("/api/v4/posts", &Value::Object(payload))?;
        match response.status {
            201 => Ok(()),
            status => {
                let msg = response.body.get("message").map(|m| m.to_string());
                Err(Error::PostFail(status, msg))
            }
        }
    }
}
