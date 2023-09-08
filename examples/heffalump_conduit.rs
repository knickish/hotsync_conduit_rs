use std::ffi::{c_long, c_uchar, c_void, CString};

use hotsync_conduit_rs::{ConduitBuilder, ConduitDBSource};
use palmrs::database::{PalmDatabase, PdbDatabase};

const CREATOR: [c_uchar; 4] = [b'H', b'E', b'F', b'f'];
const AUTHOR_DB: &[u8] = include_bytes!("HeffalumpAuthorDB.pdb");
const CONTENT_DB: &[u8] = include_bytes!("HeffalumpContentDB.pdb");

#[no_mangle]
pub extern "cdecl" fn OpenConduit(_: *const c_void, _: *const c_void) -> c_long {
    let conduit =
        ConduitBuilder::new_with_name_creator(CString::new("heffalump_conduit").unwrap(), CREATOR)
            .overwrite_db(ConduitDBSource::Static(
                CString::new("HeffalumpAuthorDB").unwrap(),
                [b'A', b'u', b't', b'h'],
                PalmDatabase::<PdbDatabase>::from_bytes(&AUTHOR_DB).unwrap(),
            ))
            .overwrite_db(ConduitDBSource::Static(
                CString::new("HeffalumpContentDB").unwrap(),
                [b'T', b'o', b'o', b't'],
                PalmDatabase::<PdbDatabase>::from_bytes(&CONTENT_DB).unwrap(),
            ))
            .build();

    match conduit.sync() {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[cfg(test)]
mod test {
    use hotsync_conduit_rs::{ConduitInstallation, ConduitManager};
    use std::ffi::CString;

    #[test]
    #[ignore]
    fn install_test_conduit() {
        let creator = {
            let mut creator = [char::default(); 4];
            for (i, c) in crate::CREATOR.into_iter().enumerate() {
                creator[i] = char::from_u32(c as u32).unwrap();
            }
            creator
        };

        let builder = ConduitInstallation::new_with_creator(
            creator,
            CString::new("heffalump_conduit.dll").unwrap(),
        )
        .unwrap()
        .with_title(CString::new("Heffalump").unwrap());

        ConduitManager::initialize()
            .unwrap()
            .reinstall(builder)
            .unwrap();
    }
}
