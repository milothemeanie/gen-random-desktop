#[macro_use]
extern crate serde_derive;
use restson::{RestPath, Error, RestClient};
use serde::{Deserialize, Deserializer};

#[derive(Serialize,Deserialize)]
struct Photo {
    id: String,
    description: String,
    links: Links,
}

#[derive(Serialize,Deserialize)]
struct Links {
    download: String
}

impl RestPath<(&str)> for Photo {
    fn get_path(param: &str) -> Result<String, Error> {
        Ok(format!("photos/{}", param.to_string()))
    }
}

fn main() {
    println!("retrieving a random photo");

    let mut client = RestClient::new("https://api.unsplash.com").unwrap();

    let query = vec![("client_id","3ac00ee4846a33d1d4b87cdff1d57f7471309a7d5b2639cba011a2187eee4cad")];
    let data: Photo = client.get_with("random", &query).unwrap();

    println!("received photo, id:{} description:{}, downloadlink:{}",
             data.id, data.description, data.links.download)
}


//https://api.unsplash.com/photos/random?client_id=3ac00ee4846a33d1d4b87cdff1d57f7471309a7d5b2639cba011a2187eee4cad