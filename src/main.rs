extern crate env_logger;
extern crate log;
extern crate requests;

use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use env_logger::Env;
use log::info;
use requests::ToJson;

struct Photo {
    id: String,
    description: String,
    download_link: String,
    width: u32,
    height: u32,
    raw_json: String,
}

/// clientid is assigned from unsplash.com for api usage
const CLIENT_ID: &'static str = "3ac00ee4846a33d1d4b87cdff1d57f7471309a7d5b2639cba011a2187eee4cad";

fn main() {
    env_logger::from_env(Env::default().default_filter_or("info")).init();

    let data = retrieve_photo();

    let gen_folder = Path::new("/tmp/gen_random_desktop/");
    if !gen_folder.exists()
    {
        fs::create_dir(gen_folder).expect("unable to create temp directory");
    }

    let image_path_string = &format!("{}/{}.jpg", gen_folder.to_str().unwrap(), data.id);
    write_image(&data.download_link, image_path_string);
    set_wallpaper_cinnamon(image_path_string);
    write_description_file(data, gen_folder);
}

fn write_description_file(data: Photo, gen_folder: &Path) {
    let description_file = &format!("{}/{}.json", gen_folder.to_str().unwrap(), data.id);
    let description_file = Path::new(description_file);
    let mut file = File::create(description_file).expect("Failed creating the description file");
    file.write_all(data.raw_json.as_bytes()).expect("Failed writing the description file");
}

fn retrieve_photo() -> Photo {
    info!("retrieving a random photo");
    let response = requests::get(format!("https://api.unsplash.com/photos/random?client_id={}", CLIENT_ID)).unwrap();
    let data = response.json().unwrap();
    let data = Photo {
        id: data["id"].to_string(),
        description: data["description"].to_string(),
        download_link: data["links"]["download"].to_string(),
        width: data["width"].as_u32().unwrap(),
        height: data["height"].as_u32().unwrap(),
        raw_json: data.pretty(4),
    };

    info!("<----------retrieve photo success---------->");
    info!("  id:{}", data.id);
    info!("  description:{}", data.description);
    info!("  downloadlink:{}", data.download_link);
    info!("  width:{}", data.width);
    info!("  height:{}", data.height);

    data
}

fn set_wallpaper_cinnamon(image_path_string: &String) {
    let image_parm = format!(r#"'file:///{}'"#, image_path_string);
    Command::new("dconf")
        .arg("write")
        .arg("/org/cinnamon/desktop/background/picture-uri")
        .arg(image_parm)
        .spawn()
        .expect("failed to set the wallpaper, might need to install dconf-cli");
}

fn write_image(download_link: &String, path_string: &String) {
    let path = Path::new(path_string);
    let image_response = requests::get(download_link).unwrap();
    let image_response = image_response.content();
    fs::write(path, image_response).expect("failed to write image");
}

