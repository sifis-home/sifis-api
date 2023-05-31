//! Mock sifis runtime
//!
//! It simulates a number of devices

use futures::{future, prelude::*};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tarpc::context::Context;
use tarpc::server::{self, Channel};
use tarpc::tokio_serde::formats::Bincode;
use tokio::fs::read_to_string;
use tokio::sync::Mutex;
use tracing::info;

use sifis_api::{service::*, DoorLockStatus};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
struct LampState {
    brightness: u8,
    on: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SinkState {
    flow: u8,
    temp: u8,
    level: u8,
    drain: bool,
}

impl Default for SinkState {
    #[inline]
    fn default() -> Self {
        Self {
            flow: 0,
            temp: 20,
            level: 0,
            drain: true,
        }
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
struct DoorState {
    is_open: bool,
    lock: DoorLockStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct FridgeState {
    open: bool,
    temperature: i8,
    target_temperature: i8,
}

impl Default for FridgeState {
    fn default() -> Self {
        Self {
            open: false,
            temperature: 5,
            target_temperature: 4,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum DeviceKind {
    Lamp(LampState),
    Sink(SinkState),
    Door(DoorState),
    Fridge(FridgeState),
}

impl DeviceKind {
    pub fn display(&self) -> &str {
        match self {
            DeviceKind::Lamp(_) => "Lamp",
            DeviceKind::Sink(_) => "Sink",
            DeviceKind::Door(_) => "Door",
            DeviceKind::Fridge(_) => "Fridge",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Device {
    name: String,
    kind: DeviceKind,
}

#[derive(Debug, Serialize, Deserialize)]
struct SifisConf {
    devices: HashMap<String, Device>,
}

#[derive(Clone, Debug)]
struct SifisMock {
    devices: Arc<Mutex<HashMap<String, Device>>>,
}

impl SifisMock {
    async fn apply<F, R>(&self, id: &str, f: F) -> Result<R, Error>
    where
        F: FnOnce(&mut Device) -> Result<R, Error>,
    {
        let mut devs = self.devices.lock().await;

        let d = devs
            .get_mut(id)
            .ok_or_else(|| Error::NotFound(id.to_owned()))?;

        f(d)
    }
    async fn apply_lamp<F, R>(&self, id: &str, f: F) -> Result<R, Error>
    where
        F: FnOnce(&mut LampState) -> Result<R, Error>,
    {
        self.apply(id, |d| match d.kind {
            DeviceKind::Lamp(ref mut lamp) => f(lamp),
            _ => Err(Error::Mismatch {
                found: d.kind.display().to_string(),
                req: "Lamp".to_string(),
            }),
        })
        .await
    }
    async fn apply_sink<F, R>(&self, id: &str, f: F) -> Result<R, Error>
    where
        F: FnOnce(&mut SinkState) -> Result<R, Error>,
    {
        self.apply(id, |d| match d.kind {
            DeviceKind::Sink(ref mut sink) => f(sink),
            _ => Err(Error::Mismatch {
                found: d.kind.display().to_string(),
                req: "Sink".to_string(),
            }),
        })
        .await
    }
    async fn apply_door<F, R>(&self, id: &str, f: F) -> Result<R, Error>
    where
        F: FnOnce(&mut DoorState) -> Result<R, Error>,
    {
        self.apply(id, |d| match d.kind {
            DeviceKind::Door(ref mut door) => f(door),
            _ => Err(Error::Mismatch {
                found: d.kind.display().to_string(),
                req: "Door".to_string(),
            }),
        })
        .await
    }
    async fn apply_fridge<F, R>(&self, id: &str, f: F) -> Result<R, Error>
    where
        F: FnOnce(&mut FridgeState) -> Result<R, Error>,
    {
        self.apply(id, |d| match d.kind {
            DeviceKind::Fridge(ref mut fridge) => f(fridge),
            _ => Err(Error::Mismatch {
                found: d.kind.display().to_string(),
                req: "Fridge".to_string(),
            }),
        })
        .await
    }
}

#[tarpc::server]
impl SifisApi for SifisMock {
    async fn find_lamps(self, _: Context) -> Result<Vec<String>, Error> {
        let res = self
            .devices
            .lock()
            .await
            .iter()
            .filter_map(|(id, dev)| match dev.kind {
                DeviceKind::Lamp { .. } => Some(id.clone()),
                _ => None,
            })
            .collect();

        Ok(res)
    }

    async fn find_sinks(self, _: Context) -> Result<Vec<String>, Error> {
        let res = self
            .devices
            .lock()
            .await
            .iter()
            .filter_map(|(id, dev)| match dev.kind {
                DeviceKind::Sink { .. } => Some(id.clone()),
                _ => None,
            })
            .collect();

        Ok(res)
    }

    // Lamp-specific API
    async fn turn_lamp_on(self, _: Context, id: String) -> Result<bool, Error> {
        self.apply_lamp(&id, |l| {
            tracing::info!("Setting lamp {id} on property to true from {}", l.on);
            l.on = true;
            Ok(true)
        })
        .await
    }
    async fn turn_lamp_off(self, _: Context, id: String) -> Result<bool, Error> {
        self.apply_lamp(&id, |l| {
            tracing::info!("Setting lamp {id} on property to false from {}", l.on);
            l.on = false;
            Ok(false)
        })
        .await
    }
    async fn get_lamp_on_off(self, _: Context, id: String) -> Result<bool, Error> {
        self.apply_lamp(&id, |l| Ok(l.on)).await
    }
    async fn set_lamp_brightness(
        self,
        _: Context,
        id: String,
        brightness: u8,
    ) -> Result<u8, Error> {
        self.apply_lamp(&id, |l: &mut LampState| {
            tracing::info!(
                "Setting lamp {id} brightness to {brightness} from {}",
                l.brightness,
            );
            l.brightness = brightness;
            Ok(brightness)
        })
        .await
    }
    async fn get_lamp_brightness(self, _: Context, id: String) -> Result<u8, Error> {
        self.apply_lamp(&id, |l: &mut LampState| Ok(l.brightness))
            .await
    }

    // Sink-specific API
    async fn set_sink_flow(self, _: Context, id: String, flow: u8) -> Result<u8, Error> {
        self.apply_sink(&id, |s: &mut SinkState| {
            s.flow = flow;
            Ok(flow)
        })
        .await
    }
    async fn get_sink_flow(self, _: Context, id: String) -> Result<u8, Error> {
        self.apply_sink(&id, |s: &mut SinkState| Ok(s.flow)).await
    }
    async fn set_sink_temp(self, _: Context, id: String, temp: u8) -> Result<u8, Error> {
        self.apply_sink(&id, |s: &mut SinkState| {
            s.temp = temp;
            Ok(temp)
        })
        .await
    }
    async fn get_sink_temp(self, _: Context, id: String) -> Result<u8, Error> {
        self.apply_sink(&id, |s: &mut SinkState| Ok(s.temp)).await
    }
    async fn close_sink_drain(self, _: Context, id: String) -> Result<bool, Error> {
        self.apply_sink(&id, |s: &mut SinkState| {
            s.drain = false;
            Ok(false)
        })
        .await
    }
    async fn open_sink_drain(self, _: Context, id: String) -> Result<bool, Error> {
        self.apply_sink(&id, |s: &mut SinkState| {
            s.drain = true;
            Ok(true)
        })
        .await
    }
    async fn get_sink_level(self, _: Context, id: String) -> Result<u8, Error> {
        self.apply_sink(&id, |s: &mut SinkState| Ok(s.level)).await
    }

    async fn find_doors(self, _: Context) -> Result<Vec<String>, Error> {
        let res = self
            .devices
            .lock()
            .await
            .iter()
            .filter_map(|(id, dev)| match dev.kind {
                DeviceKind::Door { .. } => Some(id.clone()),
                _ => None,
            })
            .collect();

        Ok(res)
    }

    async fn get_door_lock_status(self, _: Context, id: String) -> Result<DoorLockStatus, Error> {
        self.apply_door(&id, |s: &mut DoorState| Ok(s.lock)).await
    }

    async fn get_door_open(self, _: Context, id: String) -> Result<bool, Error> {
        self.apply_door(&id, |s: &mut DoorState| Ok(s.is_open))
            .await
    }

    async fn lock_door(self, _: Context, id: String) -> Result<bool, Error> {
        self.apply_door(&id, |s: &mut DoorState| {
            Ok(match s.lock {
                DoorLockStatus::Locked => true,
                DoorLockStatus::Unlocked => {
                    s.lock = DoorLockStatus::Locked;
                    true
                }
                DoorLockStatus::Jammed => false,
            })
        })
        .await
    }

    async fn unlock_door(self, _: Context, id: String) -> Result<bool, Error> {
        self.apply_door(&id, |s: &mut DoorState| {
            Ok(match s.lock {
                DoorLockStatus::Unlocked => true,
                DoorLockStatus::Locked => {
                    s.lock = DoorLockStatus::Unlocked;
                    true
                }
                DoorLockStatus::Jammed => false,
            })
        })
        .await
    }

    async fn find_fridges(self, _: Context) -> Result<Vec<String>, Error> {
        let res = self
            .devices
            .lock()
            .await
            .iter()
            .filter_map(|(id, dev)| match dev.kind {
                DeviceKind::Fridge { .. } => Some(id.clone()),
                _ => None,
            })
            .collect();

        Ok(res)
    }

    async fn get_fridge_temperature(self, _: Context, id: String) -> Result<i8, Error> {
        self.apply_fridge(&id, |s: &mut FridgeState| Ok(s.temperature))
            .await
    }

    async fn get_fridge_target_temperature(self, _: Context, id: String) -> Result<i8, Error> {
        self.apply_fridge(&id, |s: &mut FridgeState| Ok(s.target_temperature))
            .await
    }

    async fn set_fridge_target_temperature(
        self,
        _: Context,
        id: String,
        target_temperature: i8,
    ) -> Result<i8, Error> {
        self.apply_fridge(&id, |s: &mut FridgeState| {
            s.target_temperature = target_temperature;
            Ok(target_temperature)
        })
        .await
    }

    async fn get_fridge_open(self, _: Context, id: String) -> Result<bool, Error> {
        self.apply_fridge(&id, |s: &mut FridgeState| Ok(s.open))
            .await
    }
}

async fn load_conf() -> SifisConf {
    if let Ok(conf_s) = read_to_string("sifis-runtime.toml").await {
        toml::from_str(&conf_s).expect("Failed to load configuration")
    } else {
        tracing::warn!("Cannot find a configuration file, using the default");
        let mut devices = HashMap::new();
        devices.insert(
            "lamp1".to_owned(),
            Device {
                name: "Safe lamp".to_owned(),
                kind: DeviceKind::Lamp(LampState::default()),
            },
        );
        devices.insert(
            "lamp2".to_owned(),
            Device {
                name: "Unsafe lamp".to_owned(),
                kind: DeviceKind::Lamp(LampState::default()),
            },
        );
        devices.insert(
            "sink 1".to_owned(),
            Device {
                name: "Kitchen Sink".to_owned(),
                kind: DeviceKind::Sink(SinkState::default()),
            },
        );
        devices.insert(
            "door 1".to_owned(),
            Device {
                name: "Bedroom Door".to_owned(),
                kind: DeviceKind::Door(DoorState::default()),
            },
        );
        devices.insert(
            "fridge 1".to_owned(),
            Device {
                name: "Kitchen Fridge".to_owned(),
                kind: DeviceKind::Fridge(FridgeState::default()),
            },
        );

        SifisConf { devices }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let path = std::env::var("SIFIS_SERVER").unwrap_or("/var/run/sifis.sock".to_string());
    if Path::new(&path).exists() {
        std::fs::remove_file(&path)
            .unwrap_or_else(|err| panic!("unable to remove old unix socket file: {err}"));
    }
    let listener = tarpc::serde_transport::unix::listen(path, Bincode::default).await?;

    let conf = load_conf().await;
    let devices = Arc::new(Mutex::new(conf.devices));

    listener
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        //        .max_channels_per_key(1, |t| t.transport().unwrap().peer_addr().as_pathname().unwrap())
        .map(|channel| {
            let peer = channel.transport().get_ref();

            info!("New client {peer:?}");
            let server = SifisMock {
                devices: devices.clone(),
            };
            channel.execute(server.serve())
        })
        // Max concurrent calls
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;

    Ok(())
}
