// use std::{io::Read, path::Path, process, time::Duration};
use std::{io::Read, path::Path, time::Duration};

use ggez::{filesystem::Filesystem, graphics::Font, Context};
// use log::{error, info};
use serde::de::DeserializeOwned;

use crate::ZResult;

pub fn time_s(s: f32) -> Duration {
    let ms = s * 1000.0;
    Duration::from_millis(ms as u64)
}

#[allow(dead_code)] // TODO: remove the attribute
pub fn check_assets_hash(_fs: &mut Filesystem, _expected: &str) -> ZResult {
    unimplemented!() // TODO: !

    /*
    let mut file = fs.open("/.checksum.md5")?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let real = data.trim();
    if real != expected {
        let error_code = 1;
        error!("Bad assets checksum {} (expected {})", real, expected);
        process::exit(error_code);
    }
    info!("Assets checksum is Ok");
    Ok(())
    */
}

pub fn read_file_to_string<P: AsRef<Path>>(context: &mut Context, path: P) -> ZResult<String> {
    let mut buf = String::new();
    let mut file = ggez::filesystem::open(context, path)?;
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

pub fn deserialize_from_file<P, D>(context: &mut Context, path: P) -> ZResult<D>
where
    P: AsRef<Path>,
    D: DeserializeOwned,
{
    let path = path.as_ref();
    let s = read_file_to_string(context, path)?;
    // let d = ron::de::from_str(&s).map_err(|e| format!("Can't deserialize {:?}: {:?}", path, e))?;
    let d = ron::de::from_str(&s).expect("TODO: ERROR MESSAGE!");
    Ok(d)
}

pub fn default_font(context: &mut Context) -> Font {
    // Font::new(context, "/OpenSans-Regular.ttf", 32).expect("Can't load the default font")
    Font::new(context, "/OpenSans-Regular.ttf").expect("Can't load the default font")
}
