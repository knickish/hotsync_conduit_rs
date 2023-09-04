
macro_rules! return_iff_conduit_err {
    ($expression:expr) => {
        let ret = $expression;
        if ret != SyncManagerError::SYNCERR_NONE {
            return Err(ConduitError::Sync(ret));
        }
    };
}