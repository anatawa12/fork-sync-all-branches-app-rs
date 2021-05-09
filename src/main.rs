use warp::{Filter, Rejection, Reply};
use warp::http::StatusCode;
use warp::hyper::body::Bytes;
use warp::filters::BoxedFilter;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let ctx = Context {
        webhook_secret: std::env::var("GITHUB_WEBHOOK_SECRET").unwrap().into(),
    };

    let hello = warp::path!("event_handler")
        .and(ctx.verify_webhook_signature())
        .and(warp::header::<String>("X-Github-Event"))
        .and_then(|a, b| ctx.process_event(a, b))
        .untuple_one()
        .map(warp::reply::reply)
        .recover(handle_rejection);

    warp::serve(hello)
        .run(([0, 0, 0, 0], 3030))
        .await;
    Ok(())
}

#[derive(Clone)]
struct Context {
    webhook_secret: Bytes,
}

impl Context {
    fn verify_webhook_signature(self) -> impl Filter<Extract=(Bytes, ), Error=Rejection> + Clone {
        use hmac::{Hmac, Mac, NewMac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;
        warp::body::bytes().and(warp::header::header::<String>("X-Hub-Signature-256"))
            .and_then(move |body: Bytes, header: String| async move {
                // TODO: read from env
                let webhook_secret = &self.webhook_secret;
                let mut mac = HmacSha256::new_from_slice(webhook_secret)
                    .expect("HMAC can take key of any size");
                mac.update(&body);
                let hmac_hash = &*mac.finalize().into_bytes();
                let header_hash = header.strip_prefix("sha256=")
                    .and_then(|x| hex::decode(x).ok())
                    .ok_or(warp::reject::custom(AppError::AuthFail))?;
                if &*header_hash != hmac_hash {
                    return Err(warp::reject::custom(AppError::AuthFail));
                }
                Ok(body)
            })
    }

    async fn process_event(&self, body: Bytes, event: String) -> Result<(), Rejection> {
        log::info!("event: {}", event);
        log::info!("body : {}", std::str::from_utf8(&body).unwrap_or("unknown"));
        Ok(())
    }
}

#[derive(Debug)]
enum AppError {
    AuthFail
}

impl warp::reject::Reject for AppError {
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(err) = err.find::<AppError>() {
        return match err {
            AppError::AuthFail =>
                Ok(warp::reply::with_status(warp::reply(), StatusCode::UNAUTHORIZED))
        }
    }
    return Err(err)
}
