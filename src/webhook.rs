use std::fmt::{Display, Formatter};
use std::sync::Mutex;

use actix_web::{HttpResponse, post, web, web::Data};
use log::info;
use opentelemetry::propagation::TextMapPropagator;
use opentelemetry::trace::Tracer;
use serde_derive::{Deserialize, Serialize};
use tracing_attributes::instrument;

use crate::bot::LineBot;
use crate::events::{Events, EventType};
use crate::events::messages::MessageType;
use crate::messages::{SendMessageType, TextMessage};
use crate::support::signature::Signature;

/// Signature validator
/// # Note
/// The signature in the `x-line-signature` request header must be verified to confirm that the request was sent from the LINE Platform. [\[detail\]](https://developers.line.biz/en/reference/messaging-api/#signature-validation)
/// # Example
/// ```
/// if webhook::validate_signature(channel_secret, signature, body) {
///     // OK
/// } else {
///     // NG
/// }
/// ```

/*
fn validate_signature(channel_secret: &str, signature: &str, body: &str) -> bool {
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new(channel_secret.as_bytes());
    mac.input(body.as_bytes());

    encode(&mac.result().code().to_vec()) == signature
}
*/
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatQARequest {
    #[serde(rename = "prompt_message")]
    pub prompt_message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatQAReponse {
    #[serde(rename = "message")]
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionRequest {
    #[serde(rename = "model")]
    pub model: String,
    #[serde(rename = "prompt")]
    pub prompt: String,
    #[serde(rename = "max_tokens")]
    pub max_token: i32,
    #[serde(rename = "temperature")]
    pub temperature: Option<f32>,
}

impl Display for CompletionRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "model = {}, prompt = {} , max_tokens = {} ",
            self.model, self.prompt, self.max_token,
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatGPTChoice {
    #[serde(rename = "text")]
    pub text: String,
}

impl Display for ChatGPTChoice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "text = {}", self.text)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatGPTUsage {
    #[serde(rename = "prompt_tokens")]
    pub prompt_tokens: i32,
    #[serde(rename = "completion_tokens")]
    pub completion_token: i32,
    #[serde(rename = "total_tokens")]
    pub total_tokens: i32,
}

impl Display for ChatGPTUsage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "prompt_tokens = {} , completion_token = {} , total_tokens = {}",
            self.prompt_tokens, self.completion_token, self.total_tokens,
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionResponse {
    #[serde(rename = "choices")]
    pub choices: Vec<ChatGPTChoice>,
    #[serde(rename = "usage")]
    pub usage: ChatGPTUsage,
}

impl Display for CompletionResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut msg = "".to_string();
        for (_p, text) in self.choices.iter().enumerate() {
            msg.push_str(text.text.as_str());
        }
        msg = format!("text response = {}\nusage = [\n{:#}\n]", msg, self.usage);
        write!(f, "{}", msg)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineKeys {
    pub channel_secret: String,
    pub access_token: String,
    pub chat_gpt_api_key: String,
    pub chat_gpt_max_tokens: Option<i32>,
    pub chat_gpt_temperature: Option<f32>,
    pub line_chat_prompt: String,
}

#[instrument(skip(config))]
#[post("/v1/line/webhook")]
pub async fn callback(
    _signature: Signature,
    data: web::Json<Events>,
    config: Data<Mutex<LineKeys>>,
    bytes: web::Bytes,
) -> HttpResponse {
    let config = config.lock().unwrap();

    // LineBot
    let bot = LineBot::new(config.channel_secret.as_str(), config.access_token.as_str());

    let body: &str = &String::from_utf8(bytes.to_vec()).unwrap();
    //validate_signature(&bot.channel_secret, &signature.key, &body);

    for event in &data.events {
        // MessageEvent only
        if let EventType::MessageEvent(message_event) = &event.r#type {
            // TextMessageEvent only
            if let MessageType::TextMessage(text_message) = &message_event.message.r#type {
                // Create TextMessage
                info!("message : {}", text_message.text);
                // Reply message with reply_token
                if text_message
                    .text
                    .contains(/*"Nick:>"*/ config.line_chat_prompt.as_str())
                {
                    let message = text_message.text.clone();
                    let message = message.replace("Nick:>", "");

                    let req_completion = CompletionRequest {
                        model: "text-davinci-003".to_string(),
                        prompt: message.trim().to_string(),
                        max_token: 4000,
                        temperature: Some(0.5),
                    };

                    let authorization_api_key =
                        format!("Bearer {}", config.chat_gpt_api_key.as_str());
                    let client = reqwest::Client::new();
                    let res = client
                        .post("https://api.openai.com/v1/completions")
                        .header("Authorization", authorization_api_key)
                        .json(&req_completion)
                        .send()

                        .await;
                    /////
                    match res {
                        Ok(r) => {
                            let msg_resp = r.json::<CompletionResponse>().await;
                            match msg_resp {
                                Ok(msg) => {
                                    let mut message_out = String::new();
                                    for (_pos, choice) in msg.choices.iter().enumerate() {
                                        message_out.push_str(choice.text.as_str());
                                    }
                                    let message = SendMessageType::TextMessage(TextMessage {
                                        text: message_out.trim().to_string(),
                                        emojis: None,
                                    });

                                    let res = bot
                                        .reply_message(
                                            &message_event.reply_token,
                                            vec![message],
                                        )
                                        .await;

                                    match res {
                                        Ok(_) => {}
                                        Err(e) => {
                                            return HttpResponse::InternalServerError().finish();
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Chat Error {}", e);
                                    return HttpResponse::InternalServerError().finish();
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Request Chat Error {}", e);
                            return HttpResponse::InternalServerError().finish();
                        }
                    }
                }
            }
        }
    }
    HttpResponse::Ok().finish()
}
