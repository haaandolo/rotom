use std::fmt::Debug;

use bytes::Bytes;
use chrono::Utc;
use serde::Serialize;

use crate::{
    error::SocketError,
    metric::{Field, Metric, Tag},
};

use super::{auth::Authenticator, http_parser::HttpParser, rest_request::RestRequest};

#[derive(Debug)]
pub struct RestClient<Parser, Auth> {
    pub http_client: reqwest::Client,
    pub base_url: &'static str,
    pub parser: Parser,
    pub auth: Auth,
}

impl<Parser, Auth> RestClient<Parser, Auth>
where
    Auth: Authenticator,
{
    pub fn new(base_url: &'static str, parser: Parser, auth: Auth) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            base_url,
            parser,
            auth,
        }
    }

    pub async fn execute<Request>(
        &self,
        request: Request,
    ) -> Result<(Request::Response, Metric), Parser::OutputError>
    where
        Request: RestRequest,
        Parser: HttpParser,
    {
        let request = self.build(request)?;
        let (status, payload, latency) = self.measured_execution::<Request>(request).await?;
        self.parser
            .parse::<Request::Response>(status, &payload)
            .map(|response| (response, latency))
    }

    pub fn build<Request>(&self, request: Request) -> Result<reqwest::Request, SocketError>
    where
        Request: RestRequest,
    {
        let url = format!("{}{}", self.base_url, request.path());
        let mut builder = self
            .http_client
            .request(Request::method(), url)
            .timeout(Request::timeout());

        // Add optional query params
        if let Some(query_params) = request.query_params() {
            builder = builder.query(query_params);
        }

        // Add optional Body
//        if let Some(body) = request.body() {
//            builder = builder.body(body);
//            // builder = builder.json(body);
//        }

        Auth::build_signed_request(builder, request)
    }

    pub async fn measured_execution<Request>(
        &self,
        request: reqwest::Request,
    ) -> Result<(reqwest::StatusCode, Bytes, Metric), SocketError>
    where
        Request: RestRequest,
    {
        let mut latency = Metric {
            name: "http_request_duration",
            time: Utc::now().timestamp_millis() as u64,
            tags: vec![
                Tag::new("http_method", Request::method().as_str()),
                Tag::new::<&str>("base_url", self.base_url),
                Tag::new("path", request.url().path()),
            ],
            fields: Vec::with_capacity(1),
        };

        let start = std::time::Instant::now();
        let response = self.http_client.execute(request).await?;
        let duration = start.elapsed().as_millis() as u64;

        latency
            .tags
            .push(Tag::new("status_code", response.status().as_str()));
        latency.fields.push(Field::new("duration", duration));

        let status_code = response.status();
        let payload = response.bytes().await?;

        Ok((status_code, payload, latency))
    }
}
