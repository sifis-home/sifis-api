use std::os::{fd::RawFd, raw::c_int};

pub fn peer_pid(fd: RawFd) -> c_int {
    #[cfg(any(target_os = "android", target_os = "linux"))]
    {
        use nix::sys::socket::{getsockopt, sockopt::PeerCredentials};

        getsockopt(fd, PeerCredentials)
            .map(|creds| creds.pid() as _)
            .unwrap_or(-1)
    }

    #[cfg(any(target_os = "macos", target_os = "ios",))]
    {
        // TODO use sockopt::LocalPeerPid once the release of nix
        // supporting it is out
        let mut pid = std::mem::MaybeUninit::<i32>::uninit();
        let mut len = std::mem::size_of::<i32>() as i32;
        unsafe {
            let ret = libc::getsockopt(
                fd,
                libc::SOL_LOCAL,
                libc::LOCAL_PEERPID,
                pid.as_mut_ptr() as _,
                (&mut len) as *mut i32 as _,
            );

            if ret == 0 {
                pid.assume_init()
            } else {
                -1
            }
        }
    }
}
