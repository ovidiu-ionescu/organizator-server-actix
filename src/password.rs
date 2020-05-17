extern crate ring;
extern crate data_encoding;

use ring::error::Unspecified;
use ring::rand::SecureRandom;
use ring::{digest, pbkdf2, rand};
use std::num::NonZeroU32;
use data_encoding::HEXUPPER;
use log::debug;

use crate::{
  models::{Login},
  errors::OrganizatorError
};


pub fn verify_password(password: &str, login: &Login) -> bool {
  let n_iter = NonZeroU32::new(100_000).unwrap();

  let should_succeed = pbkdf2::verify(
    pbkdf2::PBKDF2_HMAC_SHA512,
    n_iter,
    &login.salt,
    password.as_bytes(),
    &login.pbkdf2,
  );

  should_succeed.is_ok()
  
}

pub fn generate_key(key: &mut [u8; 32]) -> Result<(), OrganizatorError> {
  let rng = rand::SystemRandom::new();
  rng.fill(key)?;
  Ok(())
}

pub const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;

pub fn compute_new_password(password: &str, salt: &mut Vec<u8>, pbkdf2_hash: &mut Vec<u8>) -> Result<(), Unspecified> {
  
  let n_iter = NonZeroU32::new(100_000).unwrap();
  let rng = rand::SystemRandom::new();
  rng.fill(salt)?;

  pbkdf2::derive(
      pbkdf2::PBKDF2_HMAC_SHA512,
      n_iter,
      &salt,
      password.as_bytes(),
      pbkdf2_hash,
  );
  debug!("Salt: {}, len {}", HEXUPPER.encode(&salt), salt.len());
  Ok(())
}