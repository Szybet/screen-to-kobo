mod api;
mod server;

// Logging
use log::{debug, error, info, warn};

// Network
pub use api::{FromClientMessage, FromServerMessage};
use message_io::network::{Endpoint, ResourceId, SendStatus, Transport};
use message_io::node::{self, NodeHandler};
use std::net::ToSocketAddrs;

// Threads
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::{fs, thread, time};

// Arguments
use clap::Parser;

// Other
use rand::Rng;
use std::process::Command;
use std::time::Instant;

pub fn send_network(
    network_handler: &NodeHandler<()>,
    endpoint: Option<Endpoint>,
    message: FromServerMessage,
) {
    if let Some(endpoint) = endpoint {
        let output_data = bincode::serialize(&message).unwrap();
        let status = network_handler.network().send(endpoint, &output_data);
        //debug!("Status of message {:?} is {:?}", message, status);
        if status != SendStatus::Sent {
            error!("Packet not send?");
        }
    } else {
        error!("Failed to send network message: missing endpoint");
    }
}

pub enum ThreadCom {
    ClientConnected(Endpoint, ResourceId),
}

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"),
    );
    debug!("Starting mirKobo-host");

    let mut endpointSaved: Option<Endpoint> = None;

    // Threads
    let (tx_to_gui, rx_to_gui) = mpsc::channel();

    // Network
    let addr = ("0.0.0.0", 24356)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();
    let (handler, listener) = node::split::<()>();
    let network_handler = Arc::new(handler);
    let transport = Transport::Ws;
    match network_handler.network().listen(transport, addr) {
        Ok((_id, real_addr)) => info!("Server running at {} by {}", real_addr, transport),
        Err(_) => error!("Can not listening at {} by {}", addr, transport),
    }

    let network_handler_server = network_handler.clone();
    thread::spawn(move || {
        server::run(network_handler_server, listener, tx_to_gui); // Enable websockets
    });

    if let Ok(event) = rx_to_gui.recv() {
        match event {
            ThreadCom::ClientConnected(endpoint, _resource_id) => {
                info!("Server received: ClientConnected");
                endpointSaved = Some(endpoint);
            }
        }
    }

    let mut sha_of_file: String = String::new();
    loop {
        //debug!("Taking ss");
        // scrot -o -Z 9 -a 0,0,1280,1024 /tmp/ss.png
        let ss_time = Instant::now();
        Command::new("scrot")
            .arg("-o")
            .arg("-Z")
            .arg("0")
            .arg("-p")
            .arg("-a")
            .arg("0,56,1280,1024")
            .arg("/tmp/ss.png")
            .output()
            .expect("Failed to scrot");
        let ss_end = Instant::now();
        debug!("ss time: {}", ss_end.duration_since(ss_time).as_millis());

        let file = fs::read("/tmp/ss.png").unwrap();
        let hash = sha256::digest(&file);

        let hash_end = Instant::now();
        debug!("hash time: {}", hash_end.duration_since(ss_end).as_millis());

        if hash != sha_of_file {
            info!("New hash file: {}", hash);
            sha_of_file = hash;

            // We need to convert after
            // convert /tmp/ss.png -resize 1024X758 /tmp/ss.png
            Command::new("convert")
                .arg("/tmp/ss.png")
                .arg("-quality")
                .arg("10")
                .arg("-resize")
                .arg("1024X758!")
                /*
                .arg("-dither")
                .arg("FloydSteinberg")
                .arg("-define")
                .arg("dither:diffusion-amount=90%")
                .arg("-remap")
                .arg("/tmp/eink-2color.png")
                .arg("-depth")
                .arg("1")
                */
                .arg("/tmp/ss.png")
                .output()
                .expect("Failed to convert");

            let convert_time = Instant::now();
            debug!(
                "convert time: {}",
                convert_time.duration_since(hash_end).as_millis()
            );

            let file_converted = fs::read("/tmp/ss.png").unwrap();

            send_network(
                &network_handler.clone(),
                endpointSaved,
                FromServerMessage::Screen(file_converted),
            );
        }
    }
}
