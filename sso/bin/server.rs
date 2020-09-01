//! # sso_server
//!
//! ## Configuration
//!
//! See [Config](../sso/struct.Config.html).
//!
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use clap::{App, Arg};

const ARG_CONFIG: &str = "config";

#[tokio::main]
async fn main() {
    let matches = App::new("sso-server")
        .version(crate_version!())
        .author("Sam Ward <mail@mojzu.net>")
        .arg(
            Arg::with_name(ARG_CONFIG)
                .long("config")
                .alias("c")
                .takes_value(true)
                .required(false),
        )
        .get_matches();

    let config_name = matches.value_of(ARG_CONFIG).unwrap_or(".config/sso");
    let config =
        sso::Config::from_env(config_name).expect("parse configuration from environment failure");
    let config = config
        .load_templates()
        .await
        .expect("load template files failure");

    sso::init_panic(config.log.pretty);
    sso::init_log(config.log.pretty);

    let local = tokio::task::LocalSet::new();
    let sys = actix_rt::System::run_in_tokio("server", &local);
    local
        .run_until(async move {
            debug!("start");

            let server = sso::Server::from_config(config)
                .await
                .expect("create server from environment failure");

            let http_public = server
                .public_service()
                .expect("create http public server failure");

            let http_private = server
                .private_service()
                .expect("create http private server failure");

            tokio::task::spawn_local(async move {
                signal_int_term().await;
                debug!("stop");

                http_public.stop(true).await;
                http_private.stop(true).await;
                actix_rt::System::current().stop();
            });

            info!(
                "public interface listening on http://{}",
                server.config.http.public.bind
            );
            info!(
                "private interface listening on http://{}",
                server.config.http.private.bind
            );
            sys.await.expect("actix system failure");
        })
        .await;
}

async fn signal_int_term() {
    use tokio::signal::unix::{signal, SignalKind};

    let mut int = signal(SignalKind::interrupt()).expect("SIGINT stream failure");
    let mut term = signal(SignalKind::terminate()).expect("SIGTERM stream failure");

    loop {
        tokio::select! {
            _ = int.recv() => {
                debug!("SIGINT signal");
                break;
            }
            _ = term.recv() => {
                debug!("SIGTERM signal");
                break;
            }
        };
    }
}
