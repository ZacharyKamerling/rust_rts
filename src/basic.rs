extern crate byteorder;

use self::byteorder::{ReadBytesExt, WriteBytesExt, BigEndian, LittleEndian};
use std::io::Cursor;

fn serialize(vec: &mut Cursor<Vec<u8>>) {
    let _ = vec.write_u16::<BigEndian>(0);
    let _ = vec.read_u16::<BigEndian>();
}

fn deserialize() {
    
}