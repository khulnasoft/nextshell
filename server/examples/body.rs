#![deny(warnings)]

use serde_derive::{Deserialize, Serialize};

use nextshell::Filter;

#[derive(Deserialize, Serialize)]
struct Employee {
    name: String,
    rate: u32,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // POST /employees/:rate  {"name":"Sean","rate":2}
    let promote = nextshell::post()
        .and(nextshell::path("employees"))
        .and(nextshell::path::param::<u32>())
        // Only accept bodies smaller than 16kb...
        .and(nextshell::body::content_length_limit(1024 * 16))
        .and(nextshell::body::json())
        .map(|rate, mut employee: Employee| {
            employee.rate = rate;
            nextshell::reply::json(&employee)
        });

    nextshell::serve(promote).run(([127, 0, 0, 1], 3030)).await
}
