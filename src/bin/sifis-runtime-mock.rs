/// Mock sifis runtime
///
/// It simulates a number of devices
use tarpc::context::Context;

use sifis_api::service::*;

#[derive(Clone, Debug)]
struct SifisDht {}

#[tarpc::server]
impl SifisApi for SifisDht {
    async fn find_lamps(self, _: Context) -> Result<Vec<String>, Error> {
        todo!()
    }

    async fn find_sinks(self, _: Context) -> Result<Vec<String>, Error> {
        todo!()
    }

    // Lamp-specific API
    async fn turn_lamp_on(self, _: Context, id: String) -> Result<bool, Error> {
        todo!()
    }
    async fn turn_lamp_off(self, _: Context, id: String) -> Result<bool, Error> {
        todo!()
    }
    async fn set_lamp_brightness(
        self,
        _: Context,
        id: String,
        brightness: u8,
    ) -> Result<u8, Error> {
        todo!()
    }
    async fn get_lamp_brightness(self, _: Context, id: String) -> Result<u8, Error> {
        todo!()
    }

    // Sink-specific API
    async fn set_sink_flow(self, _: Context, id: String, flow: u8) -> Result<u8, Error> {
        todo!()
    }
    async fn get_sink_flow(self, _: Context, id: String) -> Result<u8, Error> {
        todo!()
    }
    async fn set_sink_temp(self, _: Context, id: String, temp: u8) -> Result<u8, Error> {
        todo!()
    }
    async fn get_sink_temp(self, _: Context, id: String) -> Result<u8, Error> {
        todo!()
    }
    async fn close_sink_drain(self, _: Context, id: String) -> Result<bool, Error> {
        todo!()
    }
    async fn open_sink_drain(self, _: Context, id: String) -> Result<bool, Error> {
        todo!()
    }
    async fn get_sink_level(self, _: Context, id: String) -> Result<u8, Error> {
        todo!()
    }
}

#[tokio::main]
async fn main() {}
