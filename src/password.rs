extern crate ring;
extern crate data_encoding;

use data_encoding::HEXUPPER;
use ring::error::Unspecified;
use ring::rand::SecureRandom;
use ring::{digest, pbkdf2, rand};
use std::num::NonZeroU32;

use crate::models::{Login};


pub fn verify_password(password: &str, login: &Login) -> bool {
  const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;
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

pub fn generate_key(key: &mut [u8; 32]) {
  let rng = rand::SystemRandom::new();
  rng.fill(key);
}