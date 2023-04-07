use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};
use tarpc::client::RpcError;
use tarpc::tokio_serde::formats::Bincode;

// TODO: Use sifis-hazards
/// Hazard descriptions
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Hazard {
    /// The execution may cause fire.
    Fire,
    /// Information about energy consumption may be leaked.
    LogEnergyConsumption,
    /// The energy consumption may increase.
    EnergyConsumption,
    /// The execution may cause power outage.
    PowerOutage,
    /// Water can overflow and flood the building
    Flood,
    /// Might boil water or heat up a surface
    Scald,
}

impl Display for Hazard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

/// Lower level rpc
pub mod service {
    use crate::DoorLockStatus;

    use super::Hazard;

    #[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
    pub enum Error {
        #[error("Device of kind {found} found {req} requested")]
        Mismatch { found: String, req: String },
        #[error("Device {0} not found")]
        NotFound(String),
        #[error("Operation forbidden {risk}: {comment}")]
        Forbidden { risk: Hazard, comment: String },
    }

    #[tarpc::service]
    pub trait SifisApi {
        // Lamp-specific API
        async fn find_lamps() -> Result<Vec<String>, Error>;
        /// Turns a light on.
        ///
        /// # Hazards
        /// * [Hazard::Fire]
        /// * [Hazard::LogEnergyConsumption]
        /// * [Hazard::EnergyConsumption]
        async fn turn_lamp_on(id: String) -> Result<bool, Error>;
        /// Turns a light off.
        ///
        /// # Hazards
        /// * [Hazard::LogEnergyConsumption]
        async fn turn_lamp_off(id: String) -> Result<bool, Error>;
        /// Get the current on/off status for a light
        async fn get_lamp_on_off(id: String) -> Result<bool, Error>;
        /// Change the brightness.
        ///
        /// # Hazards
        /// * [Hazard::Fire]
        /// * [Hazard::LogEnergyConsumption]
        /// * [Hazard::EnergyConsumption]
        async fn set_lamp_brightness(id: String, brightness: u8) -> Result<u8, Error>;
        /// Get the current brightness level.
        async fn get_lamp_brightness(id: String) -> Result<u8, Error>;

        // Sink-specific API
        async fn find_sinks() -> Result<Vec<String>, Error>;
        /// Change the water flow.
        ///
        /// # Hazards
        /// * [Hazard::Flood]
        async fn set_sink_flow(id: String, flow: u8) -> Result<u8, Error>;
        /// Get the current water flow status
        async fn get_sink_flow(id: String) -> Result<u8, Error>;
        /// Set the sink the temperature
        ///
        /// # Hazard
        /// * [Hazard::Scald]
        async fn set_sink_temp(id: String, temp: u8) -> Result<u8, Error>;
        /// Get the current water temperature.
        async fn get_sink_temp(id: String) -> Result<u8, Error>;
        /// Close the drain
        ///
        /// let the water level in the sink rise.
        ///
        /// # Hazard
        /// * [Hazard::Flood]
        async fn close_sink_drain(id: String) -> Result<bool, Error>;
        /// Open the drain, emptying the sink.
        async fn open_sink_drain(id: String) -> Result<bool, Error>;
        /// Get the water level in the sink.
        async fn get_sink_level(id: String) -> Result<u8, Error>;

        // Door-specific API
        async fn find_doors() -> Result<Vec<String>, Error>;
        /// Get the lock status of a door.
        async fn get_door_lock_status(id: String) -> Result<DoorLockStatus, Error>;
        /// Get the open status of a door.
        async fn get_door_open(id: String) -> Result<bool, Error>;
        /// Lock a door.
        async fn lock_door(id: String) -> Result<bool, Error>;
        /// Unlock a door.
        async fn unlock_door(id: String) -> Result<bool, Error>;

        // Fridge-specific API
        async fn find_fridges() -> Result<Vec<String>, Error>;
        /// Get the current temperature of the fridge.
        async fn get_fridge_temperature(id: String) -> Result<i8, Error>;
        /// Get the target temperature of the fridge.
        async fn get_fridge_target_temperature(id: String) -> Result<i8, Error>;
        /// Set the target temperature of the fridge.
        async fn set_fridge_target_temperature(
            id: String,
            target_temperature: i8,
        ) -> Result<i8, Error>;
        /// Get the open status of the fridge.
        async fn get_fridge_open(id: String) -> Result<bool, Error>;
    }
}

use service::SifisApiClient;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DoorLockStatus {
    #[default]
    Unlocked,
    Locked,
    Jammed,
}

impl Display for DoorLockStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Unlocked => "unlocked",
            Self::Locked => "locked",
            Self::Jammed => "jammed",
        };
        f.write_str(s)
    }
}

/// Error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Runtime error")]
    Runtime(#[from] service::Error),
    #[error("RPC error")]
    Rpc(#[from] RpcError),
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("Device not found")]
    NotFound,
}

type Result<T> = std::result::Result<T, Error>;

/// Sifis client entry point
pub struct Sifis {
    client: SifisApiClient,
}

impl Sifis {
    /// Start the sifis client it will connect to the default unix socket
    pub async fn new() -> Result<Sifis> {
        let sifis_server =
            std::env::var("SIFIS_SERVER").unwrap_or("/var/run/sifis.sock".to_string());
        let transport =
            tarpc::serde_transport::unix::connect(sifis_server, Bincode::default).await?;
        let client = SifisApiClient::new(Default::default(), transport).spawn();

        Ok(Sifis { client })
    }

    /// Lookup for a Lamp with the specific id.
    pub async fn lamp(&self, lamp_id: &str) -> Result<Lamp> {
        self.client
            .find_lamps(tarpc::context::current())
            .await?
            .map(|lamps| {
                lamps.into_iter().find_map(|id| {
                    if lamp_id == id {
                        Some(Lamp {
                            client: &self.client,
                            id,
                        })
                    } else {
                        None
                    }
                })
            })?
            .ok_or_else(|| Error::NotFound)
    }

    /// Provide a list of the currently available Lamps.
    pub async fn lamps(&self) -> Result<Vec<Lamp>> {
        let r = self
            .client
            .find_lamps(tarpc::context::current())
            .await?
            .map(|lamps| {
                lamps
                    .into_iter()
                    .map(|id| Lamp {
                        client: &self.client,
                        id,
                    })
                    .collect()
            })?;
        Ok(r)
    }

    /// Lookup for a Sink with the specific id.
    pub async fn sink(&self, sink_id: &str) -> Result<Sink> {
        self.client
            .find_sinks(tarpc::context::current())
            .await?
            .map(|sinks| {
                sinks.into_iter().find_map(|id| {
                    if sink_id == id {
                        Some(Sink {
                            client: &self.client,
                            id,
                        })
                    } else {
                        None
                    }
                })
            })?
            .ok_or_else(|| Error::NotFound)
    }

    /// Provide a list of the currently available Sinks.
    pub async fn sinks(&self) -> Result<Vec<Sink>> {
        let r = self
            .client
            .find_sinks(tarpc::context::current())
            .await?
            .map(|sinks| {
                sinks
                    .into_iter()
                    .map(|id| Sink {
                        client: &self.client,
                        id,
                    })
                    .collect()
            })?;
        Ok(r)
    }

    /// Lookup for a Door with the specific id.
    pub async fn door(&self, door_id: &str) -> Result<Door> {
        self.client
            .find_doors(tarpc::context::current())
            .await?
            .map(|doors| {
                doors.into_iter().find_map(|id| {
                    if door_id == id {
                        Some(Door {
                            client: &self.client,
                            id,
                        })
                    } else {
                        None
                    }
                })
            })?
            .ok_or_else(|| Error::NotFound)
    }

    /// Provide a list of the currently available Doors.
    pub async fn doors(&self) -> Result<Vec<Door>> {
        let r = self
            .client
            .find_doors(tarpc::context::current())
            .await?
            .map(|doors| {
                doors
                    .into_iter()
                    .map(|id| Door {
                        client: &self.client,
                        id,
                    })
                    .collect()
            })?;
        Ok(r)
    }

    /// Lookup for a Fridge with the specific id.
    pub async fn fridge(&self, fridge_id: &str) -> Result<Fridge> {
        self.client
            .find_fridges(tarpc::context::current())
            .await?
            .map(|fridges| {
                fridges.into_iter().find_map(|id| {
                    if fridge_id == id {
                        Some(Fridge {
                            client: &self.client,
                            id,
                        })
                    } else {
                        None
                    }
                })
            })?
            .ok_or_else(|| Error::NotFound)
    }

    /// Provide a list of the currently available Fridges.
    pub async fn fridges(&self) -> Result<Vec<Fridge>> {
        let r = self
            .client
            .find_fridges(tarpc::context::current())
            .await?
            .map(|fridges| {
                fridges
                    .into_iter()
                    .map(|id| Fridge {
                        client: &self.client,
                        id,
                    })
                    .collect()
            })?;
        Ok(r)
    }
}

/// A connected Lamp
pub struct Lamp<'a> {
    client: &'a SifisApiClient,
    pub id: String,
}

impl Display for Lamp<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lamp - {}", self.id)
    }
}

impl<'a> Lamp<'a> {
    /// Turn on the lamp
    ///
    /// # Hazards
    /// * [Hazard::Fire]
    /// * [Hazard::LogEnergyConsumption]
    /// * [Hazard::EnergyConsumption]
    pub async fn turn_on(&self) -> Result<bool> {
        let r = self
            .client
            .turn_lamp_on(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }
    /// Turn off the lamp
    ///
    /// # Hazards
    /// * [Hazard::LogEnergyConsumption]
    pub async fn turn_off(&self) -> Result<bool> {
        let r = self
            .client
            .turn_lamp_off(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }
    /// Get the current on/off status for a light
    pub async fn get_on_off(&self) -> Result<bool> {
        let r = self
            .client
            .get_lamp_on_off(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }
    /// Get the current brightness level.
    pub async fn get_brightness(&self) -> Result<u8> {
        let r = self
            .client
            .get_lamp_brightness(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }
    /// Change the brightness.
    ///
    /// # Hazards
    /// * [Hazard::Fire]
    /// * [Hazard::LogEnergyConsumption]
    /// * [Hazard::EnergyConsumption]
    pub async fn set_brightness(&self, brightness: u8) -> Result<u8> {
        let r = self
            .client
            .set_lamp_brightness(tarpc::context::current(), self.id.clone(), brightness)
            .await??;
        Ok(r)
    }
}

/// Connected water basin/sink
pub struct Sink<'a> {
    client: &'a SifisApiClient,
    pub id: String,
}

impl Display for Sink<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sink - {}", self.id)
    }
}

impl<'a> Sink<'a> {
    /// Open the drain, emptying the sink.
    pub async fn open_drain(&self) -> Result<bool> {
        let r = self
            .client
            .open_sink_drain(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }
    /// Close the drain
    ///
    /// let the water level in the sink rise.
    ///
    /// # Hazard
    /// * [Hazard::Flood]
    pub async fn close_drain(&self) -> Result<bool> {
        let r = self
            .client
            .close_sink_drain(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }
    /// Get the water level in the sink.
    pub async fn get_water_level(&self) -> Result<u8> {
        let r = self
            .client
            .get_sink_level(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }
    /// Change the water flow.
    ///
    /// # Hazards
    /// * [Hazard::Flood]
    pub async fn set_flow(&self, brightness: u8) -> Result<u8> {
        let r = self
            .client
            .set_sink_flow(tarpc::context::current(), self.id.clone(), brightness)
            .await??;
        Ok(r)
    }
    /// Get the current water flow status
    pub async fn get_flow(&self) -> Result<u8> {
        let r = self
            .client
            .get_sink_flow(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }
    /// Set the sink the temperature
    ///
    /// # Hazard
    /// * [Hazard::Scald]
    pub async fn set_temperature(&self, brightness: u8) -> Result<u8> {
        let r = self
            .client
            .set_sink_temp(tarpc::context::current(), self.id.clone(), brightness)
            .await??;
        Ok(r)
    }
    /// Get the current water temperature.
    pub async fn get_temperature(&self) -> Result<u8> {
        let r = self
            .client
            .get_sink_temp(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }
}

/// Connected door
pub struct Door<'a> {
    client: &'a SifisApiClient,
    pub id: String,
}

impl Display for Door<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Door - {}", self.id)
    }
}

impl<'a> Door<'a> {
    /// Get the current open status.
    pub async fn is_open(&self) -> Result<bool> {
        let r = self
            .client
            .get_door_open(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }

    /// Get the current lock status.
    pub async fn lock_status(&self) -> Result<DoorLockStatus> {
        let r = self
            .client
            .get_door_lock_status(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }

    /// Try to lock the door.
    ///
    /// Returns false if the lock is jammed, true otherwise.
    pub async fn lock(&self) -> Result<bool> {
        let r = self
            .client
            .lock_door(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }

    /// Try to unlock the door.
    ///
    /// Returns false if the lock is jammed, true otherwise.
    pub async fn unlock(&self) -> Result<bool> {
        let r = self
            .client
            .unlock_door(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }
}

impl<'a> Fridge<'a> {
    /// Get the current open status.
    pub async fn is_open(&self) -> Result<bool> {
        let r = self
            .client
            .get_fridge_open(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }

    /// Get the current temperature.
    pub async fn temperature(&self) -> Result<i8> {
        let r = self
            .client
            .get_fridge_temperature(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }

    /// Get the target temperature.
    pub async fn target_temperature(&self) -> Result<i8> {
        let r = self
            .client
            .get_fridge_target_temperature(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }

    /// Set the target temperature.
    pub async fn set_target_temperature(&self, target_temperature: i8) -> Result<i8> {
        let r = self
            .client
            .set_fridge_target_temperature(
                tarpc::context::current(),
                self.id.clone(),
                target_temperature,
            )
            .await??;
        Ok(r)
    }
}

/// Connected fridge
pub struct Fridge<'a> {
    client: &'a SifisApiClient,
    pub id: String,
}

impl Display for Fridge<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fridge - {}", self.id)
    }
}
