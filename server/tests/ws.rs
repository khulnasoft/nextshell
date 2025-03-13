#![deny(warnings)]

use futures_util::{FutureExt, SinkExt, StreamExt};
use nextshell::ws::Message;
use nextshell::Filter;
use serde_derive::Deserialize;

#[tokio::test]
async fn upgrade() {
    let _ = pretty_env_logger::try_init();

    let route = nextshell::ws().map(|ws: nextshell::ws::Ws| ws.on_upgrade(|_| async {}));

    // From https://tools.ietf.org/html/rfc6455#section-1.2
    let key = "dGhlIHNhbXBsZSBub25jZQ==";
    let accept = "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=";

    let resp = nextshell::test::request()
        .header("connection", "upgrade")
        .header("upgrade", "websocket")
        .header("sec-websocket-version", "13")
        .header("sec-websocket-key", key)
        .reply(&route)
        .await;

    assert_eq!(resp.status(), 101);
    assert_eq!(resp.headers()["connection"], "upgrade");
    assert_eq!(resp.headers()["upgrade"], "websocket");
    assert_eq!(resp.headers()["sec-websocket-accept"], accept);

    let resp = nextshell::test::request()
        .header("connection", "keep-alive, Upgrade")
        .header("upgrade", "Websocket")
        .header("sec-websocket-version", "13")
        .header("sec-websocket-key", key)
        .reply(&route)
        .await;

    assert_eq!(resp.status(), 101);
}

#[tokio::test]
async fn fail() {
    let _ = pretty_env_logger::try_init();

    let route = nextshell::any().map(nextshell::reply);

    nextshell::test::ws()
        .handshake(route)
        .await
        .expect_err("handshake non-websocket route should fail");
}

#[tokio::test]
async fn text() {
    let _ = pretty_env_logger::try_init();

    let mut client = nextshell::test::ws()
        .handshake(ws_echo())
        .await
        .expect("handshake");

    client.send_text("hello nextshell").await;

    let msg = client.recv().await.expect("recv");
    assert_eq!(msg.to_str(), Ok("hello nextshell"));
}

#[tokio::test]
async fn binary() {
    let _ = pretty_env_logger::try_init();

    let mut client = nextshell::test::ws()
        .handshake(ws_echo())
        .await
        .expect("handshake");

    client
        .send(nextshell::ws::Message::binary(&b"bonk"[..]))
        .await;
    let msg = client.recv().await.expect("recv");
    assert!(msg.is_binary());
    assert_eq!(msg.as_bytes(), &b"bonk"[..]);
}

#[tokio::test]
async fn wsclient_sink_and_stream() {
    let _ = pretty_env_logger::try_init();

    let mut client = nextshell::test::ws()
        .handshake(ws_echo())
        .await
        .expect("handshake");

    let message = nextshell::ws::Message::text("hello");
    SinkExt::send(&mut client, message.clone()).await.unwrap();
    let received_message = client.next().await.unwrap().unwrap();
    assert_eq!(message, received_message);
}

#[tokio::test]
async fn close_frame() {
    let _ = pretty_env_logger::try_init();

    let route = nextshell::ws().map(|ws: nextshell::ws::Ws| {
        ws.on_upgrade(|mut websocket| async move {
            let msg = websocket.next().await.expect("item").expect("ok");
            let _ = msg.close_frame().expect("close frame");
        })
    });

    let client = nextshell::test::ws()
        .handshake(route)
        .await
        .expect("handshake");
    drop(client);
}

#[tokio::test]
async fn send_ping() {
    let _ = pretty_env_logger::try_init();

    let filter = nextshell::ws().map(|ws: nextshell::ws::Ws| {
        ws.on_upgrade(|mut websocket| {
            async move {
                websocket.send(Message::ping("srv")).await.unwrap();
                // assume the client will pong back
                let msg = websocket.next().await.expect("item").expect("ok");
                assert!(msg.is_pong());
                assert_eq!(msg.as_bytes(), &b"srv"[..]);
            }
        })
    });

    let mut client = nextshell::test::ws()
        .handshake(filter)
        .await
        .expect("handshake");

    let msg = client.recv().await.expect("recv");
    assert!(msg.is_ping());
    assert_eq!(msg.as_bytes(), &b"srv"[..]);

    client.recv_closed().await.expect("closed");
}

#[tokio::test]
async fn echo_pings() {
    let _ = pretty_env_logger::try_init();

    let mut client = nextshell::test::ws()
        .handshake(ws_echo())
        .await
        .expect("handshake");

    client.send(Message::ping("clt")).await;

    // tungstenite sends the PONG first
    let msg = client.recv().await.expect("recv");
    assert!(msg.is_pong());
    assert_eq!(msg.as_bytes(), &b"clt"[..]);

    // and then `ws_echo` sends us back the same PING
    let msg = client.recv().await.expect("recv");
    assert!(msg.is_ping());
    assert_eq!(msg.as_bytes(), &b"clt"[..]);

    // and then our client would have sent *its* PONG
    // and `ws_echo` would send *that* back too
    let msg = client.recv().await.expect("recv");
    assert!(msg.is_pong());
    assert_eq!(msg.as_bytes(), &b"clt"[..]);
}

#[tokio::test]
async fn pongs_only() {
    let _ = pretty_env_logger::try_init();

    let mut client = nextshell::test::ws()
        .handshake(ws_echo())
        .await
        .expect("handshake");

    // construct a pong message and make sure it is correct
    let msg = Message::pong("clt");
    assert!(msg.is_pong());
    assert_eq!(msg.as_bytes(), &b"clt"[..]);

    // send it to echo and wait for `ws_echo` to send it back
    client.send(msg).await;

    let msg = client.recv().await.expect("recv");
    assert!(msg.is_pong());
    assert_eq!(msg.as_bytes(), &b"clt"[..]);
}

#[tokio::test]
async fn closed() {
    let _ = pretty_env_logger::try_init();

    let route = nextshell::ws()
        .map(|ws: nextshell::ws::Ws| ws.on_upgrade(|websocket| websocket.close().map(|_| ())));

    let mut client = nextshell::test::ws()
        .handshake(route)
        .await
        .expect("handshake");

    client.recv_closed().await.expect("closed");
}

#[tokio::test]
async fn limit_message_size() {
    let _ = pretty_env_logger::try_init();

    let echo = nextshell::ws().map(|ws: nextshell::ws::Ws| {
        ws.max_message_size(1024).on_upgrade(|websocket| {
            // Just echo all messages back...
            let (tx, rx) = websocket.split();
            rx.forward(tx).map(|result| {
                assert!(result.is_err());
                assert_eq!(
                    format!("{}", result.unwrap_err()).as_str(),
                    "Space limit exceeded: Message too big: 0 + 1025 > 1024"
                );
            })
        })
    });
    let mut client = nextshell::test::ws()
        .handshake(echo)
        .await
        .expect("handshake");

    client
        .send(nextshell::ws::Message::binary(vec![0; 1025]))
        .await;
    client.send_text("hello nextshell").await;
    assert!(client.recv().await.is_err());
}

#[tokio::test]
async fn limit_frame_size() {
    let _ = pretty_env_logger::try_init();

    let echo = nextshell::ws().map(|ws: nextshell::ws::Ws| {
        ws.max_frame_size(1024).on_upgrade(|websocket| {
            // Just echo all messages back...
            let (tx, rx) = websocket.split();
            rx.forward(tx).map(|result| {
                assert!(result.is_err());
                assert_eq!(
                    format!("{}", result.unwrap_err()).as_str(),
                    "Space limit exceeded: Message length too big: 1025 > 1024"
                );
            })
        })
    });
    let mut client = nextshell::test::ws()
        .handshake(echo)
        .await
        .expect("handshake");

    client
        .send(nextshell::ws::Message::binary(vec![0; 1025]))
        .await;
    client.send_text("hello nextshell").await;
    assert!(client.recv().await.is_err());
}

#[derive(Deserialize)]
struct MyQuery {
    hello: String,
}

#[tokio::test]
async fn ws_with_query() {
    let ws_filter = nextshell::path("my-ws")
        .and(nextshell::query::<MyQuery>())
        .and(nextshell::ws())
        .map(|query: MyQuery, ws: nextshell::ws::Ws| {
            assert_eq!(query.hello, "world");

            ws.on_upgrade(|websocket| {
                let (tx, rx) = websocket.split();
                rx.inspect(|i| log::debug!("ws recv: {:?}", i))
                    .forward(tx)
                    .map(|_| ())
            })
        });

    nextshell::test::ws()
        .path("/my-ws?hello=world")
        .handshake(ws_filter)
        .await
        .expect("handshake");
}

// Websocket filter that echoes all messages back.
fn ws_echo() -> impl Filter<Extract = (impl nextshell::Reply,), Error = nextshell::Rejection> + Copy
{
    nextshell::ws().map(|ws: nextshell::ws::Ws| {
        ws.on_upgrade(|websocket| {
            // Just echo all messages back...
            let (tx, rx) = websocket.split();
            rx.inspect(|i| log::debug!("ws recv: {:?}", i))
                .forward(tx)
                .map(|_| ())
        })
    })
}
