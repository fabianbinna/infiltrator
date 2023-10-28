use byte_unit::Byte;
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Config {
    pub download_part_size: Byte,
    pub upload_part_size: Byte,
    pub data_path: String,
    pub max_simultaneous_uploads: usize,
    pub max_upload_directory_size: Byte
}

impl Default for Config {
    fn default() -> Self {
        Config {
            download_part_size: Byte::from_str("10 MB").unwrap(),
            upload_part_size: Byte::from_str("10 MB").unwrap(),
            data_path: String::from("data/"),
            max_simultaneous_uploads: 10,
            max_upload_directory_size: Byte::from_str("1 GB").unwrap()
        }
    }
}
