use std::env;
use std::sync::Mutex;

use actix_web::{App, HttpResponse, HttpServer, web, web::Data};
use actix_web::middleware::Logger;
use actix_web_opentelemetry::RequestTracing;
use opentelemetry::global::shutdown_tracer_provider;
use rand::Rng;
use tracing::debug;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

use crate::webhook::LineKeys;

mod bot;
mod client;
mod events;
mod messages;
mod objects;
mod support;
mod webhook;

//use chatgpt::prelude::*;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_insights_connection_str = env::var("APPLICATIONINSIGHTS_CON_STRING");

    if let Ok(app_insights_connection) = app_insights_connection_str {
        debug!(
                "APPLICATIONINSIGHTS_CON_STRING = {}",
                app_insights_connection
            );
        let exporter = opentelemetry_application_insights::new_pipeline_from_connection_string(
            app_insights_connection,
        )
            .unwrap()
            .with_client(reqwest::Client::new())
            .with_service_name("LineChatBot")
            .install_simple();

        let telemetry = tracing_opentelemetry::layer().with_tracer(exporter);
        let subscriber = Registry::default().with(telemetry);
        tracing::subscriber::set_global_default(subscriber)
            .expect("setting global default failed");
    }
    ////////

    let channel_secret: &str =
        &env::var("LINE_CHANNEL_SECRET").expect("Failed getting LINE_CHANNEL_SECRET");

    let access_token: &str =
        &env::var("LINE_CHANNEL_ACCESS_TOKEN").expect("Failed getting LINE_CHANNEL_ACCESS_TOKEN");

    let chat_gpt_api_key: &str =
        &env::var("CHATGPT_API_KEY").expect("Failed getting CHATGPT_API_KEY");

    let line_chat_prompt: &str =
        &env::var("LINE_CHAT_PROMPT").expect("Failed getting LINE_CHAT_PROMPT");

    let data = Data::new(Mutex::new(LineKeys {
        channel_secret: channel_secret.to_string(),
        access_token: access_token.to_string(),
        chat_gpt_api_key: chat_gpt_api_key.to_string(),
        chat_gpt_max_tokens: None,
        chat_gpt_temperature: None,
        line_chat_prompt: line_chat_prompt.to_string(),
    }));

    /////
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(RequestTracing::new())
            .wrap(TracingLogger::default())
            .app_data(Data::clone(&data))
            .service(webhook::callback)
            .service(
                web::resource("/")
                    .route(web::get().to(|| async { HttpResponse::Ok().body("Hello World!") }))
            )
    })
        .workers(20)
        .bind("0.0.0.0:8080")?
        .run()
        .await?;

    // wait until all pending spans get exported.
    shutdown_tracer_provider();

    Ok(())
}
