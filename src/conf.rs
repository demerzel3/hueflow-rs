use std::env::var_os;

pub fn get_username() -> String {
    var_os("HUE_USERNAME")
        .expect("Please set the env var HUE_USERNAME")
        .into_string()
        .expect("Invalid data in HUE_USERNAME env var")
}

pub fn get_lat() -> f64 {
    var_os("LAT")
        .expect("Please set the env var LAT")
        .into_string()
        .expect("Invalid data in LAT env var")
        .parse()
        .expect("Invalid data in LAT env var")
}

pub fn get_lng() -> f64 {
    var_os("LNG")
        .expect("Please set the env var LNG")
        .into_string()
        .expect("Invalid data in LNG env var")
        .parse()
        .expect("Invalid data in LNG env var")
}
