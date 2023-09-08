#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused)]

use std::{
    ffi::{c_int, c_short, CString},
    marker::PhantomData,
};

use dlopen2::wrapper::WrapperApi;

use crate::error::SyncManagerError;

const SYNC_DB_NAMELEN: usize = 32;
const DB_NAMELEN: usize = 32;
const SYNC_REMOTE_USERNAME_BUF_SIZE: usize = 64;
const SYNC_REMOTE_PASSWORD_BUF_SIZE: usize = 64;

pub type CONDHANDLE = u32;
pub type openDatabaseHandle = u8;
type byteCardNo = u8;
type intCardNo = c_int;

#[repr(u32)]
pub enum eSyncTypes {
    eFast,
    eSlow,
    eHHtoPC,
    ePCtoHH,
    eInstall,
    eBackup,
    eDoNothing,
    eProfileInstall,
    eSyncTypeDoNotUse = 0xffffffff,
}
#[repr(u32)]
pub enum eFirstSync {
    eNeither,
    ePC,
    eHH,
    eFirstSyncDoNotUse = 0xffffffff,
}
#[repr(u32)]
pub enum eConnType {
    eCable,
    eModemConnType,
    eConnTypeDoNotUse = 0xffffffff,
}
#[repr(u32)]
pub enum eSyncPref {
    eNoPreference,
    ePermanentPreference,
    eTemporaryPreference,
    eSyncPrefDoNotUse = 0xffffffff,
}

bitflags::bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug)]
    pub struct eDbFlags: u32 {
        const eRecord               = 0x0000;
        const eResource             = 0x0001;
        const eReadOnly             = 0x0002;
        const eAppInfoDirty         = 0x0004;
        const eBackupDb             = 0x0008;
        const eOkToInstallNewer     = 0x0010;
        const eResetAfterInstall    = 0x0020;
        const eCopyPRevention       = 0x0040;
        const eOpenDb               = 0x8000;
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug)]
    pub struct eSyncRecAttrs: u32 {
        const eRecAttrDeleted	= 0x80;	// indicates that this record has been deleted on the handheld
        const eRecAttrDirty		= 0x40;	// indicates that this record was modified
        const eRecAttrBusy		= 0x20;	// SYSTEM USE ONLY: indicates that this record is currently in use
                                        // by some application on the remote, hand-held device.
                                        // CONDUITS: this attribute must be treated as read-only; do *not* pass
                                        // eRecAttrBusy when writing records.
        const eRecAttrSecret	= 0x10;	// "secret" record - password protected (also known as "private")
        const eRecAttrArchived	= 0x08;	// indicates that this record has been marked for archival
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug)]
    pub struct eDbOpenModes: u8 {
        const eDbShowSecret  = 0x0010;
        const eDbExclusive   = 0x0020;
        const eDbWrite       = 0x0040;
        const eDbRead        = 0x0080;
    }
}

#[repr(packed, C)]
pub struct CDbCreateDB {
    /// Upon return gets filled in by SyncMgr.Dll
    m_FileHandle: openDatabaseHandle,
    /// Supplied by caller, obtained from DbList
    m_Creator: u32,
    /// Supplied by caller, Res/Rec/RAM
    m_Flags: u32,
    /// Supplied by caller, target card #
    m_CardNo: byteCardNo,
    /// Supplied by caller, target DBase Name.
    /// must be null-terminated
    m_Name: [core::ffi::c_uchar; DB_NAMELEN],
    /// 4-char type of the new db
    /// for example 'DATA' or 'APPL'...
    m_Type: u32,

    m_Version: u16,
    m_dwReserved: u32,
}

impl CDbCreateDB {
    pub fn new(to_create: CString, creator_id: u32, ty: u32, resource: bool) -> Self {
        let m_Flags = if resource {
            eDbFlags::eResource.bits()
        } else {
            eDbFlags::eRecord.bits()
        };

        let m_Name = {
            let mut name = [0; DB_NAMELEN];
            for (idx, char) in to_create.into_bytes().into_iter().enumerate() {
                if idx >= DB_NAMELEN - 1 {
                    break;
                }
                name[idx] = char;
            }
            name
        };

        Self {
            m_FileHandle: 0,
            m_Creator: creator_id,
            m_Flags,
            m_CardNo: 0,
            m_Name,
            m_Type: ty,
            m_Version: 0,
            m_dwReserved: 0,
        }
    }

    pub fn handle(self) -> openDatabaseHandle {
        self.m_FileHandle
    }
}

#[repr(packed, C)]
pub struct CDbGenInfo {
    /// Name of remote database file
    m_FileName: [core::ffi::c_uchar; SYNC_DB_NAMELEN],
    /// When reading, the caller must fill this in
    /// with the size of the buffer pointed to by m_pBytes;
    /// When writing, the caller must set both this field
    /// and m_BytesRead to the size of the block being written.
    m_TotalBytes: i16,
    /// *This field is poorly named*
    /// When reading, it will
    /// be filled in with the actual size of
    /// the app or sort block, which
    /// may be bigger than the amount of data which is
    /// copied to m_pBytes in the event the block is bigger
    /// than the buffer (in this case, only the first m_TotalBytes
    /// of record data will be copied to caller's buffer by
    /// Sync API v2.1 or later, and *NOTHING* will
    /// be copied by Sync API before v2.1).
    /// When writing, the caller must set this field (in addition to
    /// m_TotalBytes) to the size of the block being written.
    m_BytesRead: i16,
    // pointer to caller's buffer
    m_pBytes: *mut u8,
    // Reserved - set to NULL
    m_dwReserved: i32,
}

///  Used by all the Record Oriented API's. Houses the DT_Link version
///  of a database's record layout, specifically that of the remote device.
///  Raw bytes will be formatted into this structure by the DTLinkConverter
///  object which resides inside of each Conduit.DLL.
#[repr(packed, C)]
pub struct CRawRecordInfo<'buffer> {
    /// Supplied by caller
    m_FileHandle: openDatabaseHandle,
    /// Supplied by caller when reading or deleting records by record id; supplied by
    /// caller as the resource type when deleting a resource; filled in
    /// by HH when reading (unique record id for records and resource type for resources).
    m_RecId: u32,
    /// Supplied by caller when reading records or resources by index; supplied by caller
    /// as the resource id when deleting a resource; filled in by handheld as the resource
    /// id when reading a resource; filled in by HH when reading a record using Sync API v2.1
    /// or later.
    m_RecIndex: u16,
    /// Filled in by HH when reading, and by caller when writing
    m_Attribs: u8,
    /// Filled in by HH when reading, and by caller when writing
    m_CatId: c_short,
    /// Ignore
    m_ConduitId: c_int,
    /// When reading, filled in by HH with the actual record/resource size,
    /// which might be bigger than buffer size m_TotalBytes (in this
    /// case, only the first m_TotalBytes of record data will be copied
    /// to caller's buffer by Sync API v2.1 or later, and NOTHING will
    /// be copied by Sync API before v2.1).  When writing, filled in by
    /// caller with record data size (same as m_TotalBytes).
    ///
    /// ****NOTE that m_TotalBytes is defined as WORD, meaning that only
    /// records and resources under 64K may be read or written using this
    /// API (the actual maximum is ~63.8K bytes).
    m_RecSize: u32,
    /// Supplied by caller: buffer size for reading; record data size for writing
    m_TotalBytes: u16,
    /// Buffer allocated by caller for reading or writing
    m_pBytes: *mut u8,
    /// Reserved	- set to NULL
    m_dwReserved: u32,
    /// zst to track lifetime of data buffer
    buffer_lifetime: PhantomData<&'buffer [u8]>,
}

impl<'buffer> CRawRecordInfo<'buffer> {
    pub unsafe fn new_for_writing(
        m_FileHandle: openDatabaseHandle,
        m_Attribs: u8,
        m_CatId: i16,
        resource_type_and_id: Option<(u32, u16)>,
        bytes: &mut Vec<u8>,
    ) -> Self {
        Self {
            m_FileHandle,
            m_RecId: resource_type_and_id.map(|(ty, _)| ty).unwrap_or(0),
            m_RecIndex: resource_type_and_id.map(|(_, id)| id).unwrap_or(0),
            m_Attribs,
            m_CatId,
            m_ConduitId: 0,
            m_RecSize: bytes.len() as u32,
            m_TotalBytes: bytes.len() as u16,
            m_pBytes: bytes.as_mut_ptr(),
            m_dwReserved: 0,
            buffer_lifetime: PhantomData,
        }
    }
}

#[repr(packed, C)]
pub struct CUserIDInfo {
    m_pName: [core::ffi::c_uchar; SYNC_REMOTE_USERNAME_BUF_SIZE],
    m_NameLength: i16,
    m_Password: [core::ffi::c_uchar; SYNC_REMOTE_PASSWORD_BUF_SIZE],
    m_PasswdLength: i16,
    /// Date/Time of last synchronization
    m_LastSyncDate: i32,
    m_LastSyncPC: u32,
    m_Id: u32,
    m_ViewerId: u32,
    /// Reserved - set to NULL
    m_dwReserved: u32,
}

///  A single element for a ReadDBList function call.
#[repr(packed, C)]
pub struct CDbList {
    m_CardNum: i16,
    /// contains Res/Record/Backup/ReadOnly (see enum eDbFlags)
    m_DbFlags: u16,
    m_DbType: u32,
    m_Name: [core::ffi::c_uchar; SYNC_DB_NAMELEN],
    m_Creator: u32,
    m_Version: u16,
    m_ModNumber: u32,
    /// not returned for SyncFindDbByName/TypeCreator and SyncReadOpenDbInfo
    m_Index: u16,
    m_CreateDate: i32,
    m_ModDate: i32,
    m_BackupDate: i32,
    /// miscellaneous db list flags (see eMiscDbListFlags)
    m_miscFlags: i32,
    /// Unused - Not filled in by SyncManager calls
    m_RecCount: i32,
    /// Unused - set to null
    m_dwReserved: i32,
}

/// Used to obtain remote system information.
#[repr(packed, C)]
pub struct CSystemInfo {
    m_RomSoftVersion: u32,                    // Upon return is filled in
    m_LocalId: u32,                           // Upon return is filled in
    m_ProdIdLength: u8,                       // Upon return is filled in (actual len)
    m_AllocedLen: u8, // Supplied by caller: size of buffer for ProductIdText
    m_ProductIdText: *mut core::ffi::c_uchar, // Allocated by caller: bufer for ProductIdText
    m_dwReserved: u32, // Reserved - set to NULL
}

impl CSystemInfo {
    /// Product ID buffer size in number of byts
    pub const SYNC_MAX_PROD_ID_SIZE: usize = 255;
}

///  A structure element for the SyncReadSingleCardInfo() function call.
#[repr(packed, C)]
pub struct CCardInfo {
    m_CardNo: u8,
    m_CardVersion: u16,
    m_CreateDate: i32,
    m_RomSize: u32,
    m_RamSize: u32,
    m_FreeRam: u32,
    m_CardNameLen: u8,
    m_ManufNameLen: u8,
    m_CardName: [core::ffi::c_uchar; Self::SYNC_REMOTE_CARDNAME_BUF_SIZE],
    m_ManufName: [core::ffi::c_uchar; Self::SYNC_REMOTE_MANUFNAME_BUF_SIZE],
    /// number of ROM-based databases
    m_romDbCount: u16,
    /// number of RAM-based databases
    m_ramDbCount: u16,
    /// Reserved - set to NULL
    m_dwReserved: u32,
}

impl CCardInfo {
    pub const SYNC_REMOTE_CARDNAME_BUF_SIZE: usize = 32;
    pub const SYNC_REMOTE_MANUFNAME_BUF_SIZE: usize = 32;
}

///  Used by the 'SyncCallApplication()' API
#[repr(packed, C)]
pub struct CCallAppParams {
    m_CreatorID: u32,
    m_ActionCode: u16,
    m_ResultCode: u16,
    m_ParamSize: u16,
    m_pParams: *mut u8,
}

///  Used by ReadPositionXMap
#[repr(packed, C)]
pub struct CPositionInfo {
    /// Open database handle
    m_FileHandle: openDatabaseHandle,
    /// offset of first position to read
    m_FirstPos: u16,
    /// total number of record Id's to read
    m_MaxEntries: u16,
    /// updated with number read in
    m_NumReadIn: u16,
    /// total length of 'pBytes'
    m_TotalBytes: u16,
    /// buffer to contain all record Id's
    m_pBytes: *mut u8,
}

#[rustfmt::skip]
#[derive(WrapperApi)]
pub struct SyncMgrApi {
    SyncAddLogEntry:            unsafe extern "C" fn(text: *const core::ffi::c_uchar) -> SyncManagerError,
    SyncRegisterConduit:        unsafe extern "C" fn(condhandle: *mut CONDHANDLE)-> SyncManagerError,
    SyncUnRegisterConduit:      unsafe extern "C" fn(condhandle: CONDHANDLE)-> SyncManagerError,
    SyncReadUserID:             unsafe extern "C" fn(user_info: *mut CUserIDInfo) -> SyncManagerError,
    SyncOpenDB:                 unsafe extern "C" fn(pName: *const core::ffi::c_uchar,nCardNum: intCardNo,rHandle: *mut openDatabaseHandle, openMode: eDbOpenModes) -> SyncManagerError,
    SyncDeleteDB:               unsafe extern "C" fn(pName: *const core::ffi::c_uchar, nCardNum: intCardNo) -> SyncManagerError,
    SyncCreateDB:               unsafe extern "C" fn(rDbStats: *mut CDbCreateDB) -> SyncManagerError,
    SyncCloseDB:                unsafe extern "C" fn(fHandle: openDatabaseHandle) -> SyncManagerError,
    SyncGetDBRecordCount:       unsafe extern "C" fn(fHandle: openDatabaseHandle, rCount: *mut u32) -> SyncManagerError,
    SyncPurgeDeletedRecs:       unsafe extern "C" fn(fHandle: openDatabaseHandle) -> SyncManagerError,
    SyncPurgeAllRecs:           unsafe extern "C" fn(fHandle: openDatabaseHandle) -> SyncManagerError,
    SyncResetSyncFlags:         unsafe extern "C" fn(fHandle: openDatabaseHandle) -> SyncManagerError,
    SyncReadDBList:             unsafe extern "C" fn(cardNo: byteCardNo, startIX: u16, bRam: bool, pList: *mut CDbList, rCnt: *mut i16) -> SyncManagerError,
    SyncWriteRec:               unsafe extern "C" fn(rRec: *mut CRawRecordInfo) -> SyncManagerError,
    SyncDeleteRec:              unsafe extern "C" fn(rRec: *const CRawRecordInfo) -> SyncManagerError,
    SyncDeleteResourceRec:      unsafe extern "C" fn(rRec: CRawRecordInfo) -> SyncManagerError,
    SyncDeleteAllResourceRec:   unsafe extern "C" fn(fHandle: openDatabaseHandle) -> SyncManagerError,
    SyncReadRecordById:         unsafe extern "C" fn(rRec: *mut CRawRecordInfo) -> SyncManagerError,
    SyncReadRecordByIndex:      unsafe extern "C" fn(rRec: *mut CRawRecordInfo) -> SyncManagerError,
    SyncReadResRecordByIndex:   unsafe extern "C" fn(rRec: *mut CRawRecordInfo, bBody: bool) -> SyncManagerError,
    SyncReadNextModifiedRec:    unsafe extern "C" fn(rRec: *mut CRawRecordInfo) -> SyncManagerError,
    SyncReadDBAppInfoBlock:     unsafe extern "C" fn(fHandle: openDatabaseHandle, rDbInfo: *mut CDbGenInfo) -> SyncManagerError,
    SyncWriteDBAppInfoBlock:    unsafe extern "C" fn(fHandle: openDatabaseHandle, rDbInfo: *const CDbGenInfo) -> SyncManagerError,
    SyncWriteResourceRec:       unsafe extern "C" fn(rRec: CRawRecordInfo) -> SyncManagerError,
    SyncRebootSystem:           unsafe extern "C" fn() -> SyncManagerError,
    SyncReadSystemInfo:         unsafe extern "C" fn(rInfo: *mut CSystemInfo) -> SyncManagerError,
    SyncReadSingleCardInfo:     unsafe extern "C" fn(rInfo: *mut CCardInfo) -> SyncManagerError,
    SyncReadSysDateTime:        unsafe extern "C" fn(rDate: *mut i32) -> SyncManagerError,
    SyncWriteSysDateTime:       unsafe extern "C" fn(lDate: i32) -> SyncManagerError,
    SyncReadDBSortInfoBlock:    unsafe extern "C" fn(fHandle: openDatabaseHandle, rDbInfo: *mut CDbGenInfo) -> SyncManagerError,
    SyncWriteDBSortInfoBlock:   unsafe extern "C" fn(fHandle: openDatabaseHandle, pDbInfo: *const CDbGenInfo) -> SyncManagerError,
    SyncCallApplication:        unsafe extern "C" fn(rOutParams: *mut CCallAppParams, rInParams: *mut CCallAppParams) -> SyncManagerError,
    SyncChangeCategory:         unsafe extern "C" fn(fHandle: openDatabaseHandle, from: u8, to: u8) -> SyncManagerError,
    SyncReadPositionXMap:       unsafe extern "C" fn(rInfo: *mut CPositionInfo) -> SyncManagerError,
    SyncYieldCycles:            unsafe extern "C" fn(wMaxMiliSecs: u16) -> SyncManagerError,
}
