#![deny(warnings)]
use nextshell::Filter;
use std::convert::Infallible;

#[tokio::test]
async fn flattens_tuples() {
    let _ = pretty_env_logger::try_init();

    let str1 = nextshell::any().map(|| "nextshell");
    let true1 = nextshell::any().map(|| true);
    let unit1 = nextshell::any();

    // just 1 value
    let ext = nextshell::test::request().filter(&str1).await.unwrap();
    assert_eq!(ext, "nextshell");

    // just 1 unit
    let ext = nextshell::test::request().filter(&unit1).await.unwrap();
    assert_eq!(ext, ());

    // combine 2 values
    let and = str1.and(true1);
    let ext = nextshell::test::request().filter(&and).await.unwrap();
    assert_eq!(ext, ("nextshell", true));

    // combine 2 reversed
    let and = true1.and(str1);
    let ext = nextshell::test::request().filter(&and).await.unwrap();
    assert_eq!(ext, (true, "nextshell"));

    // combine 1 with unit
    let and = str1.and(unit1);
    let ext = nextshell::test::request().filter(&and).await.unwrap();
    assert_eq!(ext, "nextshell");

    let and = unit1.and(str1);
    let ext = nextshell::test::request().filter(&and).await.unwrap();
    assert_eq!(ext, "nextshell");

    // combine 3 values
    let and = str1.and(str1).and(true1);
    let ext = nextshell::test::request().filter(&and).await.unwrap();
    assert_eq!(ext, ("nextshell", "nextshell", true));

    // combine 2 with unit
    let and = str1.and(unit1).and(true1);
    let ext = nextshell::test::request().filter(&and).await.unwrap();
    assert_eq!(ext, ("nextshell", true));

    let and = unit1.and(str1).and(true1);
    let ext = nextshell::test::request().filter(&and).await.unwrap();
    assert_eq!(ext, ("nextshell", true));

    let and = str1.and(true1).and(unit1);
    let ext = nextshell::test::request().filter(&and).await.unwrap();
    assert_eq!(ext, ("nextshell", true));

    // nested tuples
    let str_true_unit = str1.and(true1).and(unit1);
    let unit_str_true = unit1.and(str1).and(true1);

    let and = str_true_unit.and(unit_str_true);
    let ext = nextshell::test::request().filter(&and).await.unwrap();
    assert_eq!(ext, ("nextshell", true, "nextshell", true));

    let and = unit_str_true.and(unit1).and(str1).and(str_true_unit);
    let ext = nextshell::test::request().filter(&and).await.unwrap();
    assert_eq!(ext, ("nextshell", true, "nextshell", "nextshell", true));
}

#[tokio::test]
async fn map() {
    let _ = pretty_env_logger::try_init();

    let ok = nextshell::any().map(nextshell::reply);

    let req = nextshell::test::request();
    let resp = req.reply(&ok).await;
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn or() {
    let _ = pretty_env_logger::try_init();

    // Or can be combined with an infallible filter
    let a = nextshell::path::param::<u32>();
    let b = nextshell::any().map(|| 41i32);
    let f = a.or(b);

    let _: Result<_, Infallible> = nextshell::test::request().filter(&f).await;
}

#[tokio::test]
async fn or_else() {
    let _ = pretty_env_logger::try_init();

    let a = nextshell::path::param::<u32>();
    let f = a.or_else(|_| async { Ok::<_, nextshell::Rejection>((44u32,)) });

    assert_eq!(
        nextshell::test::request()
            .path("/33")
            .filter(&f)
            .await
            .unwrap(),
        33,
    );
    assert_eq!(nextshell::test::request().filter(&f).await.unwrap(), 44,);

    // OrElse can be combined with an infallible filter
    let a = nextshell::path::param::<u32>();
    let f = a.or_else(|_| async { Ok::<_, Infallible>((44u32,)) });

    let _: Result<_, Infallible> = nextshell::test::request().filter(&f).await;
}

#[tokio::test]
async fn recover() {
    let _ = pretty_env_logger::try_init();

    let a = nextshell::path::param::<String>();
    let f = a.recover(|err| async move { Err::<String, _>(err) });

    // not rejected
    let resp = nextshell::test::request().path("/hi").reply(&f).await;
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.body(), "hi");

    // rejected, recovered, re-rejected
    let resp = nextshell::test::request().reply(&f).await;
    assert_eq!(resp.status(), 404);

    // Recover can be infallible
    let f = a.recover(|_| async move { Ok::<_, Infallible>("shh") });

    let _: Result<_, Infallible> = nextshell::test::request().filter(&f).await;
}

#[tokio::test]
async fn unify() {
    let _ = pretty_env_logger::try_init();

    let a = nextshell::path::param::<u32>();
    let b = nextshell::path::param::<u32>();
    let f = a.or(b).unify();

    let ex = nextshell::test::request()
        .path("/1")
        .filter(&f)
        .await
        .unwrap();

    assert_eq!(ex, 1);
}

#[should_panic]
#[tokio::test]
async fn nested() {
    let f = nextshell::any().and_then(|| async {
        let p = nextshell::path::param::<u32>();
        nextshell::test::request().filter(&p).await
    });

    let _ = nextshell::test::request().filter(&f).await;
}
