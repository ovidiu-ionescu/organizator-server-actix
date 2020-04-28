use actix_web::{HttpResponse, ResponseError};
use deadpool_postgres::PoolError;
use derive_more::{Display, From};
use tokio_pg_mapper::Error as PGMError;
use tokio_postgres::error::Error as PGError;

#[derive(Display, From, Debug)]
pub enum OrganizatorError {
	NotFound,
	PGError(PGError),
	PGMError(PGMError),
	PoolError(PoolError),
}
impl std::error::Error for OrganizatorError {}

impl ResponseError for OrganizatorError {
	fn error_response(&self) -> HttpResponse {
		match *self {
			OrganizatorError::NotFound => HttpResponse::NotFound().finish(),
			OrganizatorError::PoolError(ref err) => {
				HttpResponse::InternalServerError().body(err.to_string())
			}
			_ => HttpResponse::InternalServerError().finish(),
		}
	}
}
