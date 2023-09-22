use anyhow::Result;
use assert_cmd::prelude::*;
use sifis_api::{DoorLockStatus, Sifis};
use std::{path::PathBuf, process::Command, sync::OnceLock, time::Duration};
use tempfile::{tempdir, TempDir};

#[derive(Debug)]
struct Mock {
    sock: PathBuf,
    _dir: TempDir,
}

static SERVER: OnceLock<Result<Mock>> = OnceLock::new();

impl Mock {
    fn new() -> Result<Mock> {
        let dir: TempDir = tempdir()?;
        let sock: PathBuf = dir.path().join("sifis.sock");

        let _server = Command::cargo_bin("sifis-runtime-mock")?
            .env("SIFIS_SERVER", &sock)
            .spawn()?;

        // Wait for the server to get up
        std::thread::sleep(Duration::from_secs(1));

        Ok(Mock { sock, _dir: dir })
    }

    fn run() -> PathBuf {
        let mock = SERVER.get_or_init(Mock::new);

        mock.as_ref().map(|m| m.sock.to_owned()).unwrap()
    }

    async fn spawn() -> Result<Sifis> {
        let sock = Self::run();
        let sifis = Sifis::from_path(&sock).await?;

        Ok(sifis)
    }
}

#[tokio::test]
async fn lamp() -> Result<()> {
    let sifis = Mock::spawn().await?;

    let lamps = sifis.lamps().await?;

    for lamp in lamps {
        let on = lamp.get_on_off().await?;
        let brightness = lamp.get_brightness().await?;

        assert!(!on);
        assert_eq!(0, brightness);

        assert!(!lamp.turn_off().await?);
        assert!(lamp.turn_on().await?);
        assert_eq!(50, lamp.set_brightness(50).await?);
        assert_eq!(100, lamp.set_brightness(100).await?);
    }

    Ok(())
}

#[tokio::test]
async fn sink() -> Result<()> {
    let sifis = Mock::spawn().await?;

    let sinks = sifis.sinks().await?;

    for sink in sinks {
        let flow = sink.get_flow().await?;
        let level = sink.get_water_level().await?;
        let temp = sink.get_temperature().await?;

        assert_eq!(0, flow);
        assert_eq!(0, level);
        assert_eq!(20, temp);

        assert_eq!(0, sink.set_flow(0).await?);
        assert!(sink.open_drain().await?);
        assert!(!sink.close_drain().await?);
        assert_eq!(50, sink.set_flow(50).await?);
        assert_eq!(100, sink.set_temperature(100).await?);
    }

    Ok(())
}

#[tokio::test]
async fn door() -> Result<()> {
    let sifis = Mock::spawn().await?;

    let doors = sifis.doors().await?;

    for door in doors {
        let open = door.is_open().await?;
        let lock = door.lock_status().await?;

        assert!(!open);
        assert_eq!(DoorLockStatus::Unlocked, lock);

        assert!(door.unlock().await?);
        assert!(door.lock().await?);
    }

    Ok(())
}

#[tokio::test]
async fn fridge() -> Result<()> {
    let sifis = Mock::spawn().await?;

    let fridges = sifis.fridges().await?;

    for fridge in fridges {
        let open = fridge.is_open().await?;
        let temp = fridge.temperature().await?;
        let targ = fridge.target_temperature().await?;

        assert!(!open);
        assert_eq!(5, temp);
        assert_eq!(4, targ);

        assert_eq!(0, fridge.set_target_temperature(0).await?);
    }

    Ok(())
}
