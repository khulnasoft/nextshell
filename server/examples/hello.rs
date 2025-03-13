#![deny(warnings)]
use nextshell::Filter;

#[tokio::main]
async fn main() {
    // Match any request and return hello world!
    let routes = nextshell::any().map(|| "Hello, World!");

    nextshell::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
