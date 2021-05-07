use warp::{Filter, Rejection, Reply};
use warp::http::StatusCode;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String)
        .and_then(|_name| async move { 
            if false {
                Ok(warp::reply())
            } else {
                Err(warp::reject::custom(AppError::AuthFail))
            }
        })
        .recover(handle_rejection);

    warp::serve(hello)
        .run(([0, 0, 0, 0], 3030))
        .await;
    Ok(())
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
