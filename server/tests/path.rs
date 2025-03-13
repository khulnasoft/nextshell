#![deny(warnings)]
#[macro_use]
extern crate nextshell;

use futures_util::future;
use nextshell::Filter;

#[tokio::test]
async fn path() {
    let _ = pretty_env_logger::try_init();

    let foo = nextshell::path("foo");
    let bar = nextshell::path(String::from("bar"));
    let foo_bar = foo.and(bar.clone());

    // /foo
    let foo_req = || nextshell::test::request().path("/foo");

    assert!(foo_req().matches(&foo).await);
    assert!(!foo_req().matches(&bar).await);
    assert!(!foo_req().matches(&foo_bar).await);

    // /foo/bar
    let foo_bar_req = || nextshell::test::request().path("/foo/bar");

    assert!(foo_bar_req().matches(&foo).await);
    assert!(!foo_bar_req().matches(&bar).await);
    assert!(foo_bar_req().matches(&foo_bar).await);
}

#[tokio::test]
async fn param() {
    let _ = pretty_env_logger::try_init();

    let num = nextshell::path::param::<u32>();

    let req = nextshell::test::request().path("/321");
    assert_eq!(req.filter(&num).await.unwrap(), 321);

    let s = nextshell::path::param::<String>();

    let req = nextshell::test::request().path("/nextshell");
    assert_eq!(req.filter(&s).await.unwrap(), "nextshell");

    // u32 doesn't extract a non-int
    let req = nextshell::test::request().path("/nextshell");
    assert!(!req.matches(&num).await);

    let combo = num.map(|n| n + 5).and(s);

    let req = nextshell::test::request().path("/42/vroom");
    assert_eq!(req.filter(&combo).await.unwrap(), (47, "vroom".to_string()));

    // empty segments never match
    let req = nextshell::test::request();
    assert!(
        !req.matches(&s).await,
        "param should never match an empty segment"
    );
}

#[tokio::test]
async fn end() {
    let _ = pretty_env_logger::try_init();

    let foo = nextshell::path("foo");
    let end = nextshell::path::end();
    let foo_end = foo.and(end);

    assert!(
        nextshell::test::request().path("/").matches(&end).await,
        "end() matches /"
    );

    assert!(
        nextshell::test::request()
            .path("http://localhost:1234")
            .matches(&end)
            .await,
        "end() matches /"
    );

    assert!(
        nextshell::test::request()
            .path("http://localhost:1234?q=2")
            .matches(&end)
            .await,
        "end() matches empty path"
    );

    assert!(
        nextshell::test::request()
            .path("localhost:1234")
            .matches(&end)
            .await,
        "end() matches authority-form"
    );

    assert!(
        !nextshell::test::request().path("/foo").matches(&end).await,
        "end() doesn't match /foo"
    );

    assert!(
        nextshell::test::request()
            .path("/foo")
            .matches(&foo_end)
            .await,
        "path().and(end()) matches /foo"
    );

    assert!(
        nextshell::test::request()
            .path("/foo/")
            .matches(&foo_end)
            .await,
        "path().and(end()) matches /foo/"
    );
}

#[tokio::test]
async fn tail() {
    let tail = nextshell::path::tail();

    // matches full path
    let ex = nextshell::test::request()
        .path("/42/vroom")
        .filter(&tail)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "42/vroom");

    // matches index
    let ex = nextshell::test::request()
        .path("/")
        .filter(&tail)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "");

    // doesn't include query
    let ex = nextshell::test::request()
        .path("/foo/bar?baz=quux")
        .filter(&tail)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "foo/bar");

    // doesn't include previously matched prefix
    let and = nextshell::path("foo").and(tail);
    let ex = nextshell::test::request()
        .path("/foo/bar")
        .filter(&and)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "bar");

    // sets unmatched path index to end
    let m = tail.and(nextshell::path("foo"));
    assert!(
        !nextshell::test::request()
            .path("/foo/bar")
            .matches(&m)
            .await
    );

    let m = tail.and(nextshell::path::end());
    assert!(
        nextshell::test::request()
            .path("/foo/bar")
            .matches(&m)
            .await
    );

    let ex = nextshell::test::request()
        .path("localhost")
        .filter(&tail)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "/");
}

#[tokio::test]
async fn or() {
    let _ = pretty_env_logger::try_init();

    // /foo/bar OR /foo/baz
    let foo = nextshell::path("foo");
    let bar = nextshell::path("bar");
    let baz = nextshell::path("baz");
    let p = foo.and(bar.or(baz));

    // /foo/bar
    let req = nextshell::test::request().path("/foo/bar");

    assert!(req.matches(&p).await);

    // /foo/baz
    let req = nextshell::test::request().path("/foo/baz");

    assert!(req.matches(&p).await);

    // deeper nested ORs
    // /foo/bar/baz OR /foo/baz/bar OR /foo/bar/bar
    let p = foo
        .and(bar.and(baz).map(|| panic!("shouldn't match")))
        .or(foo.and(baz.and(bar)).map(|| panic!("shouldn't match")))
        .or(foo.and(bar.and(bar)));

    // /foo/baz
    let req = nextshell::test::request().path("/foo/baz/baz");
    assert!(!req.matches(&p).await);

    // /foo/bar/bar
    let req = nextshell::test::request().path("/foo/bar/bar");
    assert!(req.matches(&p).await);
}

#[tokio::test]
async fn or_else() {
    let _ = pretty_env_logger::try_init();

    let foo = nextshell::path("foo");
    let bar = nextshell::path("bar");

    let p = foo.and(bar.or_else(|_| future::ok::<_, std::convert::Infallible>(())));

    // /foo/bar
    let req = nextshell::test::request().path("/foo/nope");

    assert!(req.matches(&p).await);
}

#[tokio::test]
async fn path_macro() {
    let _ = pretty_env_logger::try_init();

    let req = nextshell::test::request().path("/foo/bar");
    let p = path!("foo" / "bar");
    assert!(req.matches(&p).await);

    let req = nextshell::test::request().path("/foo/bar");
    let p = path!(String / "bar");
    assert_eq!(req.filter(&p).await.unwrap(), "foo");

    let req = nextshell::test::request().path("/foo/bar");
    let p = path!("foo" / String);
    assert_eq!(req.filter(&p).await.unwrap(), "bar");

    // Requires path end

    let req = nextshell::test::request().path("/foo/bar/baz");
    let p = path!("foo" / "bar");
    assert!(!req.matches(&p).await);

    let req = nextshell::test::request().path("/foo/bar/baz");
    let p = path!("foo" / "bar").and(nextshell::path("baz"));
    assert!(!req.matches(&p).await);

    // Prefix syntax

    let req = nextshell::test::request().path("/foo/bar/baz");
    let p = path!("foo" / "bar" / ..);
    assert!(req.matches(&p).await);

    let req = nextshell::test::request().path("/foo/bar/baz");
    let p = path!("foo" / "bar" / ..).and(nextshell::path!("baz"));
    assert!(req.matches(&p).await);

    // Empty

    let req = nextshell::test::request().path("/");
    let p = path!();
    assert!(req.matches(&p).await);

    let req = nextshell::test::request().path("/foo");
    let p = path!();
    assert!(!req.matches(&p).await);
}

#[tokio::test]
async fn full_path() {
    let full_path = nextshell::path::full();

    let foo = nextshell::path("foo");
    let bar = nextshell::path("bar");
    let param = nextshell::path::param::<u32>();

    // matches full request path
    let ex = nextshell::test::request()
        .path("/42/vroom")
        .filter(&full_path)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "/42/vroom");

    // matches index
    let ex = nextshell::test::request()
        .path("/")
        .filter(&full_path)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "/");

    // does not include query
    let ex = nextshell::test::request()
        .path("/foo/bar?baz=quux")
        .filter(&full_path)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "/foo/bar");

    // includes previously matched prefix
    let and = foo.and(full_path);
    let ex = nextshell::test::request()
        .path("/foo/bar")
        .filter(&and)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "/foo/bar");

    // includes following matches
    let and = full_path.and(foo);
    let ex = nextshell::test::request()
        .path("/foo/bar")
        .filter(&and)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "/foo/bar");

    // includes previously matched param
    let and = foo.and(param).and(full_path);
    let (_, ex) = nextshell::test::request()
        .path("/foo/123")
        .filter(&and)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "/foo/123");

    // does not modify matching
    let m = full_path.and(foo).and(bar);
    assert!(
        nextshell::test::request()
            .path("/foo/bar")
            .matches(&m)
            .await
    );

    // doesn't panic on authority-form
    let ex = nextshell::test::request()
        .path("localhost:1234")
        .filter(&full_path)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "/");
}

#[tokio::test]
async fn peek() {
    let peek = nextshell::path::peek();

    let foo = nextshell::path("foo");
    let bar = nextshell::path("bar");
    let param = nextshell::path::param::<u32>();

    // matches full request path
    let ex = nextshell::test::request()
        .path("/42/vroom")
        .filter(&peek)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "42/vroom");

    // matches index
    let ex = nextshell::test::request()
        .path("/")
        .filter(&peek)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "");

    // does not include query
    let ex = nextshell::test::request()
        .path("/foo/bar?baz=quux")
        .filter(&peek)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "foo/bar");

    // does not include previously matched prefix
    let and = foo.and(peek);
    let ex = nextshell::test::request()
        .path("/foo/bar")
        .filter(&and)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "bar");

    // includes following matches
    let and = peek.and(foo);
    let ex = nextshell::test::request()
        .path("/foo/bar")
        .filter(&and)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "foo/bar");

    // does not include previously matched param
    let and = foo.and(param).and(peek);
    let (_, ex) = nextshell::test::request()
        .path("/foo/123")
        .filter(&and)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "");

    // does not modify matching
    let and = peek.and(foo).and(bar);
    assert!(
        nextshell::test::request()
            .path("/foo/bar")
            .matches(&and)
            .await
    );
}

#[tokio::test]
async fn peek_segments() {
    let peek = nextshell::path::peek();

    // matches full request path
    let ex = nextshell::test::request()
        .path("/42/vroom")
        .filter(&peek)
        .await
        .unwrap();

    assert_eq!(ex.segments().collect::<Vec<_>>(), &["42", "vroom"]);

    // matches index
    let ex = nextshell::test::request()
        .path("/")
        .filter(&peek)
        .await
        .unwrap();

    let segs = ex.segments().collect::<Vec<_>>();
    assert_eq!(segs, Vec::<&str>::new());
}
