use actix_web::{HttpResponse, ResponseError};
use deadpool_postgres::PoolError;
use derive_more::{Display, From};
use std::error::Error;
use tokio_pg_mapper::Error as PGMError;
use tokio_postgres::error::Error as PGError;

use tokio_postgres::error::DbError;
use ring::error::Unspecified;
use actix_http::error::Error as ActixHttpError;

use log::error;
use actix_threadpool;

#[derive(Display, From, Debug)]
pub enum OrganizatorError {
	NotFound,
	PGError(PGError),
	PGMError(PGMError),
	PoolError(PoolError),
	Internal,
	BlockingError,
}
impl std::error::Error for OrganizatorError {}

impl ResponseError for OrganizatorError {
	fn error_response(&self) -> HttpResponse {
		match *self {
			OrganizatorError::NotFound => HttpResponse::NotFound().finish(),
			OrganizatorError::PoolError(ref err) => {
				HttpResponse::InternalServerError().body(err.to_string())
			}
			OrganizatorError::PGError(ref err) => {
				let sql_state = err.source().unwrap().downcast_ref::<DbError>().unwrap().code();
				match sql_state.code() {
					"2F004" => HttpResponse::Forbidden().body(err.to_string()),
					"28000" => HttpResponse::Unauthorized().body(err.to_string()),
					"02000" => HttpResponse::NotFound().body(err.to_string()),
					_ => HttpResponse::InternalServerError().body(err.to_string()), 
				}

				
			}
			_ => HttpResponse::InternalServerError().finish(),
		}
	}
}

impl From<Unspecified> for OrganizatorError {
	fn from(unspecified_error: Unspecified) -> Self {
		error!("Got an unspecified error {:#?}", &unspecified_error);
		OrganizatorError::Internal
	}
}

impl From<actix_http::error::Error> for OrganizatorError {
	fn from (error: ActixHttpError) -> Self {
		error!("Got an actix error {:#?}", &error);
		OrganizatorError::Internal
	}
}

impl From<actix_threadpool::BlockingError<std::io::Error>> for OrganizatorError {
	fn from (error: actix_threadpool::BlockingError<std::io::Error>) -> Self {
		error!("Got a blocking error {:#?}", error);
		OrganizatorError::BlockingError
	}
}

impl From<std::string::FromUtf8Error> for OrganizatorError {
	fn from (error: std::string::FromUtf8Error) -> Self {
		error!("Got a utf8 error {:#?}", error);
		OrganizatorError::BlockingError
	}
}