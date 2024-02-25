extern crate image;

use log::info;

use hidapi::*;
use image::imageops::FilterType;
use image::DynamicImage;
use image::GenericImageView;
use std::cmp;
use std::ffi::CStr;
use std::ffi::CString;

pub struct StreamDeckController {
    hid_device: HidDevice,
}

impl StreamDeckController {
    const VENDOR_ID: u16 = 0x0fd9;
    const PRODUCT_ID: u16 = 0x0060;
    const ICON_SIZE: u32 = 72;

    const NUM_KEYS: i32 = 15;
    const NUM_COLUMNS: i32 = 5;
    // const NUM_ROWS: i32 = 3;

    const PAGE_PACKET_SIZE: usize = 8191;
    const NUM_FIRST_PAGE_PIXELS: u32 = 2583;
    const NUM_SECOND_PAGE_PIXELS: u32 = 2601;

    pub fn new() -> Result<StreamDeckController, HidError> {
        let mut hid_api = HidApi::new()?;

        hid_api.refresh_devices().unwrap();
        for dev in hid_api.device_list() {
            info!("Device:");
            info!("path: {:?}", dev.path());
            info!("manufacturer: {:?}", Some(dev.manufacturer_string()));
            info!("vendorid: {}", dev.vendor_id());
            info!("productid: {}", dev.product_id());
            info!("---");
        }

        let hid_device = hid_api.open(Self::VENDOR_ID, Self::PRODUCT_ID)?;
        let stream_deck = StreamDeckController { hid_device };

        Ok(stream_deck)
    }

    pub fn from_device_path(device_path: &str) -> Result<StreamDeckController, HidError> {
        let hid_api = HidApi::new().unwrap();

        let path_cstring: CString = CString::new(device_path).unwrap();
        let path_cstr: &CStr = path_cstring.as_c_str();
        let hid_device = hid_api.open_path(path_cstr)?;

        let stream_deck = StreamDeckController { hid_device };
        Ok(stream_deck)
    }

    pub fn get_num_keys(&self) -> i32 {
        Self::NUM_KEYS
    }

    pub fn create_keystates_buf(&self) -> Vec<u8> {
        vec![0; usize::try_from(Self::NUM_KEYS + 1).unwrap()]
    }

    pub fn get_keystates(&self) -> Vec<u8> // todo: needs return type
    {
        let timeout_ms = 500;
        let mut keystates_buf = self.create_keystates_buf();
        self.hid_device
            .read_timeout(keystates_buf.as_mut(), timeout_ms)
            .unwrap();
        keystates_buf
    }

    pub fn convert_key_index(&self, key_index: i32) -> i32 {
        let key_col = key_index % Self::NUM_COLUMNS;
        (key_index - key_col) + ((Self::NUM_COLUMNS - 1) - key_col)
    }

    pub fn fill_color(&self, key_index: i32, r: u8, g: u8, b: u8) {
        let device_key_index = self.convert_key_index(key_index);

        let pixel: Vec<u8> = vec![b, g, r];
        self.write_page_1(
            device_key_index,
            &self.create_buffer(
                usize::try_from(Self::NUM_FIRST_PAGE_PIXELS * 3).unwrap(),
                &pixel,
            ),
        );
        self.write_page_2(
            device_key_index,
            &self.create_buffer(
                usize::try_from(Self::NUM_SECOND_PAGE_PIXELS * 3).unwrap(),
                &pixel,
            ),
        );
    }

    fn create_buffer(&self, buffer_length: usize, repeated_fill_data: &Vec<u8>) -> Vec<u8> {
        let mut output_buffer = vec![0; buffer_length];
        let copy_len = repeated_fill_data.len();
        let max_copies = buffer_length / copy_len;
        for idx in 0..max_copies {
            let start_offset = idx * copy_len;
            let end_offset = start_offset + copy_len;
            output_buffer[start_offset..end_offset]
                .copy_from_slice(&repeated_fill_data[0..copy_len]);
        }

        output_buffer
    }

    pub fn fill_all_color(&self, r: u8, g: u8, b: u8) {
        for key_index in 0..Self::NUM_KEYS - 1 {
            self.fill_color(key_index, r, g, b);
        }
    }

    pub fn set_image(&self, key_index: i32, image_file_path: &str) {
        if key_index < 0 || key_index >= Self::NUM_KEYS {
            // TODO: how to do error here?! ImageError::Parameter()
        }
        let img = image::open(image_file_path).unwrap();
        let resized_image = img.resize(Self::ICON_SIZE, Self::ICON_SIZE, FilterType::Triangle);
        self.set_image_exact(key_index, resized_image);
    }

    pub fn clear_key(&self, key_index: i32) {
        self.fill_color(key_index, 0, 0, 0);
    }

    pub fn clear_all_keys(&self) {
        for key_index in 0..Self::NUM_KEYS - 1 {
            self.clear_key(key_index);
        }
    }

    pub fn reset(&self) {}

    pub fn set_brightness(&self, percentage: i32) {
        let clamped_percentage = num::clamp(percentage, 0, 100);
        let command_buffer = vec![
            0x05,
            0x55,
            0xaa,
            0xd1,
            0x01,
            u8::try_from(clamped_percentage).unwrap(),
        ];
        self.send_feature_report(&self.pad_buffer_to_length(&command_buffer, 17));
    }

    fn pad_buffer_to_length(&self, input_buffer: &Vec<u8>, desired_length: usize) -> Vec<u8> {
        let mut output_buffer: Vec<u8> = vec![0; desired_length];

        let len = cmp::min(input_buffer.len(), output_buffer.len());
        output_buffer[0..len].copy_from_slice(&input_buffer[0..len]);
        output_buffer
    }

    fn set_image_exact(&self, key_index: i32, image: DynamicImage) {
        let device_key_index = self.convert_key_index(key_index);

        info!(
            "set_image_exact.  key_index: {}  device_key_index: {}",
            key_index, device_key_index
        );

        let mut page_1_buf: Vec<u8> =
            vec![0; usize::try_from(Self::NUM_FIRST_PAGE_PIXELS * 3).unwrap()];
        let mut page_2_buf: Vec<u8> =
            vec![0; usize::try_from(Self::NUM_SECOND_PAGE_PIXELS * 3).unwrap()];

        info!("image dims: {}, {}", image.width(), image.height());

        for pixel_offset in 0..Self::NUM_FIRST_PAGE_PIXELS {
            // reverse the image
            let x = image.width() - 1 - (pixel_offset % image.width());
            let y = pixel_offset / image.width();
            if x < image.width() && y < image.height() {
                let pixel = image.get_pixel(x, y);
                let buffer_offset: usize = usize::try_from(pixel_offset * 3).unwrap();
                page_1_buf[buffer_offset] = pixel[2]; // B
                page_1_buf[buffer_offset + 1] = pixel[1]; // G
                page_1_buf[buffer_offset + 2] = pixel[0]; // R
            }
        }

        for pixel_offset in
            Self::NUM_FIRST_PAGE_PIXELS..Self::NUM_FIRST_PAGE_PIXELS + Self::NUM_SECOND_PAGE_PIXELS
        {
            let x = image.width() - 1 - (pixel_offset % image.width());
            let y = pixel_offset / image.width();

            if x < image.width() && y < image.height() {
                let pixel = image.get_pixel(x, y);

                let buffer_offset: usize =
                    usize::try_from((pixel_offset - Self::NUM_FIRST_PAGE_PIXELS) * 3).unwrap();
                page_2_buf[buffer_offset] = pixel[2]; // B
                page_2_buf[buffer_offset + 1] = pixel[1]; // G
                page_2_buf[buffer_offset + 2] = pixel[0]; // R
            }
        }

        self.write_page_1(device_key_index, &page_1_buf);
        self.write_page_2(device_key_index, &page_2_buf);
    }

    fn send_feature_report(&self, data: &Vec<u8>) {
        self.hid_device.send_feature_report(data).unwrap(); // todo: error handling
    }

    fn build_packet(
        &self,
        header: &Vec<u8>,
        buffer: &Vec<u8>,
        padded_buffer_length: usize,
    ) -> Vec<u8> {
        let output_buffer_len = header.len() + buffer.len();
        let mut output_buffer: Vec<u8> = vec![0; output_buffer_len];

        output_buffer[0..header.len()].copy_from_slice(&header[0..header.len()]);
        output_buffer[header.len()..output_buffer_len].copy_from_slice(&buffer[0..buffer.len()]);

        self.pad_buffer_to_length(&output_buffer, padded_buffer_length)
    }

    fn write_page_1(&self, key_index: i32, buffer: &Vec<u8>) {
        self.write(&self.build_packet(
            &self.get_page_one_header(key_index),
            buffer,
            Self::PAGE_PACKET_SIZE,
        ));
    }

    fn write_page_2(&self, key_index: i32, buffer: &Vec<u8>) {
        self.write(&self.build_packet(
            &self.get_page_two_header(key_index),
            buffer,
            Self::PAGE_PACKET_SIZE,
        ));
    }

    fn get_page_one_header(&self, key_index: i32) -> Vec<u8> {
        vec![
            0x02,
            0x01,
            0x01,
            0x00,
            0x00,
            u8::try_from(key_index + 1).unwrap(),
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x42,
            0x4d,
            0xf6,
            0x3c,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x36,
            0x00,
            0x00,
            0x00,
            0x28,
            0x00,
            0x00,
            0x00,
            0x48,
            0x00,
            0x00,
            0x00,
            0x48,
            0x00,
            0x00,
            0x00,
            0x01,
            0x00,
            0x18,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0xc0,
            0x3c,
            0x00,
            0x00,
            0xc4,
            0x0e,
            0x00,
            0x00,
            0xc4,
            0x0e,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
        ]
    }

    fn get_page_two_header(&self, key_index: i32) -> Vec<u8> {
        vec![
            0x02,
            0x01,
            0x02,
            0x00,
            0x01,
            u8::try_from(key_index + 1).unwrap(),
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
        ]
    }

    fn write(&self, data: &[u8]) {
        self.hid_device.write(data).unwrap();
    }
}
