mod port;
mod session;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

pub use self::session::Session;
pub type Sessions = Arc<Mutex<HashMap<String, Arc<Mutex<Session>>>>>;
pub use self::port::Port;
