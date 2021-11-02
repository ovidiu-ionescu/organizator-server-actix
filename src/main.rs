use actix_web::{App, HttpServer};
use dotenv::dotenv;
use tokio_postgres::NoTls;

mod config;
mod db;
mod errors;
mod models;
mod password;
mod routes;

mod check_security_middleware;

use crate::config::{Config, FileUploadConfig };
use crate::password::generate_key;

use actix_session::CookieSession;
use actix_web::middleware::Logger;
use env_logger::Env;

// mod check_security_middleware;
use check_security_middleware::CheckSecurity;

/// Main test server, configurable via env variables:
/// DB_HOST - host name of PostgreSQL DB
/// WORKERS - number of workers (busy CPU cores)
/// POOL_SIZE - number of DB connections per worker (busy Postgres cores)
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    //env_logger::init();

    let config = Config::from_env().unwrap();
    env_logger::from_env(Env::default().default_filter_or(config.log_level)).init();

    let pool = config.pg.create_pool(NoTls).unwrap();
    let file_upload_config = FileUploadConfig {dir: config.file_upload_dir};

    let mut key = [0; 32];
    generate_key(&mut key);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(CheckSecurity)
            .wrap(CookieSession::signed(&key).secure(false))
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .data(pool.clone())
            .data(file_upload_config.clone())
            .service(routes::get_user)
            .service(routes::get_users)
            .service(routes::get_memo_titles)
            .service(routes::search_memo)
            .service(routes::get_memo)
            .service(routes::memo_write)
            .service(routes::get_memo_group)
            .service(routes::login)
            .service(routes::logout)
            .service(routes::change_password)
            .service(routes::version)
            .service(routes::upload_file)
            .service(routes::file_auth)
            .service(routes::explicit_permissions)
    })
    .bind(config.bind)?
    .workers(config.workers)
    .run();

    //info!("Server available at http://127.0.0.1:3002/");

    server.await
}
