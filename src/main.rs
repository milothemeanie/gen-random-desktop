extern crate env_logger;
extern crate log;
extern crate requests;
extern crate structopt;
extern crate eventual;

use std::fs;
use std::fs::File;
use std::io::{Write, Read};
use std::path::Path;
use std::process::Command;

use env_logger::Env;
use log::{info, debug};
use requests::ToJson;
use structopt::StructOpt;
use eventual::Timer;

struct Photo {
    id: String,
    description: String,
    download_link: String,
    width: u32,
    height: u32,
    raw_json: String,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "gen-random-desktop", about = "sets desktop wallpaper to a random image from unsplash")]
struct Opt {
    /// Save last random wallpaper
    #[structopt(short = "s", long = "save")]
    save: bool,

    /// Print last random wallpaper details in json format
    #[structopt(short = "d", long = "detail")]
    detail: bool,

    /// Set wallpaper to a random image
    #[structopt(short = "r", long = "random")]
    random: bool,

    /// Set random wallpaper per X minutes
    #[structopt(short = "t", long = "timer", default_value="0")]
    timer: u32,
}


/// clientid is assigned from unsplash.com for api usage
const CLIENT_ID: &'static str = "3ac00ee4846a33d1d4b87cdff1d57f7471309a7d5b2639cba011a2187eee4cad";

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let opt = Opt::from_args();
    debug!("{:?}", opt);

    let gen_folder = Path::new("/tmp/gen_random_desktop");

    if opt.timer > 0 {
        let timer = Timer::new();

        let ticks = timer.interval_ms(minutes_to_milli(opt.timer)).iter();
        for _ in ticks {
            set_random_wallpaper(gen_folder);
        }
    }

    set_random_wallpaper(gen_folder);

    if opt.save {
        save_last_wallpaper(gen_folder);
    }
    if opt.detail {
        print_current_details(gen_folder);
    }
}

fn minutes_to_milli(minutes: u32) -> u32{
    let milli = (minutes * 60) * 1000;

    milli
}

fn save_last_wallpaper(gen_folder: &Path) {
    let id = retrieve_current_id(gen_folder);

    copy_to_save_location(gen_folder, &id, ".jpg", true);
    copy_to_save_location(gen_folder, &id, ".json", false);
}

fn retrieve_current_id(gen_folder: &Path) -> String {
    let id_file = &format!("{}/current_wallpaper_id", gen_folder.to_str().unwrap());
    let mut file = File::open(id_file).expect("Failed to read current_wallpaper_id file");
    let mut id = String::new();
    file.read_to_string(&mut id).expect("Failed retrieve id from current_wallpaper_id file");
    id
}

fn print_current_details(gen_folder: &Path) {
    let detail_file = &format!("{}/{}.json", gen_folder.to_str().unwrap(), retrieve_current_id(gen_folder));
    let mut file = File::open(detail_file).expect("Failed to read current_wallpaper_id file");
    let mut detail = String::new();
    file.read_to_string(&mut detail).expect("Failed retrieve id from current_wallpaper_id file");

    info!("{}", detail)
}

fn copy_to_save_location(gen_folder: &Path, id: &String, ext: &str, save: bool) {
    let image_file = &format!("{}/{}{}", gen_folder.to_str().unwrap(), id, ext);
    let move_location = &format!("/home/cward/Pictures/{}{}", id, ext);
    info!("Saving last wallpaper {} to {}", image_file, move_location);
    fs::copy(image_file, move_location).expect("Failed to save last wallpaper");

    if save {
        set_wallpaper_cinnamon(move_location);
    }
}

fn set_random_wallpaper(gen_folder: &Path) {
    let data = retrieve_photo();
    if !gen_folder.exists() {
        fs::create_dir(gen_folder).expect("unable to create temp directory");
    }
    let image_path_string = &format!("{}/{}.jpg", gen_folder.to_str().unwrap(), data.id);
    write_image(&data.download_link, image_path_string);
    set_wallpaper_cinnamon(image_path_string);
    let data = write_description_file(data, gen_folder);
    write_current_wallpaper_file(data.id, gen_folder);
}

fn write_description_file(data: Photo, gen_folder: &Path) -> Photo {
    let description_file = &format!("{}/{}.json", gen_folder.to_str().unwrap(), data.id);
    let description_file = Path::new(description_file);
    let mut file = File::create(description_file).expect("Failed creating the description file");
    file.write_all(data.raw_json.as_bytes()).expect("Failed writing the description file");
    data
}

fn write_current_wallpaper_file(id: String, gen_folder: &Path) {
    let id_file = &format!("{}/current_wallpaper_id", gen_folder.to_str().unwrap());
    let mut file = File::create(id_file).expect("Failed to create current_wallpaper_id file");
    file.write_all(id.as_bytes()).expect("Failed writing photo id in current_wallpaper_id file");
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

