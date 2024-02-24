use anyhow::Result;

use quick_xml::de::from_str;
use serde::Deserialize;

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

    pub fn turn_on(&self) -> Result<()> {
        // TODO: determine which (if not possibly both) that I need.

        // device power
        self.send_command("PWON")?;

        // main zone power
        self.send_command("ZMON")?;

        Ok(())
    }

    pub fn turn_off(&self) -> Result<()> {
        // device_power
        self.send_command("PWSTANDBY")?;

        // main zone power
        self.send_command("ZMSTANDBY")?;

        Ok(())
    }

    pub fn set_volume(&self, volume: i32) -> Result<()> {
        self.send_command(&format!("MV{:2}", num::clamp(volume, 0, 95)))?;
        Ok(())
    }

    pub fn get_volume(&self) -> Result<i32> {
        Ok(self.load_summary()?.volume)
    }

    pub fn mute(&self, mute: bool) -> Result<()> {
        if mute {
            self.send_command("MUOFF")?;
        }
        Ok(())
    }

    pub fn is_muted(&self) -> Result<bool> {
        let summary = self.load_summary()?;
        Ok(summary.is_muted)
    }

    pub fn select_input(&self, input: ReceiverInput) -> Result<()> {
        self.send_command(&format!("SI{}", receiver_input_to_command_string(input)))?;

        Ok(())
    }

    fn get_cmd_url(&self) -> String {
        format!(
            "http://{}/goform/formiPhoneAppDirect.xml?",
            self.device_address
        )
    }

    fn get_status_url(&self) -> String {
        format!("http://{}/goform/", self.device_address)
    }

    fn send_command(&self, command: &str) -> Result<()> {
        let url = format!("{}{}", self.get_cmd_url(), urlencoding::encode(command));
        reqwest::blocking::get(url)?;
        Ok(())
    }

    fn load_summary(&self) -> Result<DenonSummary> {
        let url = format!("{}formMainZone_MainZoneXml.xml", self.get_status_url());
        // info!("{}", url);
        let response = reqwest::blocking::get(url).unwrap();
        let response_text = response.text().unwrap();

        // info!("{}", response_text);

        let summary_struct: DenonSummaryRaw = from_str(&response_text)?;

        let is_muted = summary_struct.mute.value.text == "on";
        let is_volume_absolute = summary_struct.volume_display.value.text == "Absolute";
        let volume_value = summary_struct.master_volume.value.text;
        let mut volume = 0;
        if volume_value != "--" {
            let f = volume_value.parse::<f64>()?;

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

        Ok(DenonSummary { volume, is_muted })
    }
}

pub enum ReceiverInput {
    Phono,
    Cd,
    Tuner,
    Dvd,
    Tv,
    SatCbl,
    Mplay,
    Game,
    HdRadio,
    Net,
    Pandora,
    SiriusXm,
    Spotify,
    LastFm,
    Flickr,
    Radio,
    Server,
    Favorites,
    Aux1,
    Aux2,
    Bluetooth,
    UsbIpod,
    Usb,
    IpodDirect,
    RadioRecent,
    UsbFavorites,
}

fn receiver_input_to_command_string(input: ReceiverInput) -> String {
    match input {
        ReceiverInput::Phono => "PHONO".to_string(),
        ReceiverInput::Cd => "CD".to_string(),
        ReceiverInput::Tuner => "TUNER".to_string(),
        ReceiverInput::Dvd => "DVD".to_string(),
        ReceiverInput::Tv => "TV".to_string(),
        ReceiverInput::SatCbl => "SAT/CBL".to_string(),
        ReceiverInput::Mplay => "MPLAY".to_string(),
        ReceiverInput::Game => "GAME".to_string(),
        ReceiverInput::HdRadio => "HDRADIO".to_string(),
        ReceiverInput::Net => "NET".to_string(),
        ReceiverInput::Pandora => "PANDORA".to_string(),
        ReceiverInput::SiriusXm => "SIRIUSXM".to_string(),
        ReceiverInput::Spotify => "SPOTIFY".to_string(),
        ReceiverInput::LastFm => "LASTFM".to_string(),
        ReceiverInput::Flickr => "FLIKR".to_string(),
        ReceiverInput::Radio => "IRADIO".to_string(),
        ReceiverInput::Server => "SERVER".to_string(),
        ReceiverInput::Favorites => "FAVORITES".to_string(),
        ReceiverInput::Aux1 => "AUX1".to_string(),
        ReceiverInput::Aux2 => "AUX2".to_string(),
        ReceiverInput::Bluetooth => "BT".to_string(),
        ReceiverInput::UsbIpod => "USB/IPOD".to_string(),
        ReceiverInput::Usb => "USB".to_string(),
        ReceiverInput::IpodDirect => "IPD".to_string(),
        ReceiverInput::RadioRecent => "IRP".to_string(),
        ReceiverInput::UsbFavorites => "FVP".to_string(),
    }
}
