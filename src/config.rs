pub use ::config::ConfigError;
use serde::Deserialize;
use deadpool_postgres::{RecyclingMethod, ManagerConfig};

#[derive(Deserialize)]
pub struct Config {
	pub workers: usize,
	pub pg: deadpool_postgres::Config,
	pub bind: String,
	pub log_level: String,
	pub file_upload_dir: String,
}

impl Config {
	pub fn from_env() -> Result<Self, ConfigError> {
		let mut cfg = ::config::Config::new();
		cfg.merge(::config::Environment::new())?;
		let mut cfg: Config = cfg.try_into()?;
		cfg.pg.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });
		// println!("dbname: {:#?}", &cfg.pg);
		Ok(cfg)
	}
}

pub struct FileUploadConfig {
	pub dir: String,
}

impl Clone for FileUploadConfig {
	fn clone(&self) -> Self {
		FileUploadConfig { dir: self.dir.clone() }
	}
}