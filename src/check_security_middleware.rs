use std::pin::Pin;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::{dev::{ServiceRequest, ServiceResponse, Extensions, Payload}, Error, FromRequest, HttpMessage, HttpRequest};
use futures::future::{ok, Ready};
use futures::Future;

use log::{debug};

#[derive(Debug, Clone)]
pub struct Security {
    pub user_name: Option<String>,
    // pub user_name: usize,
}

impl Security {
    pub fn from(s: &str) -> Security {
        Security { user_name: Some(String::from(s))}
        // Security{ user_name: 2}
    }

    fn get_security(extensions: &mut Extensions) -> Security {
        match extensions.get::<Security>() {
            Some(s) => s.clone(),
            None => {
                println!("No security in here");
                Security::from("aha")
            }
        }
    }
}

impl Drop for Security {
    fn drop(&mut self) {
        //println!("Dropping security");
    }
}

impl FromRequest for Security {
    type Error = Error;
    type Future = Ready<Result<Security, Error>>;
    type Config = ();

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {        
        ok(Security::get_security(&mut *req.extensions_mut()))
    }
}


pub struct CheckSecurity;

// Middleware factory
impl<S, B> Transform<S> for CheckSecurity
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CheckSecurityMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CheckSecurityMiddleware { service })
    }
}

pub struct CheckSecurityMiddleware<S> {
  service: S,
}

impl<S, B> Service for CheckSecurityMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let a = req.headers().get("X-SSL-Client-S-DN");
        let user_name = a.unwrap().to_str().unwrap();
        debug!("X-SSL-Client-S-DN {}", user_name);

        req.extensions_mut().insert(Security::from(user_name));

        //println!("Hi from start. You requested: {}", req.path());

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            //println!("Hi from response");
            Ok(res)
        })
    }
}