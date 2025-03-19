use once_cell::sync::OnceCell;
use std::error::Error;
use std::sync::Arc;
use zenoh::{Config, Session, Wait};

struct SessionManager {
    session: Arc<Session>,
}

impl SessionManager {
    fn initialize() -> Result<Self, zenoh::Error> {
        let session: Arc<Session> = match std::env::var("COMMUNICATION_CONFIG") {
            Ok(env) => {
                match Config::from_json5(&env) {
                    Ok(config) => {
                        match zenoh::open(config).wait() {
                            Ok(session) => Arc::new(session),
                            Err(e) => { return Err(e.into()); }
                        }
                    }
                    Err(e) => { return Err(e.into()); }
                }
            }
            Err(std::env::VarError::NotPresent) => {
                match zenoh::open(Config::default()).wait() {
                    Ok(session) => Arc::new(session),
                    Err(e) => { return Err(e.into()); }
                }
            }
            Err(e) => { return Err(e.into()); }
        };

        Ok(SessionManager {
            session,
        })
    }

    fn get_session(&self) -> Arc<Session> {
        Arc::clone(&self.session)
    }
}

static SESSION_MANAGER: OnceCell<SessionManager> = OnceCell::new();

pub(crate) fn get_session() -> Arc<Session> {
    match SESSION_MANAGER
        .get() {
        Some(manager) => manager.get_session(),
        None => panic!("SessionManager is not initialized"),
    }
}

pub(crate) fn initialize() -> Result<(), Box<dyn Error + Send + Sync>> {
    let manager = SessionManager::initialize()?;
    SESSION_MANAGER
        .set(manager)
        .map_err(|_| "SessionManager is already initialized")?;
    Ok(())
}
