#![deny(warnings)]

use nextshell::Filter;

#[tokio::main]
async fn main() {
    let file = nextshell::path("todos").and(nextshell::fs::file("./examples/todos.rs"));
    // NOTE: You could double compress something by adding a compression
    // filter here, a la
    // ```
    // let file = nextshell::path("todos")
    //     .and(nextshell::fs::file("./examples/todos.rs"))
    //     .with(nextshell::compression::brotli());
    // ```
    // This would result in a browser error, or downloading a file whose contents
    // are compressed

    let dir = nextshell::path("ws_chat").and(nextshell::fs::file("./examples/websockets_chat.rs"));

    let file_and_dir = nextshell::get()
        .and(file.or(dir))
        .with(nextshell::compression::gzip());

    let examples = nextshell::path("ex")
        .and(nextshell::fs::dir("./examples/"))
        .with(nextshell::compression::deflate());

    // GET /todos => gzip -> toods.rs
    // GET /ws_chat => gzip -> ws_chat.rs
    // GET /ex/... => deflate -> ./examples/...
    let routes = file_and_dir.or(examples);

    nextshell::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
