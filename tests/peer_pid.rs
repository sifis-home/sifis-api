use nix::{
    sys::wait::waitpid,
    unistd::{fork, ForkResult},
};
use std::{
    io::Read,
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
};
use std::{io::Write, os::unix::io::AsRawFd};

#[test]
fn peer_pid() {
    let path = Path::new(concat!(env!("CARGO_TARGET_TMPDIR"), "/test.socket"));
    if path.exists() {
        std::fs::remove_file(&path).unwrap();
    }
    let u = UnixListener::bind(&path).unwrap();

    match unsafe { fork().unwrap() } {
        ForkResult::Child => {
            eprintln!("Child connecting");
            let mut ux = UnixStream::connect(&path).unwrap();
            let mut s = String::new();

            eprintln!("Child connected");

            let _ = ux.read_to_string(&mut s);

            eprintln!("{s}");
        }
        ForkResult::Parent { child, .. } => {
            eprintln!("Waiting for client");
            let mut s = u.incoming().next().unwrap().unwrap();

            let fd = s.as_raw_fd();

            eprintln!("Connected");

            let pid = sifis_api::runtime::peer_pid(fd);

            assert_eq!(pid, child.as_raw());

            s.write_all("Done".as_bytes()).unwrap();

            s.shutdown(std::net::Shutdown::Both).unwrap();

            waitpid(child, None).unwrap();
        }
    }
}
