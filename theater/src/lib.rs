mod denon;
mod epson;

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
        self.receiver.turn_on();
        self.projector.turn_on();
        // todo: sleep to ensure rcvr is on?
        DenonReceiver::select_input(input);
    }

    pub fn turn_off(&self) {
        self.projector.turn_off();
        self.receiver.turn_off();
    }

    pub fn set_volume(&self, volume: i32) {
        self.receiver.set_volume(volume);
    }

    pub fn get_volume(&self) -> i32 {
        self.receiver.get_volume()
    }
}
