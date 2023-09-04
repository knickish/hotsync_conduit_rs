use std::fmt::Display;

const SYNC_ERR_CLASS: i32 = 0x00004000;
const SYNC_FATAL_ERR_MASK: i32 = 0x10000000;
const SYNC_FATAL_ERR: i32 = SYNC_ERR_CLASS + SYNC_FATAL_ERR_MASK;
const TRANS_ERR_CLASS: i32 = 0x00002000;
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(i32)]
#[non_exhaustive]
#[allow(unused, non_camel_case_types)]
pub enum SyncManagerError {
    /// Success
    SYNCERR_NONE                 = 0x0000,	
    /// An unknown error occurred (local/remote error code mapping does not exist)
    SYNCERR_UNKNOWN               = (SYNC_ERR_CLASS + 0x01),	   
    /// NOT USED
    SYNCERR_MORE                  = (SYNC_ERR_CLASS + 0x02),	   
    /// Requested database, record, resource, etc. not found
    SYNCERR_NOT_FOUND				= (SYNC_ERR_CLASS + 0x03),     
    /// Attempt to open a database failed
    SYNCERR_FILE_NOT_OPEN         = (SYNC_ERR_CLASS + 0x04),	   
    /// NOT USED
    SYNCERR_FILE_OPEN             = (SYNC_ERR_CLASS + 0x05),	   
    /// The requested record is in use by someone else and will remain so indefinitely
    SYNCERR_RECORD_BUSY           = (SYNC_ERR_CLASS + 0x06),	   
    /// The requested record has either been deleted or archived
    SYNCERR_RECORD_DELETED        = (SYNC_ERR_CLASS + 0x07),	   
    /// Caller does not have write access or database is in ROM Defined for backward compatility
    SYNCERR_READ_ONLY             = (SYNC_ERR_CLASS + 0x09),	   
    /// Communications have not been intialized (this is an internal error code)
    SYNCERR_COMM_NOT_INIT         = (SYNC_ERR_CLASS + 0x0A),	   
    /// Could not create database because another one with the same name already exists on remote
    SYNCERR_FILE_ALREADY_EXIST    = (SYNC_ERR_CLASS + 0x0B),	   
    /// The requested database is presently open by someone else
    SYNCERR_FILE_ALREADY_OPEN     = (SYNC_ERR_CLASS + 0x0C),	   
    /// An operation was requested on a database when no databases were open
    SYNCERR_NO_FILES_OPEN         = (SYNC_ERR_CLASS + 0x0D),	   
    /// The requested operation is not supported on the given database type(record or resource).
    SYNCERR_BAD_OPERATION         = (SYNC_ERR_CLASS + 0x0E),	   
    /// Invalid argument passed to remote
    SYNCERR_REMOTE_BAD_ARG        = (SYNC_ERR_CLASS + 0x0F),	   
    /// Internal Desktop Link error -- indicates protocol implementation error
    SYNCERR_BAD_ARG_WRAPPER       = (SYNC_ERR_CLASS + 0x10),	   
    /// Internal Desktop Link error -- indicates protocol implementation error
    SYNCERR_ARG_MISSING           = (SYNC_ERR_CLASS + 0x11),	   
    /// The passed buffer is too small for the reply data
    SYNCERR_LOCAL_BUFF_TOO_SMALL  = (SYNC_ERR_CLASS + 0x12),	   
    /// Insufficient memory on remote to receive or complete the request
    SYNCERR_REMOTE_MEM            = (SYNC_ERR_CLASS + 0x13),	   
    /// Insufficient memory in remote data store to complete the request (write record, resource, etc.)
    SYNCERR_REMOTE_NO_SPACE       = (SYNC_ERR_CLASS + 0x14),	   
    /// Generic remote system error (returned when exact cause is unknown)
    SYNCERR_REMOTE_SYS            = (SYNC_ERR_CLASS + 0x15),	   
    /// Local (PC) memory allocation error
    SYNCERR_LOCAL_MEM             = (SYNC_ERR_CLASS + 0x16),	   
    /// Invalid parameter to local function, or parameter is too big
    SYNCERR_BAD_ARG			    = (SYNC_ERR_CLASS + 0x17),	   
    /// Data limit exceeded on remote (for example, when the hotsync error log size limit has been exceeded on remote)
    SYNCERR_LIMIT_EXCEEDED		= (SYNC_ERR_CLASS + 0x18),	   
    /// This request (command) is not supported by remote
    SYNCERR_UNKNOWN_REQUEST		= (SYNC_ERR_CLASS + 0x19),	   
    /// Request failed because there are too many open databases (for efficiency, the current Desktop Link implementation supports only one open database at a time)
    SYNCERR_TOO_MANY_OPEN_FILES   = (SYNC_FATAL_ERR + 0x403),     
    /// The request to cancel HotSync was initiated from the remote
    SYNCERR_REMOTE_CANCEL_SYNC    = (SYNC_FATAL_ERR + 0x405),     
    /// Connection is lost.  We add TRANS_ERR_CLASS because existing software checks that bit to detect connection loss.
    SYNCERR_LOST_CONNECTION		= (SYNC_FATAL_ERR + TRANS_ERR_CLASS + 0x410),
    /// The request to cancel HotSync was initiated from the PC
    SYNCERR_LOCAL_CANCEL_SYNC		= (SYNC_FATAL_ERR + 0x411)
}

impl Display for SyncManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncManagerError::SYNCERR_NONE => write!(f, "SYNCERR_NONE"),
            SyncManagerError::SYNCERR_UNKNOWN => write!(f, "SYNCERR_UNKNOWN"),
            SyncManagerError::SYNCERR_MORE => write!(f, "SYNCERR_MORE"),
            SyncManagerError::SYNCERR_NOT_FOUND => write!(f, "SYNCERR_NOT_FOUND"),
            SyncManagerError::SYNCERR_FILE_NOT_OPEN => write!(f, "SYNCERR_FILE_NOT_OPEN"),
            SyncManagerError::SYNCERR_FILE_OPEN => write!(f, "SYNCERR_FILE_OPEN"),
            SyncManagerError::SYNCERR_RECORD_BUSY => write!(f, "SYNCERR_RECORD_BUSY"),
            SyncManagerError::SYNCERR_RECORD_DELETED => write!(f, "SYNCERR_RECORD_DELETED"),
            SyncManagerError::SYNCERR_READ_ONLY => write!(f, "SYNCERR_READ_ONLY"),
            SyncManagerError::SYNCERR_COMM_NOT_INIT => write!(f, "SYNCERR_COMM_NOT_INIT"),
            SyncManagerError::SYNCERR_FILE_ALREADY_EXIST => write!(f, "SYNCERR_FILE_ALREADY_EXIST"),
            SyncManagerError::SYNCERR_FILE_ALREADY_OPEN => write!(f, "SYNCERR_FILE_ALREADY_OPEN"),
            SyncManagerError::SYNCERR_NO_FILES_OPEN => write!(f, "SYNCERR_NO_FILES_OPEN"),
            SyncManagerError::SYNCERR_BAD_OPERATION => write!(f, "SYNCERR_BAD_OPERATION"),
            SyncManagerError::SYNCERR_REMOTE_BAD_ARG => write!(f, "SYNCERR_REMOTE_BAD_ARG"),
            SyncManagerError::SYNCERR_BAD_ARG_WRAPPER => write!(f, "SYNCERR_BAD_ARG_WRAPPER"),
            SyncManagerError::SYNCERR_ARG_MISSING => write!(f, "SYNCERR_ARG_MISSING"),
            SyncManagerError::SYNCERR_LOCAL_BUFF_TOO_SMALL => write!(f, "SYNCERR_LOCAL_BUFF_TOO_SMALL"),
            SyncManagerError::SYNCERR_REMOTE_MEM => write!(f, "SYNCERR_REMOTE_MEM"),
            SyncManagerError::SYNCERR_REMOTE_NO_SPACE => write!(f, "SYNCERR_REMOTE_NO_SPACE"),
            SyncManagerError::SYNCERR_REMOTE_SYS => write!(f, "SYNCERR_REMOTE_SYS"),
            SyncManagerError::SYNCERR_LOCAL_MEM => write!(f, "SYNCERR_LOCAL_MEM"),
            SyncManagerError::SYNCERR_BAD_ARG => write!(f, "SYNCERR_BAD_ARG"),
            SyncManagerError::SYNCERR_LIMIT_EXCEEDED => write!(f, "SYNCERR_LIMIT_EXCEEDED"),
            SyncManagerError::SYNCERR_UNKNOWN_REQUEST => write!(f, "SYNCERR_UNKNOWN_REQUEST"),
            SyncManagerError::SYNCERR_TOO_MANY_OPEN_FILES => write!(f, "SYNCERR_TOO_MANY_OPEN_FILES"),
            SyncManagerError::SYNCERR_REMOTE_CANCEL_SYNC => write!(f, "SYNCERR_REMOTE_CANCEL_SYNC"),
            SyncManagerError::SYNCERR_LOST_CONNECTION => write!(f, "SYNCERR_LOST_CONNECTION"),
            SyncManagerError::SYNCERR_LOCAL_CANCEL_SYNC => write!(f, "SYNCERR_LOCAL_CANCEL_SYNC"),
        }
    }
}

const ERR_CONDUIT_MGR: i32 = -1000;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(i32)]
#[non_exhaustive]
#[allow(unused, non_camel_case_types)]
pub enum ConduitRegistrationError {
    ERR_NONE = (0),
    ERR_INDEX_OUT_OF_RANGE = (ERR_CONDUIT_MGR - 1),
    ERR_UNABLE_TO_DELETE = (ERR_CONDUIT_MGR - 2),
    ERR_NO_CONDUIT = (ERR_CONDUIT_MGR - 3),
    ERR_NO_MEMORY = (ERR_CONDUIT_MGR - 4),
    ERR_CREATORID_ALREADY_IN_USE = (ERR_CONDUIT_MGR - 5),
    ERR_REGISTRY_ACCESS = (ERR_CONDUIT_MGR - 6),
    ERR_UNABLE_TO_CREATE_CONDUIT = (ERR_CONDUIT_MGR - 7),
    ERR_UNABLE_TO_SET_CONDUIT_VALUE = (ERR_CONDUIT_MGR - 8),
    ERR_INVALID_HANDLE = (ERR_CONDUIT_MGR - 9),
    ERR_BUFFER_TOO_SMALL = (ERR_CONDUIT_MGR - 10),
    ERR_VALUE_NOT_FOUND = (ERR_CONDUIT_MGR - 11),
    ERR_INVALID_CREATOR_ID = (ERR_CONDUIT_MGR - 12),
    ERR_INVALID_POINTER = (ERR_CONDUIT_MGR - 13),
    ERR_UNABLE_TO_INSTALL_OLD = (ERR_CONDUIT_MGR - 14),
    ERR_INVALID_CONDUIT_TYPE = (ERR_CONDUIT_MGR - 15),
    ERR_INVALID_COM_PORT_TYPE = (ERR_CONDUIT_MGR - 16),
    ERR_NO_LONGER_SUPPORTED = (ERR_CONDUIT_MGR - 17),
}

#[allow(unreachable_patterns)]
impl Display for ConduitRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self as i32 == 0 {
            write!(f, "No errors detected")?;
            Ok(())
        } else {
            match self {
                ConduitRegistrationError::ERR_INDEX_OUT_OF_RANGE => {
                    write!(f, "ERR_INDEX_OUT_OF_RANGE")?
                }
                ConduitRegistrationError::ERR_UNABLE_TO_DELETE => {
                    write!(f, "ERR_UNABLE_TO_DELETE")?
                }
                ConduitRegistrationError::ERR_NO_CONDUIT => write!(f, "ERR_NO_CONDUIT")?,
                ConduitRegistrationError::ERR_NO_MEMORY => write!(f, "ERR_NO_MEMORY")?,
                ConduitRegistrationError::ERR_CREATORID_ALREADY_IN_USE => {
                    write!(f, "ERR_CREATORID_ALREADY_IN_USE")?
                }
                ConduitRegistrationError::ERR_REGISTRY_ACCESS => write!(f, "ERR_REGISTRY_ACCESS")?,
                ConduitRegistrationError::ERR_UNABLE_TO_CREATE_CONDUIT => {
                    write!(f, "ERR_UNABLE_TO_CREATE_CONDUIT")?
                }
                ConduitRegistrationError::ERR_UNABLE_TO_SET_CONDUIT_VALUE => {
                    write!(f, "ERR_UNABLE_TO_SET_CONDUIT_VALUE")?
                }
                ConduitRegistrationError::ERR_INVALID_HANDLE => write!(f, "ERR_INVALID_HANDLE")?,
                ConduitRegistrationError::ERR_BUFFER_TOO_SMALL => {
                    write!(f, "ERR_BUFFER_TOO_SMALL")?
                }
                ConduitRegistrationError::ERR_VALUE_NOT_FOUND => write!(f, "ERR_VALUE_NOT_FOUND")?,
                ConduitRegistrationError::ERR_INVALID_CREATOR_ID => {
                    write!(f, "ERR_INVALID_CREATOR_ID")?
                }
                ConduitRegistrationError::ERR_INVALID_POINTER => write!(f, "ERR_INVALID_POINTER")?,
                ConduitRegistrationError::ERR_UNABLE_TO_INSTALL_OLD => {
                    write!(f, "ERR_UNABLE_TO_INSTALL_OLD")?
                }
                ConduitRegistrationError::ERR_INVALID_CONDUIT_TYPE => {
                    write!(f, "ERR_INVALID_CONDUIT_TYPE")?
                }
                ConduitRegistrationError::ERR_INVALID_COM_PORT_TYPE => {
                    write!(f, "ERR_INVALID_COM_PORT_TYPE")?
                }
                ConduitRegistrationError::ERR_NO_LONGER_SUPPORTED => {
                    write!(f, "ERR_NO_LONGER_SUPPORTED")?
                }
                _ => write!(f, "UNKNOWN CONDUIT ERROR ENCOUNTERED: {}", *self as i32)?,
            }
            Ok(())
        }
    }
}

#[derive(Debug)]
pub enum ConduitError {
    NonAsciiErr,
    Registration(ConduitRegistrationError),
    Sync(SyncManagerError),
    DlOpen2(dlopen2::Error),
    Io(std::io::Error),
}

impl From<ConduitRegistrationError> for ConduitError {
    fn from(value: ConduitRegistrationError) -> Self {
        Self::Registration(value)
    }
}

impl From<SyncManagerError> for ConduitError {
    fn from(value: SyncManagerError) -> Self {
        Self::Sync(value)
    }
}

impl From<std::io::Error> for ConduitError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<dlopen2::Error> for ConduitError {
    fn from(value: dlopen2::Error) -> Self {
        Self::DlOpen2(value)
    }
}

impl Display for ConduitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Conduit Error Encountered")?;
        match self {
            ConduitError::NonAsciiErr => write!(f, "Non-ascii characters are not supported"),
            ConduitError::Registration(inner) => inner.fmt(f),
            ConduitError::Sync(inner) => inner.fmt(f),
            ConduitError::Io(inner) => inner.fmt(f),
            ConduitError::DlOpen2(inner) => inner.fmt(f),
        }
    }
}

impl std::error::Error for ConduitError {}
