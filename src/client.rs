use opentelemetry::trace::FutureExt;
use opentelemetry::Context;
use reqwest::{Client, Error, Response};
//use reqwest::blocking::{Client, Response};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Url;
use serde_json::Value;

static BASE_URL: &str = "https://api.line.me/v2/bot";
static BASEDATA_URL: &str = "https://api-data.line.me/v2/bot";

#[derive(Debug)]
pub struct HttpClient {
    client: Client,
    headers: HeaderMap,
    endpoint_base: String,
    endpoint_base_data: String,
}

impl HttpClient {
    /// # Note
    /// Instantiate a HttpClient.
    /// ```
    /// let http_client = HttpClient::new("<channel secret>");
    /// ```
    pub fn new(channel_token: &str) -> HttpClient {
        let mut headers = HeaderMap::new();
        if let Ok(v) = format!("Bearer {}", channel_token).parse::<String>() {
            if let Ok(header_value) = HeaderValue::from_str(&v) {
                headers.insert(AUTHORIZATION, header_value);
            }
        }
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        HttpClient {
            client: Client::new(),
            headers,
            endpoint_base: String::from(BASE_URL),
            endpoint_base_data: String::from(BASEDATA_URL),
        }
    }

    /// # Note
    /// `GET` request
    /// ```
    /// let res: Result<Response, Error> = http_client.get("https://example.com");
    /// ```
    pub async fn get(
        &self,
        endpoint: &str,
        query: Vec<(&str, &str)>,
        data: Value,
    ) -> Result<Response, Error> {
        let uri = Url::parse(&format!("{}{}", self.endpoint_base, endpoint)).unwrap();
        self.client
            .get(uri)
            .query(&query)
            .headers(self.headers.clone())
            .json(&data)
            .send()
            .await
    }

    /// # Note
    /// `GET` request
    /// ```
    /// let res: Result<Response, Error> = http_client.get_data("https://example.com");
    /// ```
    pub async fn get_data(
        &self,
        endpoint: &str,
        query: Vec<(&str, &str)>,
        data: Value,
    ) -> Result<Response, Error> {
        let uri = Url::parse(&format!("{}{}", self.endpoint_base_data, endpoint)).unwrap();
        self.client
            .get(uri)
            .query(&query)
            .headers(self.headers.clone())
            .json(&data)
            .send()
            .await
    }

    /// # Note
    /// `POST` request
    /// ```
    /// let res: Result<Response, Error> = http_client.post("https://example.com");
    /// ```
    pub async fn post(&self, endpoint: &str, data: Value) -> Result<Response, Error> {
        let uri = Url::parse(&format!("{}{}", self.endpoint_base, endpoint)).unwrap();
        self.client
            .post(uri)
            .headers(self.headers.clone())
            .json(&data)
            .send()
            .await
    }
    /// # Note
    /// `POST` request
    /// ```
    /// let res: Result<Response, Error> = http_client.post("https://example.com");
    /// ```
    pub async fn post_with_context(
        &self,
        endpoint: &str,
        data: Value,
        context: Context,
    ) -> Result<Response, Error> {
        let uri = Url::parse(&format!("{}{}", self.endpoint_base, endpoint)).unwrap();
        self.client
            .post(uri)
            .headers(self.headers.clone())
            .json(&data)
            .send()
            .with_context(context.to_owned())
            .await
    }

    /// # Note
    /// `PUT` request
    /// ```
    /// let res: Result<Response, Error> = http_client.put("https://example.com");
    /// ```
    pub async fn put(&self, endpoint: &str, data: Value) -> Result<Response, Error> {
        let uri = Url::parse(&format!("{}{}", self.endpoint_base, endpoint)).unwrap();
        self.client
            .put(uri)
            .headers(self.headers.clone())
            .json(&data)
            .send()
            .await
    }

    /// # Note
    /// `DELETE` request
    /// ```
    /// let res: Result<Response, Error> = http_client.delete("https://example.com");
    /// ```
    pub async fn delete(&self, endpoint: &str, data: Value) -> Result<Response, Error> {
        let uri = Url::parse(&format!("{}{}", self.endpoint_base, endpoint)).unwrap();
        self.client
            .delete(uri)
            .headers(self.headers.clone())
            .json(&data)
            .send()
            .await
    }
}
