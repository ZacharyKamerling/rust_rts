extern crate byteorder;

use self::byteorder::{WriteBytesExt, BigEndian};
use enum_primitive::FromPrimitive;
use std::io::Cursor;
use std::fs::File;
use std::io::prelude::*;
use serde_json;
use data::aliases::*;

pub struct MapData {
    width: usize,
    height: usize,
    collisions: Vec<usize>,
    tiles: Vec<Location>,
    start_locations: Vec<Location>,
    prime_nodes: Vec<Location>,
}

#[derive(Serialize, Deserialize)]
struct TempMapData {
    width: usize,
    height: usize,
    tileheight: usize,
    tilewidth: usize,
    layers: Vec<Layer>,
}

#[derive(Serialize, Deserialize)]
struct Layer {
    name: String,
    data: Option<Vec<usize>>,
    collision: Option<Vec<usize>>,
    objects: Option<Vec<Location>>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
struct Location {
    x: usize,
    y: usize,
}

impl Location {
    pub fn xy(self) -> (usize, usize) {
        (self.x, self.y)
    }
}

impl MapData {
    pub fn collisions(&self) -> &Vec<usize> {
        &self.collisions
    }

    pub fn new(map_name: &str) -> MapData {
        let tilesheet_w = 2048;
        let mut file = File::open(map_name).unwrap();
        let mut contents = String::new();

        file.read_to_string(&mut contents).unwrap();

        let mut tiles = Vec::new();
        let mut collisions = Vec::new();
        let mut start_locations = Vec::new();
        let mut prime_nodes = Vec::new();
        let temp: TempMapData = serde_json::from_str(&contents).unwrap();
        let w = temp.width;
        let h = temp.height;
        let tw = temp.tilewidth;
        let th = temp.tilewidth;
        let modi = tilesheet_w / tw;

        for layer in &temp.layers {
            if layer.name == "terrain" {
                if let Some(ref data) = layer.data {

                    for tile in data {
                        let x = (tile - 1) % modi;
                        let y = (tile - 1) / modi;

                        tiles.push(Location { x: x, y: y });
                    }
                }
            }

            if layer.name == "collision" {
                if let Some(ref data) = layer.data {

                    for &collision in data {
                        let col_type_id = match MoveType::from_usize(collision).unwrap() {
                            MoveType::None => 0,
                            MoveType::Ground => 1,
                            MoveType::Air => 2,
                            MoveType::Water => 3,
                            MoveType::Hover => 4,
                        };

                        collisions.push(col_type_id);
                    }
                }
            }

            if layer.name == "start_locations" {
                if let Some(ref objects) = layer.objects {

                    for start_loc in objects {
                        let x = start_loc.x / tw;
                        let y = start_loc.y / th;
                        start_locations.push(Location { x: x, y: y });
                    }
                }
            }

            if layer.name == "prime_nodes" {
                if let Some(ref objects) = layer.objects {

                    for start_loc in objects {
                        let x = start_loc.x / tw;
                        let y = start_loc.y / th;
                        prime_nodes.push(Location { x: x, y: y });
                    }
                }
            }
        }

        MapData {
            width: w,
            height: h,
            collisions: collisions,
            tiles: tiles,
            start_locations: start_locations,
            prime_nodes: prime_nodes,
        }
    }

    pub fn width_and_height(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut vec = Cursor::new(Vec::new());

        let _ = vec.write_u16::<BigEndian>(self.width as u16);
        let _ = vec.write_u16::<BigEndian>(self.height as u16);

        for tile in &self.tiles {
            let _ = vec.write_u8(tile.x as u8);
            let _ = vec.write_u8(tile.y as u8);
        }

        for &collision in &self.collisions {
            let _ = vec.write_u8(collision as u8);
        }

        let _ = vec.write_u8(self.start_locations.len() as u8);

        for start_location in &self.start_locations {
            let _ = vec.write_u16::<BigEndian>(start_location.x as u16);
            let _ = vec.write_u16::<BigEndian>(start_location.y as u16);
        }

        let _ = vec.write_u32::<BigEndian>(self.prime_nodes.len() as u32);

        for prime_node in &self.prime_nodes {
            let _ = vec.write_u16::<BigEndian>(prime_node.x as u16);
            let _ = vec.write_u16::<BigEndian>(prime_node.y as u16);
        }

        vec.into_inner()
    }
}