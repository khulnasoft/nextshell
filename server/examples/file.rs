#![deny(warnings)]

use nextshell::Filter;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let readme = nextshell::get()
        .and(nextshell::path::end())
        .and(nextshell::fs::file("./README.md"));

    // dir already requires GET...
    let examples = nextshell::path("ex").and(nextshell::fs::dir("./examples/"));

    // GET / => README.md
    // GET /ex/... => ./examples/..
    let routes = readme.or(examples);

    nextshell::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
