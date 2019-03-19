extern crate requests;

use std::fs;
use std::path::Path;

use requests::ToJson;

use std::process::Command;


struct Photo {
    id: String,
    description: String,
    download_link: String,
    width: u32,
    height: u32,
}

fn main() {
    println!("retrieving a random photo");
    let response = requests::get("https://api.unsplash.com/photos/random?client_id=3ac00ee4846a33d1d4b87cdff1d57f7471309a7d5b2639cba011a2187eee4cad").unwrap();
    let data = response.json().unwrap();

//    print!("{}", data["id"]);
    let data = Photo {
        id: data["id"].to_string(),
        description: data["description"].to_string(),
        download_link: data["links"]["download"].to_string(),
        width: data["width"].as_u32().unwrap(),
        height: data["height"].as_u32().unwrap(),
    };

    println!("received photo");
    println!("id:{} description:{}, downloadlink:{}, width:{}, height: {}",
             data.id, data.description, data.download_link, data.width, data.height);

    let image_response = requests::get(data.download_link).unwrap();
    let image_response = image_response.content();

    let path_string = &format!("/tmp/{}.jpg", data.id);
    let path = Path::new(path_string);

    fs::write(path, image_response).expect("failed to write image");

    let image_parm = format!(r#"'file:///{}'"#, path_string);


    Command::new("dconf")
        .arg("write")
        .arg("/org/cinnamon/desktop/background/picture-uri")
        .arg(image_parm)
        .spawn()
        .expect("failed to set the wallpaper");
}

//https://api.unsplash.com/photos/random?client_id=3ac00ee4846a33d1d4b87cdff1d57f7471309a7d5b2639cba011a2187eee4cad
//gconftool-2 -t str --set /desktop/gnome/background/picture_filename
