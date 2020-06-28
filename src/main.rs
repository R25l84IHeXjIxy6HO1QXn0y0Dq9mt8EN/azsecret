mod backends;
mod constants;
mod ks;
mod message;
mod store;
mod util;

use store::StoreBackend;

use std::convert::Infallible;
// use std::fs::remove_file;
use std::path::Path;
use std::sync::Arc;

use hyper::Server;
use hyper::service::make_service_fn;
use hyperlocal::UnixServerExt;
use libc::{S_IRWXU, umask};
use ring::rand::{generate, SystemRandom};

fn init() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    unsafe {
        umask(!S_IRWXU);
    }

    // NOTE: coerce PRNG initialization to reduce latency later
    //       https://briansmith.org/rustdoc/ring/rand/struct.SystemRandom.html
    if let Err(_) = generate::<[u8; 8]>(&SystemRandom::new()) {
        panic!("FAILED TO INITIALIZE PRNG");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init()?;

    let skt = Path::new(constants::SOCKET_NAME);
    let store = Arc::new(backends::rocksdb::RocksDBBackend::new()?);
    
    Server::bind_unix(skt)?
        .serve(make_service_fn(move |_| {
            let store = store.clone();
            async move { <Result<_, Infallible>>::Ok(message::MessageService::new(store)) }
        }))
        .await?;

    Ok(())
}
