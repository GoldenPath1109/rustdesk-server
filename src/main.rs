// https://tools.ietf.org/rfc/rfc5128.txt
// https://blog.csdn.net/bytxl/article/details/44344855

use hbb_common::{env_logger::*, log, tokio, ResultType};
use hbbs::*;

#[tokio::main]
async fn main() -> ResultType<()> {
    init_from_env(Env::default().filter_or(DEFAULT_FILTER_ENV, "info"));
    let addr = "0.0.0.0:21116";
    log::info!("Listening on {}", addr);
    RendezvousServer::start(&addr).await?;
    Ok(())
}
