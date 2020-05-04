use actix_web::{App, HttpServer};
use dotenv::dotenv;
use tokio_postgres::NoTls;

mod config;
mod routes;
mod models;
mod errors;
mod db;

mod check_security_middleware;

use crate::config::Config;

use actix_web::middleware::Logger;
use env_logger::Env;

// mod check_security_middleware;
use check_security_middleware::{ CheckSecurity };

/// Main test server, configurable via env variables:
/// DB_HOST - host name of PostgreSQL DB
/// WORKERS - number of workers (busy CPU cores)
/// POOL_SIZE - number of DB connections per worker (busy Postgres cores)
#[actix_rt::main]
async fn main() -> std::io::Result<()> {

    dotenv().ok();
    //env_logger::init();
    env_logger::from_env(Env::default().default_filter_or("debug")).init();

    let config = Config::from_env().unwrap();
    let pool = config.pg.create_pool(NoTls).unwrap();


    let server = HttpServer::new(move || {
        App::new()
            .wrap(CheckSecurity)
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .data(pool.clone())
            .service(routes::get_user)
            .service(routes::get_memo_titles)
            .service(routes::search_memo)
            .service(routes::get_memo)

    })
    .bind("127.0.0.1:3002")?
    .workers(config.workers)
    .run();

    //info!("Server available at http://127.0.0.1:3002/");

    server.await

}
