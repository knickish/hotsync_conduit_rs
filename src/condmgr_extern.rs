#![allow(unused, non_snake_case)]
use std::ffi::{c_int, c_uchar, c_uint};

use dlopen2::wrapper::WrapperApi;

use crate::error::ConduitRegistrationError;

#[rustfmt::skip]
#[derive(WrapperApi)]
pub struct CondMgrApi {
    CmInstallCreator:           unsafe extern "system" fn(pCreatorID: *const c_uchar, iType: c_int) -> ConduitRegistrationError,
    CmSetCreatorName:           unsafe extern "system" fn(pCreatorID: *const c_uchar, pConduitName: *const c_uchar) -> ConduitRegistrationError,
    CmSetCreatorDirectory:      unsafe extern "system" fn(pCreatorID: *const c_uchar, pDirectory: *const c_uchar) -> ConduitRegistrationError,
    CmSetCreatorFile:           unsafe extern "system" fn(pCreatorID: *const c_uchar, pFile: *const c_uchar) -> ConduitRegistrationError,
    CmSetCreatorRemote:         unsafe extern "system" fn(pCreatorID: *const c_uchar, pRemoteDB: *const c_uchar) -> ConduitRegistrationError,
    CmSetCreatorTitle:          unsafe extern "system" fn(pCreatorID: *const c_uchar, pTitle: *const c_uchar) -> ConduitRegistrationError,
    CmSetCreatorUser:           unsafe extern "system" fn(pCreatorID: *const c_uchar, pUsername: *const c_uchar) -> ConduitRegistrationError,
    CmSetCreatorPriority:       unsafe extern "system" fn(pCreatorID: *const c_uchar, dwPriority: u32) -> ConduitRegistrationError,
    CmSetCreatorInfo:           unsafe extern "system" fn(pCreatorID: *const c_uchar, pInfo: *const c_uchar) -> ConduitRegistrationError,
    CmRemoveConduitByCreatorID: unsafe extern "system" fn(pCreatorID: *const c_uchar) -> ConduitRegistrationError,
    CmGetCorePath:              unsafe extern "system" fn(pPath: *mut c_uchar, bufSize: *mut c_int) -> ConduitRegistrationError,
    CmGetConduitCount:          unsafe extern "system" fn() -> ConduitRegistrationError,
    CmGetLibVersion:            unsafe extern "system" fn() -> u16,
}
