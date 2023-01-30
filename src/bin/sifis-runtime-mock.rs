//! Mock sifis runtime
//!
//! It simulates a number of devices

use futures::{future, prelude::*};
use std::collections::HashMap;
use std::sync::Arc;
use tarpc::context::Context;
use tarpc::server::{self, Channel};
use tarpc::tokio_serde::formats::Bincode;
use tokio::sync::Mutex;

use sifis_api::service::*;

#[derive(Clone, Debug)]
struct LampState {
    brightness: u8,
    on: bool,
}

#[derive(Clone, Debug)]
struct SinkState {
    flow: u8,
    temp: u8,
    level: u8,
    drain: bool,
}

#[derive(Clone, Debug)]
enum DeviceKind {
    Lamp(LampState),
    Sink(SinkState),
}

impl DeviceKind {
    pub fn display(&self) -> &str {
        match self {
            DeviceKind::Lamp(_) => "Lamp",
            DeviceKind::Sink(_) => "Sink",
        }
    }
}

#[derive(Clone, Debug)]
struct Device {
    name: String,
    kind: DeviceKind,
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
                DeviceKind::Lamp { .. } => Some(id.clone()),
                _ => None,
            })
            .collect();

        Ok(res)
    }

    // Lamp-specific API
    async fn turn_lamp_on(self, _: Context, id: String) -> Result<bool, Error> {
        self.apply_lamp(&id, |l| {
            l.on = true;
            Ok(true)
        })
        .await
    }
    async fn turn_lamp_off(self, _: Context, id: String) -> Result<bool, Error> {
        self.apply_lamp(&id, |l| {
            l.on = false;
            Ok(false)
        })
        .await
    }
    async fn set_lamp_brightness(
        self,
        _: Context,
        id: String,
        brightness: u8,
    ) -> Result<u8, Error> {
        self.apply_lamp(&id, |l: &mut LampState| {
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "/var/run/sifis.sock";
    let listener = tarpc::serde_transport::unix::listen(path, Bincode::default).await?;
    let devices = HashMap::new();
    // TODO populate

    let devices = Arc::new(Mutex::new(devices));

    listener
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        //        .max_channels_per_key(1, |t| t.transport().unwrap().peer_addr().as_pathname().unwrap())
        .map(|channel| {
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
