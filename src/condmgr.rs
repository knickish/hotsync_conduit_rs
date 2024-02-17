use std::{
    ffi::{c_int, c_uchar, CString},
    io::Write,
    num::{NonZeroU32, NonZeroU8},
};

use dlopen2::wrapper::Container;

use crate::{
    condmgr_extern::CondMgrApi,
    error::{ConduitError, ConduitRegistrationError},
};

const COND_MGR_BIN: &[u8] = include_bytes!("../include/condmgr.dll");

#[derive(Debug)]
pub struct ConduitInstallation {
    creator: CString,
    creator_name: CString,
    creator_directory: Option<CString>,
    creator_file: Option<CString>,
    creator_remote: Option<CString>,
    creator_title: Option<CString>,
    creator_user: Option<CString>,
}

fn u32_to_nonzero_u8(u32: u32) -> Result<NonZeroU8, ConduitError> {
    if let Ok(c32) = NonZeroU32::try_from(u32) {
        if let Ok(c8) = NonZeroU8::try_from(c32) {
            return Ok(c8);
        }
    }
    Err(ConduitError::NonAsciiErr)
}

macro_rules! return_iff_conduit_err {
    ($expression:expr) => {
        let ret = $expression;
        if ret != ConduitRegistrationError::ERR_NONE {
            return Err(ConduitError::Registration(ret));
        }
    };
}

impl ConduitInstallation {
    /// Set the CreatorID
    pub fn new_with_creator(
        creator: [char; 4],
        conduit_name: CString,
    ) -> Result<Self, ConduitError> {
        let string: CString = creator
            .into_iter()
            .map(u32::from)
            .map(u32_to_nonzero_u8)
            .collect::<Result<Vec<NonZeroU8>, ConduitError>>()?
            .into();
        Ok(Self {
            creator: string,
            creator_name: conduit_name,
            creator_directory: None,
            creator_file: None,
            creator_remote: None,
            creator_title: None,
            creator_user: None,
        })
    }

    /// Set the directory used for storing files
    pub fn with_directory(mut self, creator_directory: impl Into<CString>) -> Self {
        self.creator_directory = Some(CString::new(creator_directory.into()).unwrap());
        self
    }

    /// The name of a file inside of the
    pub fn with_file(mut self, creator_file: impl Into<CString>) -> Self {
        self.creator_file = Some(creator_file.into());
        self
    }

    /// The name of the database that should be created on the handheld, if not present
    pub fn with_remote(mut self, creator_remote: impl Into<CString>) -> Self {
        self.creator_remote = Some(creator_remote.into());
        self
    }

    /// Set the diplay name for the conduit
    pub fn with_title(mut self, creator_title: impl Into<CString>) -> Self {
        self.creator_title = Some(creator_title.into());
        self
    }

    /// username for whom the conduit was installed
    pub fn with_user(mut self, creator_user: impl Into<CString>) -> Self {
        self.creator_user = Some(creator_user.into());
        self
    }
}

pub struct ConduitManager {
    api: Container<CondMgrApi>,
}

impl ConduitManager {
    const CONDUIT_APPLICATION: c_int = 1;

    pub fn initialize() -> Result<Self, ConduitError> {
        let condmgr_name = "Condmgr.dll";
        let loaded = if let Ok(wrapper) = unsafe { Container::load(condmgr_name) } {
            Self { api: wrapper }
        } else {
            let mut path = std::env::temp_dir();
            path.push(condmgr_name);
            let mut file = std::fs::File::create(&path)?;
            file.write_all(COND_MGR_BIN)?;
            file.sync_all()?;
            drop(file);
            Self {
                api: unsafe { Container::load(path) }?,
            }
        };
        log::info!("Condmgr.dll loaded");
        Ok(loaded)
    }

    pub(crate) fn get_sync_mgr_dll_path(&self) -> Result<std::path::PathBuf, ConduitError> {
        use std::path::PathBuf;
        type SizeType = c_int;
        const LEN: usize = 1000;
        let mut tmp = [0 as c_uchar; LEN];
        let size: SizeType;
        unsafe {
            let mut inner_size = std::mem::MaybeUninit::new(LEN as SizeType);
            self.api
                .CmGetHotSyncExecPath(tmp.as_mut_ptr(), inner_size.as_mut_ptr());
            size = inner_size.assume_init();
        }
        if size as usize > LEN {
            panic!("Unhandled size change");
        }
        let hs_path_c_string =
            CString::from_vec_with_nul((&tmp[..(size as usize)]).to_vec()).unwrap();
        let mut hs_path = PathBuf::from(hs_path_c_string.into_string().unwrap());
        hs_path.pop();
        hs_path.push("sync20.dll");
        Ok(hs_path)
    }

    /// Get the path to the folder in which the HotSync executable resides (and conduit dlls are stored)
    pub(crate) fn hotsync_folder_path(&self) -> Result<std::path::PathBuf, ConduitError> {
        use std::path::PathBuf;
        type SizeType = c_int;
        const LEN: usize = 1000;
        let mut tmp = [0 as c_uchar; LEN];
        let size: SizeType;
        unsafe {
            let mut inner_size = std::mem::MaybeUninit::new(LEN as SizeType);
            self.api
                .CmGetHotSyncExecPath(tmp.as_mut_ptr(), inner_size.as_mut_ptr());
            size = inner_size.assume_init();
        }
        if size as usize > LEN {
            panic!("Unhandled size change");
        }
        let hs_path_c_string =
            CString::from_vec_with_nul((&tmp[..(size as usize)]).to_vec()).unwrap();
        let mut hs_path = PathBuf::from(hs_path_c_string.into_string().unwrap());
        hs_path.pop();
        Ok(hs_path)
    }

    pub fn install(
        self,
        builder: ConduitInstallation,
        dll_bytes: Option<&[u8]>,
    ) -> Result<(), ConduitError> {
        let creator_id = builder.creator.as_bytes_with_nul().as_ptr();
        let conduit_name = builder.creator_name.as_bytes_with_nul().as_ptr();
        return_iff_conduit_err!(unsafe {
            self.api
                .CmInstallCreator(creator_id, Self::CONDUIT_APPLICATION)
        });
        return_iff_conduit_err!(unsafe { self.api.CmSetCreatorName(creator_id, conduit_name) });
        if let Some(creator_directory) = builder.creator_directory {
            return_iff_conduit_err!(unsafe {
                self.api.CmSetCreatorDirectory(
                    creator_id,
                    creator_directory.as_bytes_with_nul().as_ptr(),
                )
            });
        }
        if let Some(creator_file) = builder.creator_file {
            return_iff_conduit_err!(unsafe {
                self.api
                    .CmSetCreatorFile(creator_id, creator_file.as_bytes_with_nul().as_ptr())
            });
        }
        if let Some(creator_remote) = builder.creator_remote {
            return_iff_conduit_err!(unsafe {
                self.api
                    .CmSetCreatorRemote(creator_id, creator_remote.as_bytes_with_nul().as_ptr())
            });
        }
        if let Some(creator_title) = builder.creator_title {
            return_iff_conduit_err!(unsafe {
                self.api
                    .CmSetCreatorTitle(creator_id, creator_title.as_bytes_with_nul().as_ptr())
            });
        } else {
            // set the title to the creator ID if a title wasn't set
            return_iff_conduit_err!(unsafe {
                self.api.CmSetCreatorTitle(creator_id, conduit_name)
            });
        }
        if let Some(creator_user) = builder.creator_user {
            return_iff_conduit_err!(unsafe {
                self.api
                    .CmSetCreatorUser(creator_id, creator_user.as_bytes_with_nul().as_ptr())
            });
        }

        if let Some(dll_bytes) = dll_bytes {
            let name = builder
                .creator_name
                .into_string()
                .map_err(|_| ConduitError::NonAsciiErr)?;
            let mut base = self.hotsync_folder_path()?;
            base.push(name);
            std::fs::write(&base, dll_bytes)?;
        }

        log::info!("Conduit successfully installed");
        Ok(())
    }

    pub fn reinstall(
        self,
        builder: ConduitInstallation,
        dll_bytes: Option<&[u8]>,
    ) -> Result<(), ConduitError> {
        let res = unsafe {
            self.api
                .CmRemoveConduitByCreatorID(builder.creator.as_bytes_with_nul().as_ptr())
        };
        if (res as c_int) < 0 && !matches!(res, ConduitRegistrationError::ERR_NO_CONDUIT) {
            return Err(ConduitError::Registration(res));
        };
        self.install(builder, dll_bytes)
    }
}

#[cfg(test)]
mod test {
    use std::ffi::{c_int, c_uchar};

    use super::*;

    #[test]
    #[ignore]
    fn test_load_api() {
        ConduitManager::initialize().unwrap();
    }

    #[test]
    #[ignore]
    fn test_lib_version() {
        let mgr = ConduitManager::initialize().unwrap();
        unsafe {
            if dbg!(mgr.api.CmGetLibVersion()) <= 0 {
                panic!("Error getting version");
            };
        }
    }

    #[test]
    #[ignore]
    fn test_conduit_count() {
        let mgr = ConduitManager::initialize().unwrap();
        unsafe {
            if dbg!(mgr.api.CmGetConduitCount() as i16) < 0 {
                panic!("Error getting version");
            };
        }
    }

    #[test]
    #[ignore]
    fn test_core_path() {
        type SizeType = c_int;
        const LEN: usize = 100;
        let mut tmp = [0 as c_uchar; LEN];
        let mgr = ConduitManager::initialize().unwrap();
        let size: SizeType;
        unsafe {
            let mut inner_size = std::mem::MaybeUninit::new(LEN as SizeType);
            mgr.api
                .CmGetCorePath(tmp.as_mut_ptr(), dbg!(inner_size.as_mut_ptr()));
            size = inner_size.assume_init();
        }
        if dbg!(size as usize) > LEN {
            dbg!(size);
            panic!("Unhandled size change");
        }

        dbg!(CString::from_vec_with_nul((&tmp[..(size as usize)]).to_vec()).unwrap());
    }

    #[test]
    #[ignore]
    fn test_hs_path() {
        type SizeType = c_int;
        const LEN: usize = 100;
        let mut tmp = [0 as c_uchar; LEN];
        let mgr = ConduitManager::initialize().unwrap();
        let size: SizeType;
        unsafe {
            let mut inner_size = std::mem::MaybeUninit::new(LEN as SizeType);
            mgr.api
                .CmGetHotSyncExecPath(tmp.as_mut_ptr(), dbg!(inner_size.as_mut_ptr()));
            size = inner_size.assume_init();
        }
        if dbg!(size as usize) > LEN {
            dbg!(size);
            panic!("Unhandled size change");
        }

        dbg!(CString::from_vec_with_nul((&tmp[..(size as usize)]).to_vec()).unwrap());
    }
}
