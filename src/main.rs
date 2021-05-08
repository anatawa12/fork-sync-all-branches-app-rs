use warp::{Filter, Rejection, Reply};
use warp::http::StatusCode;
use warp::hyper::body::Bytes;
use warp::filters::BoxedFilter;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let hello = warp::path!("event_handler")
        .and(verify_webhook_signature())
        .map(warp::reply::reply)
        .recover(handle_rejection);

    warp::serve(hello)
        .run(([0, 0, 0, 0], 3030))
        .await;
    Ok(())
}

fn verify_webhook_signature() -> BoxedFilter<()> {
    use hmac::{Hmac, Mac, NewMac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;
    warp::body::bytes().and(warp::header::header::<String>("X-Hub-Signature-256"))
        .and_then(|body: Bytes, header: String| async move {
            // TODO: read from env
            let webhook_secret = b"sercret";
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
            Ok(())
        })
        .untuple_one()
        .boxed()
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
