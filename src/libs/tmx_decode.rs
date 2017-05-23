

use std::fs::File;
use std::io::prelude::*;
use serde_json;

pub fn decode(map_name: &str) -> MapData {
    let mut file = File::open(map_name).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let map_data: MapData = serde_json::from_str(&contents).unwrap();

    println!("{:?} x {:?}", map_data.width, map_data.height);

    for layer in &map_data.layers {
        println!("{:?}", layer.name);

        if let Some(ref data) = layer.data {
            println!("Data: {:?}", data.len());
        }

        if let Some(ref objects) = layer.objects {
            println!("Objects: {:?}", objects.len());
        }
    }

    map_data
}

#[derive(Serialize, Deserialize)]
pub struct MapData {
    width: usize,
    height: usize,
    layers: Vec<Layer>,
}

#[derive(Serialize, Deserialize)]
pub struct Layer {
    name: String,
    data: Option<Vec<usize>>,
    objects: Option<Vec<Location>>,
}

#[derive(Serialize, Deserialize)]
pub struct Location {
    x: usize,
    y: usize,
}