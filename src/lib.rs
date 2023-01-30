use tarpc::client::RpcError;
use tarpc::tokio_serde::formats::Bincode;

pub mod service {
    #[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
    pub enum Error {
        #[error("Device of kind {found} found {req} requested")]
        Mismatch { found: String, req: String },
        #[error("Device {0} not found")]
        NotFound(String),
        #[error("Operation forbidden")]
        Forbidden(String),
    }

    #[tarpc::service]
    pub trait SifisApi {
        // Lamp-specific API
        async fn find_lamps() -> Result<Vec<String>, Error>;
        async fn turn_lamp_on(id: String) -> Result<bool, Error>;
        async fn turn_lamp_off(id: String) -> Result<bool, Error>;
        async fn set_lamp_brightness(id: String, brightness: u8) -> Result<u8, Error>;
        async fn get_lamp_brightness(id: String) -> Result<u8, Error>;

        // Sink-specific API
        async fn find_sinks() -> Result<Vec<String>, Error>;
        async fn set_sink_flow(id: String, flow: u8) -> Result<u8, Error>;
        async fn get_sink_flow(id: String) -> Result<u8, Error>;
        async fn set_sink_temp(id: String, temp: u8) -> Result<u8, Error>;
        async fn get_sink_temp(id: String) -> Result<u8, Error>;
        async fn close_sink_drain(id: String) -> Result<bool, Error>;
        async fn open_sink_drain(id: String) -> Result<bool, Error>;
        async fn get_sink_level(id: String) -> Result<u8, Error>;
    }
}

use service::SifisApiClient;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Runtime error")]
    Runtime(#[from] service::Error),
    #[error("RPC error")]
    Rpc(#[from] RpcError),
    #[error("I/O error")]
    Io(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

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
}

pub struct Lamp<'a> {
    client: &'a SifisApiClient,
    id: String,
}

impl<'a> Lamp<'a> {
    pub async fn turn_on(&self) -> Result<bool> {
        let r = self
            .client
            .turn_lamp_on(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }
    pub async fn turn_off(&self) -> Result<bool> {
        let r = self
            .client
            .turn_lamp_off(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }
    pub async fn get_brightness(&self) -> Result<u8> {
        let r = self
            .client
            .get_lamp_brightness(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }
    pub async fn set_brightness(&self, brightness: u8) -> Result<u8> {
        let r = self
            .client
            .set_lamp_brightness(tarpc::context::current(), self.id.clone(), brightness)
            .await??;
        Ok(r)
    }
}

pub struct Sink<'a> {
    client: &'a SifisApiClient,
    id: String,
}

impl<'a> Sink<'a> {
    pub async fn open_drain(&self) -> Result<bool> {
        let r = self
            .client
            .open_sink_drain(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }
    pub async fn close_drain(&self) -> Result<bool> {
        let r = self
            .client
            .close_sink_drain(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }
    pub async fn get_water_level(&self) -> Result<u8> {
        let r = self
            .client
            .get_sink_level(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }
    pub async fn set_flow(&self, brightness: u8) -> Result<u8> {
        let r = self
            .client
            .set_sink_flow(tarpc::context::current(), self.id.clone(), brightness)
            .await??;
        Ok(r)
    }
    pub async fn get_flow(&self) -> Result<u8> {
        let r = self
            .client
            .get_sink_flow(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }
    pub async fn set_temperature(&self, brightness: u8) -> Result<u8> {
        let r = self
            .client
            .set_sink_temp(tarpc::context::current(), self.id.clone(), brightness)
            .await??;
        Ok(r)
    }
    pub async fn get_temperature(&self) -> Result<u8> {
        let r = self
            .client
            .get_sink_temp(tarpc::context::current(), self.id.clone())
            .await??;
        Ok(r)
    }
}
