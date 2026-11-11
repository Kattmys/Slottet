use serde::{Serialize, Deserialize};

#[inline]
pub fn serialize(data: impl Serialize) -> Result<Vec<u8>, Box<bincode::ErrorKind>> {
    let mut buffer: Vec<u8> = Vec::new();
    let options = bincode::options();
    let mut serializer = bincode::Serializer::new(&mut buffer, options);
    data.serialize(&mut serializer)?;
    Ok(buffer)
}

#[inline]
pub fn deserialize<T: for<'a> Deserialize<'a>>(data: &renet::Bytes) -> Result<T, Box<bincode::ErrorKind>> {
    let data = data.to_vec();
    let options = bincode::options();
    let mut deserializer = bincode::Deserializer::from_slice(&data, options);
    T::deserialize(&mut deserializer)
}
