//! Runtime utilities

use nix::sys::socket::{getsockopt, sockopt};
use std::os::{fd::BorrowedFd, raw::c_int};

/// Find the pid of the unix socket peer
pub fn peer_pid(fd: BorrowedFd) -> c_int {
    #[cfg(any(target_os = "android", target_os = "linux"))]
    {
        getsockopt(&fd, sockopt::PeerCredentials)
            .map(|creds| creds.pid() as _)
            .unwrap_or(-1)
    }

    #[cfg(any(target_os = "macos", target_os = "ios",))]
    {
        getsockopt(&fd, sockopt::LocalPeerPid).unwrap_or(-1)
    }
}
