#![deny(warnings)]

use std::net::SocketAddr;
use std::sync::Arc;

use bytes::Bytes;
// use futures_util::TryStreamExt;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Error, Response};
use hyper_util::rt::TokioIo;
use surf;
use tokio::net::TcpListener;
// use tokio_util::io::ReaderStream;

// #[path = "../benches/support/mod.rs"]
// mod support;
// use support::TokioIo;

struct Drivers {
  mc: memcache::Client,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
  pretty_env_logger::init();

  let addr: SocketAddr = "127.0.0.1:1337".parse().unwrap();

  let listener = TcpListener::bind(addr).await?;
  println!("Listening on http://{}", addr);

  let drv = Arc::new(Drivers {
    mc: memcache::connect("memcache://127.0.0.1:11211?timeout=10&tcp_nodelay=true").unwrap(),
  });

  loop {
    let (stream, _) = listener.accept().await?;
    let io = TokioIo::new(stream);
    let drv = drv.clone();

    tokio::task::spawn(async move {
      if let Err(err) = http1::Builder::new()
        .serve_connection(
          io,
          service_fn(move |_req| {
            // Get the current count, and also increment by 1, in a single
            // atomic operation.
            let drv = drv.clone();
            async move {
              let k = "";
              let dat = match drv.mc.get::<Vec<u8>>(k) {
                Ok(d) => d,
                Err(_) => {
                  let r: Arc<Option<Vec<u8>>> = Arc::new(match fetch().await {
                    Ok(d) => Some(d),
                    Err(_) => None,
                  });
                  tokio::task::spawn(async move {
                    // let drv = drv.clone();
                    // let r = r.clone();
                    // this is ro, so please :)
                    // unsafe { drv.mc.set(k, r.clone(), 1024) }
                  });
                  r.as_ref().to_owned()
                }
              };
              if dat.is_none() {
                return Ok(Response::new(Full::new(Bytes::from(""))));
              }
              // Ok::<_, Error>(Response::new(Full::new(Bytes::from(format!("Request",)))))
              Ok::<_, Error>(Response::new(Full::new(Bytes::from(
                dat.unwrap().to_owned(),
              ))))
            }
          }),
        )
        .await
      {
        println!("Failed to serve connection: {:?}", err);
      }
    });
  }
}

async fn fetch() -> std::result::Result<Vec<u8>, surf::Error> {
  Ok(
    // def purge this ero thing later
    surf::get("https://i3.hhentai.net/images/2023/11v2/12/1699783384-002.jpg?imgmax=1200")
      .header("referer", "https://hentaivn.autos/")
      .send()
      .await?
      .body_bytes()
      .await?,
  )
}
