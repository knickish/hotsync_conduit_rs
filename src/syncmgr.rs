use std::{ffi::CString, mem::MaybeUninit};

use dlopen2::wrapper::Container;
use palmrs::database::{PalmDatabase, PdbDatabase};

use crate::{
    error::{ConduitError, SyncManagerError},
    syncmgr_extern::{openDatabaseHandle, SyncMgrApi, CONDHANDLE},
    ConduitManager,
};

macro_rules! return_iff_conduit_err {
    ($expression:expr) => {{
        let ret = $expression;
        if ret != SyncManagerError::SYNCERR_NONE {
            return Err(ConduitError::Sync(ret));
        }
    }};
}

struct ConduitPdb {
    pdb_name: CString,
}

pub trait DatabaseGenerator {
    fn generate(self: Box<Self>) -> (CString, PalmDatabase<PdbDatabase>);
}

pub enum ConduitDBSource {
    Static(CString, PalmDatabase<PdbDatabase>),
    File(CString, std::path::PathBuf),
    Generator(Box<dyn DatabaseGenerator>),
    // add a way to pass modified/deleted recs here and update a db
}

impl ConduitDBSource {
    fn load_db_from_path(path: std::path::PathBuf) -> PalmDatabase<PdbDatabase> {
        let file_contents = std::fs::read(path).unwrap();
        let db = PalmDatabase::<PdbDatabase>::from_bytes(&file_contents).unwrap();
        db
    }

    fn get_db(self) -> (CString, PalmDatabase<PdbDatabase>) {
        match self {
            ConduitDBSource::Static(name, db) => (name, db),
            ConduitDBSource::File(name, path) => (name, Self::load_db_from_path(path)),
            ConduitDBSource::Generator(execute) => execute.generate(),
        }
    }
}

pub struct ConduitBuilder {
    name: CString,
    create_if_not_exists: Vec<ConduitDBSource>,
    overwrite: Vec<ConduitDBSource>,
    to_remove: Vec<CString>,
}

impl ConduitBuilder {
    pub fn new_with_name(conduit_name: impl Into<CString>) -> Self {
        Self {
            name: conduit_name.into(),
            create_if_not_exists: Vec::new(),
            overwrite: Vec::new(),
            to_remove: Vec::new(),
        }
    }

    /// Remove a database from the handheld
    pub fn remove_db(mut self, to_remove: CString) -> Self {
        self.to_remove.push(to_remove);
        self
    }

    /// Add a database, overwriting if it is already present
    pub fn write_db(mut self, source: ConduitDBSource) -> Self {
        self.overwrite.push(source);
        self
    }

    /// Add a database iff it is not present on the handheld
    pub fn create_db(mut self, to_remove: CString) -> Self {
        self.to_remove.push(to_remove);
        self
    }

    /// Build the conduit
    pub fn build(
        Self {
            name,
            create_if_not_exists,
            overwrite,
            to_remove,
        }: Self,
    ) -> Conduit {
        Conduit {
            name,
            create_if_not_exists: create_if_not_exists
                .into_iter()
                .map(ConduitDBSource::get_db)
                .collect(),
            overwrite: overwrite.into_iter().map(ConduitDBSource::get_db).collect(),
            to_remove,
        }
    }
}

pub struct Conduit {
    name: CString,
    create_if_not_exists: Vec<(CString, PalmDatabase<PdbDatabase>)>,
    overwrite: Vec<(CString, PalmDatabase<PdbDatabase>)>,
    to_remove: Vec<CString>,
}

impl Conduit {
    fn remove_db(to_remove: CString, sync_api: &Container<SyncMgrApi>) -> Result<(), ConduitError> {
        Ok(())
    }
    pub fn sync(self) -> Result<(), ConduitError> {
        Ok(())
    }
}

struct SyncSession {
    cond_mgr: ConduitManager,
    api: Container<SyncMgrApi>,
    open_cond: CONDHANDLE,
    open_db: Option<(CString, openDatabaseHandle)>,
}

impl SyncSession {
    fn init() -> Result<Self, ConduitError> {
        let cond_mgr = ConduitManager::initialize()?;
        let sync_mgr_dll_path = dbg!(cond_mgr.get_sync_mgr_dll_path()?);
        std::env::set_current_dir(sync_mgr_dll_path.parent().unwrap()).unwrap();
        let api: Container<SyncMgrApi> = unsafe { Container::load(sync_mgr_dll_path) }?;
        eprintln!("Loaded api");
        let mut open_cond_init = MaybeUninit::new(0);
        let open_cond;
        unsafe {
            return_iff_conduit_err!(api.SyncRegisterConduit(open_cond_init.as_mut_ptr()));
            open_cond = open_cond_init.assume_init();
        }
        Ok(Self {
            cond_mgr,
            api,
            open_cond,
            open_db: None,
        })
    }
    fn shutdown(self) -> Result<(), ConduitError> {
        unsafe {
            return_iff_conduit_err!(self.api.SyncUnRegisterConduit(self.open_cond));
        }
        Ok(())
    }
    fn log_to_hs_log(&self, line: CString) -> Result<(), ConduitError> {
        return_iff_conduit_err!(unsafe { self.api.SyncAddLogEntry(line.as_ptr()) });
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_load_api() {
        let cond_mgr = ConduitManager::initialize().unwrap();
        let sync_mgr_dll_path = dbg!(cond_mgr.get_sync_mgr_dll_path().unwrap());
        std::env::set_current_dir(sync_mgr_dll_path.parent().unwrap()).unwrap();
        let _: Container<SyncMgrApi> = unsafe { Container::load(sync_mgr_dll_path) }.unwrap();
    }
}
