use rocket::{Request, Route};
use rocket::data::ByteUnit;
use rocket::handler::HandlerFuture;
use rocket::http::{MediaType, Method, Status};
use rocket::outcome::Outcome;
use rocket::request::Outcome as ReqOutcome;
use serde::Deserialize;
use rocket::request::FromRequest;
use crate::db::{add_short_link, Model};
use anyhow::Error;

pub async fn listen() -> anyhow::Result<()> {
    let mut add_route = Route::new(Method::Post, "/", add);
    add_route.format = Some(MediaType::JSON);

    rocket::ignite()
        .mount("/", vec![add_route])
        .launch()
        .await?;

    Ok(())
}

struct ApiKey(String);

impl ApiKey {
    // 验证apikey,目前暂支持环境变量配置
    fn valid(key: &str) -> bool {
        match std::env::var("API_KEY") {
            Ok(s) => s == key,
            _ => false
        }
    }
}

#[derive(Deserialize)]
struct Data {
    link: String,
    custom: Option<String>,
}

fn add<'r>(req: &'r Request, data: rocket::data::Data) -> HandlerFuture<'r> {
    Box::pin(async move {
        let json_limit = req.limits().get("json").unwrap_or(ByteUnit::max_value());

        let stream = data.open(json_limit);

        let data: Data = match serde_json::from_str(
            &match stream.stream_to_string().await {
                Ok(s) => s,
                Err(e) => {
                    error!("Couldn't read body stream: {:?}", e);
                    return Outcome::failure(Status::BadRequest)
                }
            }
        ) {
            Ok(d) => d,
            Err(e) => {
                error!("Couldn't parse JSON body: {:?}", e);
                return if e.is_data()
                { Outcome::failure(Status::UnprocessableEntity) }
                else
                { Outcome::failure(Status::BadRequest) };
            }
        };

        let custom = if let Some(custom) = data.custom {
            // 如果要自定义标识符,则验证apikey
            let keys: Vec<_> = req.headers().get("x-api-key").collect();
            match keys.len() {
                1 if ApiKey::valid(keys[0]) => {},
                _ => return Outcome::failure(Status::BadRequest),
            };
            Some(custom)
        } else {
            None
        };

        match add_short_link(data.link.clone(), custom).await {
            Ok(_) => {}
            Err(e) => {
                error!("`add_short_link` error: {:?}", e);
                return Outcome::failure(Status::BadRequest);
            }
        }

        Outcome::from(req, "")
    })
}
