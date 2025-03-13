#![deny(warnings)]
use nextshell::Filter;

#[tokio::test]
async fn method() {
    let _ = pretty_env_logger::try_init();
    let get = nextshell::get().map(nextshell::reply);

    let req = nextshell::test::request();
    assert!(req.matches(&get).await);

    let req = nextshell::test::request().method("POST");
    assert!(!req.matches(&get).await);

    let req = nextshell::test::request().method("POST");
    let resp = req.reply(&get).await;
    assert_eq!(resp.status(), 405);
}

#[tokio::test]
async fn method_not_allowed_trumps_not_found() {
    let _ = pretty_env_logger::try_init();
    let get = nextshell::get().and(nextshell::path("hello").map(nextshell::reply));
    let post = nextshell::post().and(nextshell::path("bye").map(nextshell::reply));

    let routes = get.or(post);

    let req = nextshell::test::request().method("GET").path("/bye");

    let resp = req.reply(&routes).await;
    // GET was allowed, but only for /hello, so POST returning 405 is fine.
    assert_eq!(resp.status(), 405);
}

#[tokio::test]
async fn bad_request_trumps_method_not_allowed() {
    let _ = pretty_env_logger::try_init();
    let get = nextshell::get()
        .and(nextshell::path("hello"))
        .and(nextshell::header::exact("foo", "bar"))
        .map(nextshell::reply);
    let post = nextshell::post()
        .and(nextshell::path("bye"))
        .map(nextshell::reply);

    let routes = get.or(post);

    let req = nextshell::test::request().method("GET").path("/hello");

    let resp = req.reply(&routes).await;
    // GET was allowed, but header rejects with 400, should not
    // assume POST was the appropriate method.
    assert_eq!(resp.status(), 400);
}
