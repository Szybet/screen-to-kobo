mod api;
mod server;
mod screen;

// Logging
use log::{debug, error, info, warn};

// Network
pub use api::{FromClientMessage, FromServerMessage};
use magick_rust::bindings::ThresholdMap;
use message_io::network::{Endpoint, ResourceId, SendStatus, Transport};
use message_io::node::{self, NodeHandler};
use scrap::{is_x11, Display};
use std::any::Any;
use std::net::ToSocketAddrs;

// Threads
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::{fs, thread, time};

// Arguments
use clap::{error, Parser};

// Other
use rand::Rng;
use std::process::{exit, Command};
use std::time::{Duration, Instant};

// Screen
use std::sync::Once;
use libwayshot::WayshotConnection;
use magick_rust::{magick_wand_genesis, MagickWand, PixelWand};

use scrap::{Capturer, Frame, TraitCapturer, TraitPixelBuffer};

// Else
use std::io::ErrorKind::WouldBlock;

pub fn is_wayland() -> bool {
    if WayshotConnection::new().is_err() {
        if is_x11() {
            return false;
        }
    }
    return true;
}

pub fn list_monitors() -> String {
    let mut str = String::from("Monitors: \n");
    for monitor in Display::all().unwrap() {
        match monitor {
            Display::X11(d) => {
                str.push_str(&format!("{}x{}\n", d.width(), d.height()));
            }
            Display::WAYLAND(d) => {
                str.push_str(&format!("{}x{}\n", d.width(), d.height()));
            }
        }
    }
    str.pop();
    str
}

pub fn first_monitor() -> String {
    match Display::all().unwrap().first().unwrap() {
        Display::X11(d) => return format!("{}x{}", d.width(), d.height()),
        Display::WAYLAND(d) => return format!("{}x{}", d.width(), d.height()),
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    #[arg(short, long, help = "Network port to use", default_value_t = 24356)]
    port: u16,
    /*
    #[arg(short, help = "X coordinate", default_value_t = 0)]
    x: u32,
    #[arg(short, help = "Y coordinate", default_value_t = 0)]
    y: u32,
    #[arg(
        short,
        help = "Width, -1 means the whole selected monitor",
        default_value_t = -1,
    )]
    w: i32,
    #[arg(
        short,
        help = "Height, -1 means the whole selected monitor",
        default_value_t = -1,
    )]
    h: i32,
    */
    #[arg(short, long, help = "Kobo screen height", default_value_t = 1024)]
    kobo_w: usize,
    #[arg(short, long, help = "Kobo screen Width", default_value_t = 758)]
    kobo_h: usize,
    #[arg(
        short,
        long,
        help = list_monitors(),
        default_value_t = first_monitor(),
    )]
    monitor: String,
    #[arg(
        short,
        help = "Number of colors - ImageMagick setting",
        default_value_t = 2
    )]
    number_of_colors: usize,
    #[arg(
        short,
        help = "Tree depth - ImageMagick setting",
        default_value_t = 4
    )]
    tree_depth: usize,
}

static START: Once = Once::new();

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
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "none,mir_kobo_host=debug"),
    );
    debug!("Starting mirKobo-host");

    let args = Args::parse();

    debug!("Calling magick");
    START.call_once(|| {
        magick_wand_genesis();
    });

    if is_wayland() == false {
        warn!("We are not running on wayland! It may work - maybe");
    } else {
        info!("Running on wayland");
    }

    if is_wayland() == false {
        error!("You need to be using wayland. I'm lazy.");
    }

    let mut endpointSaved: Option<Endpoint> = None;

    // Threads
    let (tx_to_gui, rx_to_gui) = mpsc::channel();

    // Network
    let addr = ("0.0.0.0", args.port)
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

    // This will wait untill connection
    if let Ok(event) = rx_to_gui.recv() {
        match event {
            ThreadCom::ClientConnected(endpoint, _resource_id) => {
                info!("Server received: ClientConnected");
                endpointSaved = Some(endpoint);
            }
        }
    }

    debug!("Running wayland_screen");
    let mut monitor: Option<Display> = None;
    for monitor_i in Display::all().unwrap() {
        if format!("{}x{}", monitor_i.width(), monitor_i.height()) == args.monitor {
            monitor = Some(monitor_i);
        }
    }
    if monitor.is_none() {
        error!("Monitor not found, use the monitor argument correctly");
        exit(1);
    }
    //debug!("Using monitor: {}", monitor.unwrap().name());

    let mut capturer = Capturer::new(monitor.unwrap()).expect("Couldn't begin capture.");
    let (w, h) = (capturer.width(), capturer.height());
    info!("Using size: {}x{}", w, h);
    info!("Using kobo size: {}x{}", args.kobo_w, args.kobo_h);

    let one_second = Duration::new(1, 0);
    let one_frame = one_second / 60;

    let mut image_saved: Option<Vec<u8>> = None;
    loop {
        let mut wand = MagickWand::new();
        //debug!("Taking ss");
        let ss_time = Instant::now();
        let mut image: Option<Vec<u8>> = None;

        loop {
            // Wait until there's a frame.
            let frame = match capturer.frame(Duration::from_millis(0)) {
                Ok(frame) => frame,
                Err(error) => {
                    if error.kind() == WouldBlock {
                        // Keep spinning.
                        thread::sleep(one_frame);
                        continue;
                    } else {
                        panic!("Error: {}", error);
                    }
                }
            };
            let Frame::PixelBuffer(frame) = frame else {
                return;
            };
            let buffer = frame.data();
            println!("Captured data len: {}, Saving...", buffer.len());

            // Flip the BGRA image into a RGBA image.

            let mut bitflipped = Vec::with_capacity(w * h * 4);
            let stride = buffer.len() / h;

            for y in 0..h {
                for x in 0..w {
                    let i = stride * y + 4 * x;
                    bitflipped.extend_from_slice(&[buffer[i + 2], buffer[i + 1], buffer[i], 255]);
                }
            }

            image = Some(bitflipped);

            // Save the image.
            /*
            let name = format!("screenshot{}_1.png", i);
            repng::encode(
                File::create(name.clone()).unwrap(),
                w as u32,
                h as u32,
                &bitflipped,
            )
            .unwrap();
            */

            //println!("Image saved to `{}`.", name);
            break;
        }

        let ss_end = Instant::now();
        debug!("ss time: {}", ss_end.duration_since(ss_time).as_millis());
        //if image_saved.is_none() || image_saved.clone().unwrap() != image.clone().unwrap() {
            wand.new_image(w as usize, h as usize, &PixelWand::new())
                .unwrap();
            if wand.import_image_pixels(
                0,
                0,
                w as usize,
                h as usize,
                &image.clone().unwrap(),
                "RGBA",
            ).is_err() {
                error!("Can't fill image wand");
                image_saved = image;
                continue;
            }
            wand.resize_image(
                args.kobo_w,
                args.kobo_h,
                magick_rust::bindings::FilterType_PointFilter,
            );
            // https://imagemagick.org/script/command-line-options.php
            // Not sure about measure error?
            // DitherMethod_FloydSteinbergDitherMethod
            // DitherMethod_RiemersmaDitherMethod
            wand.quantize_image(
                args.number_of_colors,
                magick_rust::bindings::ColorspaceType_GRAYColorspace,
                args.tree_depth,
                magick_rust::bindings::DitherMethod_FloydSteinbergDitherMethod,
                0,
            )
            .expect("Failed to quantize_image");

            if args.number_of_colors < 2 {
                // Make it actually monochrome
                wand.set_image_compose(magick_rust::bindings::CompositeOperator_ThresholdCompositeOp).unwrap();
            }

            /*
            let byt: Result<Vec<_>, _> = wand.write_image_blob("gray").unwrap().bytes().collect();
            let byt_c = byt as *const c_void;
            let mut info_empty: ImageInfo;
            unsafe { GetImageInfo(&mut info_empty) };
            //let img = unsafe { BlobToImage(&info_empty, byt as *const c_void, by) };
             */
            //wand.write_image_blob(format)

            // Debugging
            //wand.write_image("/tmp/mirKobo.png").unwrap();
            debug!("Sending image");

            send_network(
            &network_handler.clone(),
            endpointSaved,
            FromServerMessage::Screen(wand.write_image_blob("png").unwrap()));
            //image_saved = image;
        //}
    }
}
