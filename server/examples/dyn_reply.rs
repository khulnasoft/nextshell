#![deny(warnings)]
use nextshell::{http::StatusCode, Filter};

async fn dyn_reply(word: String) -> Result<Box<dyn nextshell::Reply>, nextshell::Rejection> {
    if &word == "hello" {
        Ok(Box::new("world"))
    } else {
        Ok(Box::new(StatusCode::BAD_REQUEST))
    }
}

#[tokio::main]
async fn main() {
    let routes = nextshell::path::param().and_then(dyn_reply);

    nextshell::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
