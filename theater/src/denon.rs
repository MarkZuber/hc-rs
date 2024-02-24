// use log::info;
use quick_xml::de::from_str;
use serde::Deserialize;

pub enum ReceiverInput {
    Xbox,
    Playstation,
    AppleTv,
}

pub struct DenonReceiver {
    device_address: String,
}

#[derive(Debug, Deserialize)]
struct DenonSummaryValueItem {
    #[serde(rename = "$value")]
    text: String,
}

#[derive(Debug, Deserialize)]
struct DenonSummaryItem {
    #[serde(rename = "value")]
    value: DenonSummaryValueItem,
}

#[derive(Debug, Deserialize)]
struct DenonSummaryRaw {
    #[serde(rename = "Mute")]
    mute: DenonSummaryItem,

    #[serde(rename = "VolumeDisplay")]
    volume_display: DenonSummaryItem,

    #[serde(rename = "MasterVolume")]
    master_volume: DenonSummaryItem,
}

struct DenonSummary {
    #[allow(dead_code)]
    is_muted: bool,
    volume: i32,
}

impl DenonReceiver {
    pub fn new(device_address: &str) -> DenonReceiver {
        DenonReceiver {
            device_address: device_address.to_string(),
        }
    }

    pub fn turn_on(&self) {
        // TODO: determine which (if not possibly both) that I need.

        // device power
        self.send_command("PWON");

        // main zone power
        self.send_command("ZMON");
    }

    pub fn turn_off(&self) {
        // device_power
        self.send_command("PWSTANDBY");

        // main zone power
        self.send_command("ZMSTANDBY");
    }

    pub fn set_volume(&self, volume: i32) {
        self.send_command(&format!("MV{:2}", num::clamp(volume, 0, 95)));
    }

    pub fn get_volume(&self) -> i32 {
        self.load_summary().volume
    }

    pub fn mute(&self, mute: bool) {
        if mute {
            self.send_command("MUON");
        } else {
            self.send_command("MUOFF");
        }
    }

    pub fn select_input(_input: ReceiverInput) {}

    fn get_cmd_url(&self) -> String {
        format!(
            "http://{}/goform/formiPhoneAppDirect.xml?",
            self.device_address
        )
    }

    fn get_status_url(&self) -> String {
        format!("http://{}/goform/", self.device_address)
    }

    fn send_command(&self, command: &str) {
        let url = format!("{}{}", self.get_cmd_url(), urlencoding::encode(command));
        reqwest::blocking::get(url).unwrap();
    }

    fn load_summary(&self) -> DenonSummary {
        let url = format!("{}formMainZone_MainZoneXml.xml", self.get_status_url());
        // info!("{}", url);
        let response = reqwest::blocking::get(url).unwrap();
        let response_text = response.text().unwrap();

        // info!("{}", response_text);

        let parse_result: Result<DenonSummaryRaw, _> = from_str(&response_text);

        match parse_result {
            Ok(summary_struct) => {
                let is_muted = summary_struct.mute.value.text == "on";
                let is_volume_absolute = summary_struct.volume_display.value.text == "Absolute";
                let volume_value = summary_struct.master_volume.value.text;
                let mut volume = 0;
                if volume_value != "--" {
                    let f = volume_value.parse::<f64>().unwrap();

                    let mut v = if f > f64::from(i32::MAX) {
                        i32::MAX
                    } else if f < f64::from(i32::MIN) {
                        i32::MIN
                    } else {
                        f.round() as i32
                    };

                    if is_volume_absolute {
                        v += 80;
                    }
                    volume = v;
                }

                DenonSummary { volume, is_muted }
            }
            Err(_) => DenonSummary {
                volume: 0,
                is_muted: false,
            },
        }
    }
}
