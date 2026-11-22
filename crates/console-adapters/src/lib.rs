//! Adapters for services a console may expose *if the user has enabled them*. Nothing here is
//! assumed present; callers verify by handshake (see `ftp::probe`) before capabilities turn on.

pub mod ftp;
pub mod ops;
pub mod paths;
