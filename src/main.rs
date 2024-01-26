use std::env;
use std::sync::Mutex;
use std::time::Duration;

use actix_web::middleware::Logger;
use actix_web::{web::Data, App, HttpServer};
use actix_web_opentelemetry::{RequestMetricsBuilder, RequestTracing};
use opentelemetry::global::shutdown_tracer_provider;
use opentelemetry::sdk::export::metrics::aggregation::stateless_temporality_selector;
use opentelemetry::sdk::metrics::{controllers, processors, selectors};
use opentelemetry::{global, metrics::Unit, Context, KeyValue};
use rand::{thread_rng, Rng};
use tracing_subscriber::{layer::SubscriberExt, Registry};

use crate::webhook::LineKeys;

mod bot;
mod client;
mod events;
mod messages;
mod objects;
mod support;
mod webhook;

//use chatgpt::prelude::*;

//open api key
//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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let instrumentation_key = std::env::var("INSTRUMENTATION_KEY")
        .unwrap_or_else(|_| "10848a4a-6c1b-45bd-9113-152bcfbac1cc".to_string());
    let instrumentation_endpoint = std::env::var("INSTRUMENTATION_ENDPOINT")
        .unwrap_or_else(|_| "https://southeastasia-1.in.applicationinsights.azure.com".to_string());

    let tracer = opentelemetry_application_insights::new_pipeline(instrumentation_key.to_owned())
        .with_client(reqwest::Client::new())
        .with_endpoint(instrumentation_endpoint.to_owned().as_str())
        .unwrap()
        .install_batch(opentelemetry::runtime::Tokio);

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer.to_owned());
    let subscriber = Registry::default().with(telemetry);
    tracing::subscriber::set_global_default(subscriber).expect("setting global default failed");

    //global::set_tracer_provider(tracer.to_owned().provider().unwrap());

    //////
    let temporality_selector = stateless_temporality_selector();
    let exporter =
        opentelemetry_application_insights::Exporter::new(instrumentation_key.to_owned(), ())
            .with_temporality_selector(temporality_selector.clone())
            .with_endpoint(instrumentation_endpoint.to_owned().as_str())
            .expect("Export exporter error");
    let controller = controllers::basic(processors::factory(
        //selectors::simple::inexpensive(),
        selectors::simple::histogram([1.0, 2.0, 5.0, 10.0, 20.0, 50.0]),
        temporality_selector,
    ))
    .with_exporter(exporter)
    .with_collect_period(Duration::from_secs(1))
    .build();

    let meter = global::meter("actix_web");
    let cpu_utilization_gauge = meter
        .to_owned()
        .f64_observable_gauge("system.cpu.utilization")
        .with_unit(Unit::new("1"))
        .init();
    meter
        .register_callback(move |cx| {
            let mut rng = thread_rng();
            cpu_utilization_gauge.observe(
                cx,
                rng.gen_range(0.1..0.2),
                &[KeyValue::new("state", "idle"), KeyValue::new("cpu", 0)],
            )
        })
        .expect("");

    let request_metrics = RequestMetricsBuilder::new().build(meter.to_owned());

    //////////
    let cx = Context::new();
    controller
        .start(&cx, opentelemetry::runtime::Tokio)
        .unwrap();
    global::set_meter_provider(controller.clone());

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
            .wrap(request_metrics.clone())
            .app_data(Data::clone(&data))
            .service(webhook::callback)
    })
    .workers(20)
    .bind("0.0.0.0:8000")?
    .run()
    .await?;

    // wait until all pending spans get exported.
    shutdown_tracer_provider();

    Ok(())
}
