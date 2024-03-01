use log::info;
use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use streamdeck::StreamDeckController;
use theater::{DenonReceiver, ReceiverInput, Theater};

struct DeviceAddresses {}
impl DeviceAddresses {
    pub fn projector() -> String {
        "192.168.2.151".to_string()
    }
    pub fn receiver() -> String {
        "192.168.2.198".to_string()
    }
}

fn main() -> anyhow::Result<(), anyhow::Error> {
    TermLogger::init(
        LevelFilter::Debug,
        ConfigBuilder::new()
            .set_target_level(LevelFilter::Error)
            .set_location_level(LevelFilter::Error)
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;

    info!("Starting up...");
    do_main_loop()?;
    info!("Exiting...");

    Ok(())
}

fn get_resource_path(resource_file_name: &str) -> String {
    let path = Path::new("resources").join(resource_file_name);
    path.to_string_lossy().to_string()
}

#[allow(dead_code)]
fn do_main_loop() -> anyhow::Result<(), anyhow::Error> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    info!("Initializing the deck");

    // Create the StreamDeckController from the vendor/product id
    let controller = StreamDeckController::new().unwrap();
    let theater = Theater::new(&DeviceAddresses::receiver(), &DeviceAddresses::projector());

    // set default brightness
    controller.set_brightness(5);

    const XBOX_KEY_IDX: i32 = 0;
    const PLAYSTATION_KEY_IDX: i32 = 1;
    const APPLETV_IDX: i32 = 2;
    const MUTE_IDX: i32 = 4;
    const VOLUP_IDX: i32 = 9;
    const POWEROFF_IDX: i32 = 10;
    const DIM_IDX: i32 = 11;
    const BRIGHT_IDX: i32 = 12;
    const VOLDOWN_IDX: i32 = 14;

    // Load the images into the streamdeck
    for key_index in 0..controller.get_num_keys() {
        // info!("setting image for key_index: {}", key_index);
        match key_index {
            XBOX_KEY_IDX => controller.set_image(key_index, &get_resource_path("xbox.png")),
            PLAYSTATION_KEY_IDX => controller.set_image(key_index, &get_resource_path("ps4.png")),
            APPLETV_IDX => controller.set_image(key_index, &get_resource_path("appletv.png")),
            BRIGHT_IDX => controller.set_image(key_index, &get_resource_path("bright.png")),
            DIM_IDX => controller.set_image(key_index, &get_resource_path("dim.png")),
            VOLUP_IDX => controller.set_image(key_index, &get_resource_path("volup.png")),
            VOLDOWN_IDX => controller.set_image(key_index, &get_resource_path("voldown.png")),
            MUTE_IDX => controller.set_image(key_index, &get_resource_path("volmute.png")),
            POWEROFF_IDX => controller.set_image(key_index, &get_resource_path("poweroff.png")),
            _ => controller.set_image(key_index, &get_resource_path("blank.png")),
        }
    }

    let mut last_key_states = controller.create_keystates_buf();

    info!("Starting the keypress loop");
    while running.load(Ordering::SeqCst) {
        // check hid_device read timeout to get the state of the keys / check for button presses

        let mut keys_pressed: Vec<i32> = vec![];
        let mut keys_released: Vec<i32> = vec![];

        let keystates = controller.get_keystates();
        for idx in 1..controller.get_num_keys() + 1 {
            let idx_usize = usize::try_from(idx).unwrap();
            if keystates[idx_usize] != last_key_states[idx_usize] {
                let converted_keypress: i32 = controller.convert_key_index(idx - 1);
                if keystates[idx_usize] > 0 {
                    info!("raw key pressed: {}", idx_usize - 1);
                    info!("converted key pressed: {}", converted_keypress);
                    keys_pressed.push(converted_keypress);
                } else {
                    keys_released.push(converted_keypress);
                }
            }
        }
        last_key_states = keystates;

        for keyidx in keys_pressed.iter() {
            info!("processing key pressed: {}", *keyidx);

            match *keyidx {
                XBOX_KEY_IDX => theater.turn_on(ReceiverInput::Game),
                PLAYSTATION_KEY_IDX => theater.turn_on(ReceiverInput::Dvd),
                APPLETV_IDX => theater.turn_on(ReceiverInput::Mplay),
                BRIGHT_IDX => controller.set_brightness(75),
                DIM_IDX => controller.set_brightness(5),
                VOLUP_IDX => theater.set_volume(theater.get_volume() + 5),
                VOLDOWN_IDX => theater.set_volume(theater.get_volume() - 5),
                MUTE_IDX => theater.toggle_mute(),
                POWEROFF_IDX => theater.turn_off(),
                _ => {}
            }
        }
    }

    Ok(())
}
