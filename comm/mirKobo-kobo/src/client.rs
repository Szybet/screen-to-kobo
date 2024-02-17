#![deny(clippy::useless_attribute)]
#![allow(clippy::single_match)]

// Logging
use log::{debug, error, info};

// Network
use crate::api::{FromClientMessage, FromServerMessage};
use message_io::network::{NetEvent, RemoteAddr, Transport};
use message_io::node::{self, NodeEvent};

// Device
use crate::device::{click, get_screen, get_screen_size};

// Other
use crate::Args;
use std::sync::{mpsc, Arc};
use std::time::Duration;
use std::{fs, thread};

use fbink_sys::*;
use std::{ffi::CString, process::exit};

enum LooseJobs {
    SendScreen(Vec<u8>),
    Stop,
}

use std::io::prelude::*;
use std::os::unix::net::UnixStream;

pub fn run(transport: Transport, remote_addr: RemoteAddr, args: &Args) {
    let (handler_regular, listener) = node::split();
    let handler = Arc::new(handler_regular);

    let (server_id, local_addr) = handler
        .network()
        .connect(transport, remote_addr.clone())
        .unwrap();

    /*

    let fbfd: ::std::os::raw::c_int = unsafe { fbink_open() };
    if fbfd < 0 {
        println!("Failed to open fbink");
        exit(1);
    }

    let mut fbink_cfg: FBInkConfig =
        unsafe { std::mem::transmute([0u8; std::mem::size_of::<FBInkConfig>()]) };
    println!("fbink_cfg default state: {:?}", fbink_cfg);
    fbink_cfg.is_flashing = false;

    unsafe {
        if fbink_init(fbfd, &fbink_cfg) < 0 {
            println!("Failed to init fbink");
            exit(1);
        }
    }

    let x = get_screen_size("busybox");
    let w = x.0 as i32;
    let h = x.1 as i32;
    info!("wxh: {}x{}", w, h);
    */
    /*
    unsafe {
        fbink_print_raw_data(fbfd, ss.as_mut_ptr(), w, h, ss.len(), 0, 0, &fbink_cfg);
        fbink_wait_for_complete(fbfd, LAST_MARKER);
    }
    c*/
    let (tx_to_loose, rx_to_loose) = mpsc::sync_channel(1); // We want synced channel because of try_send
    thread::spawn(move || loop {
        if let Ok(event) = rx_to_loose.recv() {
            match event {
                LooseJobs::SendScreen(ss) => {
                    info!("Sending screen over unix socket, length: {}", ss.len());
                    let mut stream = UnixStream::connect("/kobo/tmp/screenRGB").unwrap();
                    stream.write_all(&ss).unwrap();
                    stream.flush().unwrap();
                }
                LooseJobs::Stop => {
                    break;
                }
            }
        }
    });

    listener.for_each(move |event| match event {
        NodeEvent::Network(net_event) => match net_event {
            NetEvent::Connected(_, established) => {
                if established {
                    info!(
                        "Connected to server at {} by {}",
                        server_id.addr(),
                        transport
                    );
                    info!("Client identified by local port: {}", local_addr.port());
                    handler.signals().send(FromClientMessage::Ping);
                } else {
                    info!(
                        "Cannot connect to server at {} by {}",
                        remote_addr, transport
                    );
                    info!("Retrying in 3 seconds...");
                    tx_to_loose.send(LooseJobs::Stop).unwrap();
                    thread::sleep(Duration::from_secs(3));
                    handler.stop();
                }
            }
            NetEvent::Accepted(_, _) => unreachable!(), // Only generated when a listener accepts
            NetEvent::Message(_, input_data) => {
                debug!("Received raw input data with length: {}", input_data.len());
                let message: FromServerMessage = bincode::deserialize(input_data).unwrap();
                match message {
                    FromServerMessage::Pong => {
                        info!("Received Pong from server, awesome");
                    }
                    FromServerMessage::Screen(ss) => {
                        if tx_to_loose.try_send(LooseJobs::SendScreen(ss)).is_err() {
                            info!("Failed to send screen further");
                        }
                    }
                }
            }
            NetEvent::Disconnected(_) => {
                info!("Server is disconnected");
                info!("Retrying in 3 seconds...");
                tx_to_loose.send(LooseJobs::Stop).unwrap();
                thread::sleep(Duration::from_secs(3));
                handler.stop();
            }
        },
        NodeEvent::Signal(signal) => match signal {
            FromClientMessage::Ping => {
                info!("Sending Ping");
                let message = FromClientMessage::Ping;
                let output_data = bincode::serialize(&message).unwrap();
                handler.network().send(server_id, &output_data);
            }
            _ => {}
        },
    });
}
