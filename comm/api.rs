use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum FromClientMessage {
    Ping, // Asks for Pong
    //ChunkSize(usize), // Used when a message is potentially to big - not needed in websockets, yay
    Click(u16, u16), // Click at this location x / y
}

#[derive(Serialize, Deserialize, Debug)]
pub enum FromServerMessage {
    Pong, // Answers for Ping
    Screen(Vec<u8>),
}
