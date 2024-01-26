use opentelemetry::Context;
use reqwest::{Error, Response};
use serde_json::{json, Value};

use crate::client::HttpClient;
use crate::messages::SendMessageType;

/// LineBot Client
#[derive(Debug)]
pub struct LineBot {
    pub channel_secret: String,
    pub channel_token: String,
    pub http_client: HttpClient,
}

impl LineBot {
    /// # Note
    /// Instantiate a LineBot.
    /// ```
    /// let bot = LineBot::new("<channel secret>", "<channel access token>");
    /// ```
    pub fn new(channel_secret: &str, channel_token: &str) -> LineBot {
        LineBot {
            channel_secret: String::from(channel_secret),
            channel_token: String::from(channel_token),
            http_client: HttpClient::new(channel_token),
        }
    }

    /// # Note
    /// Send reply message. [\[detail\]](https://developers.line.biz/en/reference/messaging-api/#send-reply-message)
    /// ```
    /// let res: Result<Response, Error> = bot.reply_message("xxxxxxxxx", vec![...]);
    /// ```
    pub async fn reply_message(
        &self,
        reply_token: &str,
        msgs: Vec<SendMessageType>,
    ) -> Result<Response, Error> {
        let data: Value = json!(
                {
                "replyToken": reply_token,
                "messages": msgs,
                }
        );
        self.http_client.post("/message/reply", data).await
    }
    /// # Note
    /// Send reply message. [\[detail\]](https://developers.line.biz/en/reference/messaging-api/#send-reply-message)
    /// ```
    /// let res: Result<Response, Error> = bot.reply_message("xxxxxxxxx", vec![...]);
    /// ```
    pub async fn reply_message_with_context(
        &self,
        reply_token: &str,
        msgs: Vec<SendMessageType>,
        context: Context,
    ) -> Result<Response, Error> {
        let data: Value = json!(
                {
                "replyToken": reply_token,
                "messages": msgs,
                }
        );
        self.http_client
            .post_with_context("/message/reply", data, context.to_owned())
            .await
    }
}
