use std::{thread, time};

use anyhow::Result;
mod denon;
mod epson;
use log::error;

pub use self::denon::{DenonReceiver, ReceiverInput};
pub use self::epson::EpsonProjector;

pub struct Theater {
    receiver: DenonReceiver,
    projector: EpsonProjector,
}

impl Theater {
    pub fn new(denon_address: &str, epson_address: &str) -> Theater {
        Theater {
            projector: EpsonProjector::new(epson_address),
            receiver: DenonReceiver::new(denon_address),
        }
    }

    pub fn turn_on(&self, input: ReceiverInput) {
        let do_steps = || -> Result<()> {
            self.receiver.turn_on()?;
            self.projector.turn_on()?;

            // Give the receiver time to turn on.
            // The projector takes a bit to spin up anyway.
            thread::sleep(time::Duration::from_millis(1500));
            self.receiver.select_input(input)?;
            Ok(())
        };
        match do_steps() {
            Ok(()) => {}
            Err(e) => error!("{}", e),
        }
    }

    pub fn turn_off(&self) {
        match self.projector.turn_off() {
            Ok(()) => {}
            Err(e) => error!("{}", e),
        }
        match self.receiver.turn_off() {
            Ok(()) => {}
            Err(e) => error!("{}", e),
        }
    }

    pub fn set_volume(&self, volume: i32) {
        match self.receiver.set_volume(volume) {
            Ok(()) => {}
            Err(e) => error!("{}", e),
        }
    }

    pub fn get_volume(&self) -> i32 {
        match self.receiver.get_volume() {
            Ok(x) => x,
            Err(e) => {
                error!("{}", e);
                0
            }
        }
    }

    pub fn toggle_mute(&self) {
        match self.receiver.is_muted() {
            Ok(is_muted) => match self.receiver.mute(!is_muted) {
                Ok(()) => {}
                Err(e) => error!("{}", e),
            },
            Err(e) => error!("{}", e),
        }
    }
}
