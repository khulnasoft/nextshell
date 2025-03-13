#![deny(warnings)]
use nextshell::http::header::{HeaderMap, HeaderValue};
use nextshell::Filter;

#[tokio::test]
async fn header() {
    let header = nextshell::reply::with::header("foo", "bar");

    let no_header = nextshell::any().map(nextshell::reply).with(&header);

    let req = nextshell::test::request();
    let resp = req.reply(&no_header).await;
    assert_eq!(resp.headers()["foo"], "bar");

    let prev_header = nextshell::reply::with::header("foo", "sean");
    let yes_header = nextshell::any()
        .map(nextshell::reply)
        .with(prev_header)
        .with(header);

    let req = nextshell::test::request();
    let resp = req.reply(&yes_header).await;
    assert_eq!(resp.headers()["foo"], "bar", "replaces header");
}

#[tokio::test]
async fn headers() {
    let mut headers = HeaderMap::new();
    headers.insert("server", HeaderValue::from_static("nextshell"));
    headers.insert("foo", HeaderValue::from_static("bar"));

    let headers = nextshell::reply::with::headers(headers);

    let no_header = nextshell::any().map(nextshell::reply).with(&headers);

    let req = nextshell::test::request();
    let resp = req.reply(&no_header).await;
    assert_eq!(resp.headers()["foo"], "bar");
    assert_eq!(resp.headers()["server"], "nextshell");

    let prev_header = nextshell::reply::with::header("foo", "sean");
    let yes_header = nextshell::any()
        .map(nextshell::reply)
        .with(prev_header)
        .with(headers);

    let req = nextshell::test::request();
    let resp = req.reply(&yes_header).await;
    assert_eq!(resp.headers()["foo"], "bar", "replaces header");
}

#[tokio::test]
async fn default_header() {
    let def_header = nextshell::reply::with::default_header("foo", "bar");

    let no_header = nextshell::any().map(nextshell::reply).with(&def_header);

    let req = nextshell::test::request();
    let resp = req.reply(&no_header).await;

    assert_eq!(resp.headers()["foo"], "bar");

    let header = nextshell::reply::with::header("foo", "sean");
    let yes_header = nextshell::any()
        .map(nextshell::reply)
        .with(header)
        .with(def_header);

    let req = nextshell::test::request();
    let resp = req.reply(&yes_header).await;

    assert_eq!(resp.headers()["foo"], "sean", "doesn't replace header");
}
