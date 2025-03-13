#![deny(warnings)]

// Don't copy this `cfg`, it's only needed because this file is within
// the nextshell repository.
// Instead, specify the "tls" feature in your nextshell dependency declaration.
#[cfg(feature = "tls")]
#[tokio::main]
async fn main() {
    use nextshell::Filter;

    // Match any request and return hello world!
    let routes = nextshell::any().map(|| "Hello, World!");

    nextshell::serve(routes)
        .tls()
        // RSA
        .cert_path("examples/tls/cert.pem")
        .key_path("examples/tls/key.rsa")
        // ECC
        // .cert_path("examples/tls/cert.ecc.pem")
        // .key_path("examples/tls/key.ecc")
        .run(([127, 0, 0, 1], 3030))
        .await;
}

#[cfg(not(feature = "tls"))]
fn main() {
    eprintln!("Requires the `tls` feature.");
}
