use std::{
    error::Error,
    ffi::{c_uchar, CString},
    mem::MaybeUninit,
};

use dlopen2::wrapper::Container;
use log::info;
use palmrs::database::{
    record::{pdb_record::RecordAttributes, DatabaseRecord},
    PalmDatabase, PdbDatabase,
};

use crate::{
    error::{ConduitError, SyncManagerError},
    syncmgr_extern::{
        eDbOpenModes, openDatabaseHandle, CDbCreateDB, CRawPreferenceInfo, CRawRecordInfo,
        SyncMgrApi, CONDHANDLE, DB_NAMELEN,
    },
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

fn uchars_to_u32(creator: [c_uchar; 4]) -> u32 {
    u32::from_be_bytes(creator)
}

pub trait DatabaseGenerator {
    fn generate(self: Box<Self>) -> (CString, [c_uchar; 4], PalmDatabase<PdbDatabase>);
}

pub enum PreferenceType<T> {
    Static(u16, T),
    Dynamic(u16, Box<dyn Fn(Option<T>) -> Option<T>>),
}

/// Need the database name (with no extension), the type code, and the db itself
pub enum ConduitDBSource {
    Static(CString, [c_uchar; 4], PalmDatabase<PdbDatabase>),
    File(CString, [c_uchar; 4], std::path::PathBuf),
    Generator(Box<dyn DatabaseGenerator>),
    // add a way to pass modified/deleted recs here and update a db
}

/// Need the database name (with no extension), the type code, and the db itself
pub enum ConduitDBSink {
    Dynamic(WorkOnDbType),
    // add a way to pass modified/deleted recs here and update a db
}

impl ConduitDBSource {
    fn load_db_from_path(path: std::path::PathBuf) -> PalmDatabase<PdbDatabase> {
        let file_contents = std::fs::read(path).unwrap();
        let db = PalmDatabase::<PdbDatabase>::from_bytes(&file_contents).unwrap();
        db
    }

    fn get_db(self) -> (CString, u32, PalmDatabase<PdbDatabase>) {
        match self {
            ConduitDBSource::Static(name, arr, db) => (name, uchars_to_u32(arr), db),
            ConduitDBSource::File(name, arr, path) => {
                (name, uchars_to_u32(arr), Self::load_db_from_path(path))
            }
            ConduitDBSource::Generator(execute) => {
                let (name, arr, db) = execute.generate();
                (name, uchars_to_u32(arr), db)
            }
        }
    }
}

type WorkOnDbType = Box<
    dyn FnMut(Vec<(Vec<u8>, RecordAttributes, u32)>) -> Result<(), Box<dyn Error + Sync + Send>>,
>;

pub struct ConduitBuilder<Preferences: TryInto<Vec<u8>> + TryFrom<Vec<u8>> = Vec<u8>> {
    name: CString,
    creator_id: u32,
    create_if_not_exists: Vec<ConduitDBSource>,
    overwrite: Vec<ConduitDBSource>,

    to_remove: Vec<CString>,
    to_download: Vec<(CString, ConduitDBSink)>,
    preferences: Option<PreferenceType<Preferences>>,
}

impl<Preferences: TryInto<Vec<u8>> + TryFrom<Vec<u8>>> ConduitBuilder<Preferences> {
    /// Create a new conduit. The creator field must match the CreatorID used in your application
    pub fn new_with_name_creator(conduit_name: impl Into<CString>, creator: [c_uchar; 4]) -> Self {
        Self {
            name: conduit_name.into(),
            creator_id: uchars_to_u32(creator),
            create_if_not_exists: Vec::new(),
            overwrite: Vec::new(),
            to_remove: Vec::new(),
            to_download: Vec::new(),
            preferences: None,
        }
    }

    /// Remove a database from the handheld, if it exists
    pub fn remove_db(mut self, to_remove: CString) -> Self {
        self.to_remove.push(to_remove);
        self
    }

    /// Download the records from a database on the handheld
    pub fn download_db_and(mut self, to_download: CString, do_work: ConduitDBSink) -> Self {
        self.to_download.push((to_download, do_work));
        self
    }

    /// Add a database to the handheld, overwriting if already present
    pub fn overwrite_db(mut self, source: ConduitDBSource) -> Self {
        self.overwrite.push(source);
        self
    }

    /// Add a database iff not present on the handheld
    pub fn create_db(mut self, source: ConduitDBSource) -> Self {
        self.create_if_not_exists.push(source);
        self
    }

    /// Set the preferences for the application
    pub fn set_preferences(mut self, source: PreferenceType<Preferences>) -> Self {
        self.preferences = Some(source);
        self
    }

    /// Build the conduit
    pub fn build(self) -> Conduit<Preferences> {
        let Self {
            name,
            creator_id,
            create_if_not_exists,
            overwrite,
            to_remove,
            to_download,
            preferences,
            ..
        } = self;
        Conduit {
            name,
            creator_id,
            create_if_not_exists: create_if_not_exists
                .into_iter()
                .map(ConduitDBSource::get_db)
                .collect(),
            overwrite: overwrite.into_iter().map(ConduitDBSource::get_db).collect(),
            to_remove,
            to_download,
            preferences,
        }
    }
}

pub struct Conduit<Preferences: TryInto<Vec<u8>> + TryFrom<Vec<u8>> = Vec<u8>> {
    name: CString,
    creator_id: u32,
    create_if_not_exists: Vec<(CString, u32, PalmDatabase<PdbDatabase>)>,
    overwrite: Vec<(CString, u32, PalmDatabase<PdbDatabase>)>,

    to_remove: Vec<CString>,
    to_download: Vec<(CString, ConduitDBSink)>,
    preferences: Option<PreferenceType<Preferences>>,
}

impl<Preferences: TryInto<Vec<u8>> + TryFrom<Vec<u8>>> Conduit<Preferences> {
    fn get_existing_prefs(
        sync: &SyncSession,
        creator: u32,
        pref_id: u16,
    ) -> Result<Option<Preferences>, ConduitError> {
        let mut bytes = vec![0_u8; 1024];

        let mut to_fill = MaybeUninit::new(CRawPreferenceInfo::new_with_buffer(
            0, creator, pref_id, &mut bytes,
        ));
        let mut ret_val = unsafe { sync.api.SyncReadAppPreference(to_fill.as_mut_ptr()) };

        // retry with the correct buffer size if too small
        if ret_val == SyncManagerError::SYNCERR_LOCAL_BUFF_TOO_SMALL {
            let new_size = unsafe { to_fill.assume_init_ref().get_required_size() } as usize;
            bytes.resize(new_size, 0_u8);
            to_fill = MaybeUninit::new(CRawPreferenceInfo::new_with_buffer(
                0, creator, pref_id, &mut bytes,
            ));
            ret_val = unsafe { sync.api.SyncReadAppPreference(to_fill.as_mut_ptr()) };
        }

        if matches!(ret_val, SyncManagerError::SYNCERR_NOT_FOUND) {
            Ok(None)
        } else {
            return_iff_conduit_err!(ret_val);
            let prefs = unsafe { to_fill.assume_init() };
            let prefs_size = prefs.m_actSize as usize;
            drop(prefs);
            bytes.truncate(prefs_size);
            let prefs = bytes.try_into().map_err(|_| {
                sync.log_to_hs_log(CString::new("Failure reading prefs from device\n").unwrap())
                    .unwrap();
                ConduitError::PreferenceSerde
            })?;
            Ok(Some(prefs))
        }
    }

    fn write_prefs(
        sync: &SyncSession,
        preferences: Preferences,
        creator: u32,
        pref_id: u16,
    ) -> Result<(), ConduitError> {
        let mut pref_bytes = preferences
            .try_into()
            .map_err(|_| ConduitError::PreferenceSerde)?;
        let prefs = CRawPreferenceInfo::new_with_buffer(1, creator, pref_id, &mut pref_bytes);
        info!("prefs to write {:?}", prefs);
        unsafe {
            return_iff_conduit_err!(sync
                .api
                .SyncWriteAppPreference(&prefs as *const CRawPreferenceInfo));
        }
        Ok(())
    }

    fn read_rec_by_index(
        index: u16,
        size_estimate: Option<u16>,
        handle: openDatabaseHandle,
        sync: &SyncSession,
    ) -> Result<(Vec<u8>, RecordAttributes, u32), ConduitError> {
        let size = size_estimate.unwrap_or(1024);
        let mut bytes = vec![0_u8; size as usize];

        let mut to_fill = MaybeUninit::new(CRawRecordInfo::new_for_reading_by_index(
            handle, index, &mut bytes,
        ));
        let mut ret_val = unsafe { sync.api.SyncReadRecordByIndex(to_fill.as_mut_ptr()) };

        // retry with the correct buffer size if too small
        if ret_val == SyncManagerError::SYNCERR_LOCAL_BUFF_TOO_SMALL {
            let new_size = unsafe { to_fill.assume_init_ref().get_size() } as usize;
            bytes.resize(new_size, 0_u8);
            to_fill = MaybeUninit::new(CRawRecordInfo::new_for_reading_by_index(
                handle, index, &mut bytes,
            ));
            ret_val = unsafe { sync.api.SyncReadRecordByIndex(to_fill.as_mut_ptr()) };
        }
        return_iff_conduit_err!(ret_val);

        let record = unsafe { to_fill.assume_init() };
        let attribs: RecordAttributes = record.get_attributes().into();
        let id = record.get_id();
        Ok((bytes, attribs, id))
    }

    fn get_db_rec_count(
        handle: openDatabaseHandle,
        sync: &SyncSession,
    ) -> Result<u32, ConduitError> {
        let mut ret = MaybeUninit::new(0);
        unsafe {
            return_iff_conduit_err!(sync.api.SyncGetDBRecordCount(handle, ret.as_mut_ptr()));
            Ok(ret.assume_init())
        }
    }

    fn remove_db(to_remove: CString, sync: &SyncSession) -> Result<(), ConduitError> {
        let ret = unsafe {
            sync.api
                .SyncDeleteDB(to_remove.as_bytes_with_nul().as_ptr(), 0)
        };
        let log_str;

        let ret = match ret {
            SyncManagerError::SYNCERR_NOT_FOUND => {
                log_str = format!(
                    "Database not found: {}\n",
                    String::from_utf8_lossy(to_remove.as_bytes())
                );
                Ok(())
            }
            SyncManagerError::SYNCERR_NONE => {
                log_str = format!(
                    "Database deleted: {}\n",
                    String::from_utf8_lossy(to_remove.as_bytes())
                );
                Ok(())
            }
            e @ _ => {
                log_str = format!(
                    "Error while deleting: {}\n",
                    String::from_utf8_lossy(to_remove.as_bytes())
                );
                Err(ConduitError::Sync(e))
            }
        };

        sync.log_to_hs_log(CString::new(log_str).unwrap())?;
        ret
    }

    fn open_db(to_open: CString, sync: &SyncSession) -> Result<openDatabaseHandle, ConduitError> {
        let mut handle = MaybeUninit::new(openDatabaseHandle::default());
        let m_name = {
            let mut name = [0; DB_NAMELEN];
            for (idx, char) in to_open.into_bytes().into_iter().enumerate() {
                if idx >= DB_NAMELEN - 1 {
                    break;
                }
                name[idx] = char;
            }
            name
        };
        unsafe {
            return_iff_conduit_err!(sync.api.SyncOpenDB(
                m_name.as_ptr(),
                0,
                handle.as_mut_ptr(),
                eDbOpenModes::eDbExclusive | eDbOpenModes::eDbRead | eDbOpenModes::eDbWrite
            ));
            Ok(handle.assume_init())
        }
    }

    fn create_db(
        to_create: CString,
        creator_id: u32,
        ty: u32,
        resource: bool,
        sync: &SyncSession,
    ) -> Result<openDatabaseHandle, ConduitError> {
        let stats: CDbCreateDB;
        let mut stats_init = MaybeUninit::new(CDbCreateDB::new(
            to_create.clone(),
            creator_id,
            ty,
            resource,
        ));
        unsafe {
            return_iff_conduit_err!(sync.api.SyncCreateDB(stats_init.as_mut_ptr()));
            stats = stats_init.assume_init()
        };
        let log_str = format!(
            "Created database: {}\n",
            String::from_utf8_lossy(to_create.as_bytes())
        );
        sync.log_to_hs_log(CString::new(log_str).unwrap())?;
        Ok(stats.handle())
    }

    fn drain_db(
        handle: openDatabaseHandle,
        sync: &SyncSession,
    ) -> Result<Vec<(Vec<u8>, RecordAttributes, u32)>, ConduitError> {
        let rec_count = Self::get_db_rec_count(handle, sync)?;
        let mut ret = Vec::with_capacity(rec_count as usize);

        for record_index in 0..rec_count {
            ret.push(Self::read_rec_by_index(
                record_index as u16,
                None,
                handle,
                sync,
            )?);
        }

        Ok(ret)
    }

    fn fill_db(
        handle: openDatabaseHandle,
        contents: PalmDatabase<PdbDatabase>,
        sync: &SyncSession,
    ) -> Result<(), ConduitError> {
        for (hdr, data) in contents.list_records_resources() {
            let mut data = data.clone();
            if let Some(mut attributes) = hdr.attributes() {
                // normal record
                let _category = attributes.category as i16;
                attributes.category = 0;
                let _flags = u8::from(attributes);
                unsafe {
                    let mut rec = CRawRecordInfo::new_for_writing(handle, 0, 0, None, &mut data);
                    return_iff_conduit_err!(sync.api.SyncWriteRec(&mut rec as *mut CRawRecordInfo));
                }
            } else {
                // yer a resource, harry
                sync.log_to_hs_log(CString::new("Writing resource for some reason??\n").unwrap())?;
                let rsc_ty = u32::from_be_bytes(
                    hdr.name_str()
                        .unwrap()
                        .as_bytes()
                        .to_vec()
                        .try_into()
                        .unwrap(),
                );
                let rsc_id = hdr.resource_id().unwrap();
                unsafe {
                    let rec = CRawRecordInfo::new_for_writing(
                        handle,
                        0,
                        0,
                        Some((rsc_ty, rsc_id)),
                        &mut data,
                    );
                    return_iff_conduit_err!(sync.api.SyncWriteResourceRec(rec));
                }
            }
        }
        Ok(())
    }

    fn close_db(handle: openDatabaseHandle, sync: &SyncSession) -> Result<(), ConduitError> {
        let log_str = format!("Closing database\n");
        sync.log_to_hs_log(CString::new(log_str).unwrap())?;
        return_iff_conduit_err!(unsafe { sync.api.SyncCloseDB(handle) });
        Ok(())
    }
}

impl<Preferences: TryInto<Vec<u8>> + TryFrom<Vec<u8>>> Conduit<Preferences> {
    /// Execute the conduit tasks defined with `ConduitBuilder`
    pub fn sync(self) -> Result<(), ConduitError> {
        let ss = SyncSession::init()?;
        let name = self.name.clone();

        let ret = match self.sync_internal(&ss) {
            Ok(_) => {
                let _ = ss.log_to_hs_log(CString::new("Sync completed!").unwrap());
                Ok(())
            }
            Err(e) => {
                let err_str = format!(
                    "Encountered error {} during sync of {}\n",
                    e,
                    String::from_utf8_lossy(name.as_bytes())
                );
                // attempt to log the error, nothing we can do if it fails and already erroring out anyway
                let _ = ss.log_to_hs_log(CString::new(err_str).unwrap());
                Err(e)
            }
        };
        ret.and(ss.shutdown())
    }

    fn sync_internal(self, ss: &SyncSession) -> Result<(), ConduitError> {
        ss.log_to_hs_log(
            CString::new(format!(
                "Beginning sync of {}\n",
                String::from_utf8_lossy(self.name.as_bytes())
            ))
            .unwrap(),
        )?;

        if let Some(pref) = self.preferences {
            ss.log_to_hs_log(CString::new("Syncing preferences").unwrap())?;
            match pref {
                PreferenceType::Static(id, pref) => {
                    Self::write_prefs(ss, pref, self.creator_id, id)?
                }
                PreferenceType::Dynamic(id, dyn_pref) => {
                    let current = Self::get_existing_prefs(ss, self.creator_id, id)?;
                    match (dyn_pref)(current) {
                        Some(new) => Self::write_prefs(ss, new, self.creator_id, id)?,
                        None => (),
                    }
                }
            }
            ss.log_to_hs_log(CString::new("Finished syncing preferences").unwrap())?;
        } else {
            info!("No prefs to sync");
        }

        for (to_drain, operation) in self.to_download {
            let Ok(handle) = Self::open_db(to_drain.clone(), &ss) else {
                continue;
            };
            let results = Self::drain_db(handle, ss)?;
            match operation {
                ConduitDBSink::Dynamic(mut op) => op(results)?,
            }
            Self::close_db(handle, &ss)?;
            Self::remove_db(to_drain, &ss)?;
        }

        for to_remove in self.to_remove {
            Self::remove_db(to_remove, &ss)?;
        }
        for to_remove in self.overwrite.iter().map(|(name, _, _)| name.clone()) {
            Self::remove_db(to_remove, &ss)?;
        }

        for (name, ty, db) in self.create_if_not_exists {
            let handle = Self::create_db(name, self.creator_id, ty, false, &ss)?;
            Self::fill_db(handle, db, &ss)?;
            Self::close_db(handle, &ss)?;
        }
        for (name, ty, db) in self.overwrite {
            let handle = Self::create_db(name, self.creator_id, ty, false, &ss)?;
            Self::fill_db(handle, db, &ss)?;
            Self::close_db(handle, &ss)?;
        }

        Ok(())
    }
}

struct SyncSession {
    // cond_mgr: ConduitManager,
    api: Container<SyncMgrApi>,
    open_cond: CONDHANDLE,
}

impl SyncSession {
    fn init() -> Result<Self, ConduitError> {
        let cond_mgr = ConduitManager::initialize()?;
        let sync_mgr_dll_path = dbg!(cond_mgr.get_sync_mgr_dll_path()?);
        let current_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(sync_mgr_dll_path.parent().unwrap()).unwrap();
        let api: Container<SyncMgrApi> = unsafe { Container::load(sync_mgr_dll_path) }?;
        std::env::set_current_dir(current_dir).unwrap();
        eprintln!("Loaded api");
        let mut open_cond_init = MaybeUninit::new(0);
        let open_cond;
        unsafe {
            return_iff_conduit_err!(api.SyncRegisterConduit(open_cond_init.as_mut_ptr()));
            open_cond = open_cond_init.assume_init();
        }
        Ok(Self { api, open_cond })
    }
    fn shutdown(self) -> Result<(), ConduitError> {
        unsafe {
            return_iff_conduit_err!(self.api.SyncUnRegisterConduit(self.open_cond));
        }
        Ok(())
    }
    fn log_to_hs_log(&self, line: CString) -> Result<(), ConduitError> {
        if let Ok(string) = line.clone().into_string() {
            log::info!("HS Log entry: {}", string);
        }
        return_iff_conduit_err!(unsafe {
            self.api.SyncAddLogEntry(line.as_bytes_with_nul().as_ptr())
        });
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_char_conv() {
        assert_eq!(u8::MAX as u32, uchars_to_u32([0, 0, 0, 255]));
        assert_eq!(u16::MAX as u32, uchars_to_u32([0, 0, 255, 255]));
        assert_eq!(u32::MAX, uchars_to_u32([255_u8; 4]));
    }

    #[test]
    #[ignore]
    fn test_load_api() {
        let cond_mgr = ConduitManager::initialize().unwrap();
        let sync_mgr_dll_path = dbg!(cond_mgr.get_sync_mgr_dll_path().unwrap());
        std::env::set_current_dir(sync_mgr_dll_path.parent().unwrap()).unwrap();
        let _: Container<SyncMgrApi> = unsafe { Container::load(sync_mgr_dll_path) }.unwrap();
    }
}
