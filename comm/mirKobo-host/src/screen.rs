use std::{
    io::Read,
    os::raw::c_void,
    process::exit,
    thread,
    time::{Duration, Instant},
};

// Unfinished xclip prototype - it's slow on wayland
/*
    let mut monitor: Option<Monitor> = None;
    for monitor_i in Monitor::all().unwrap() {
        if monitor_i.name() == args.monitor {
            monitor = Some(monitor_i);
        }
    }
    if monitor.is_none() {
        error!("Monitor not found, use the monitor argument correctly");
        exit(1);
    } else {
        debug!("Using monitor: {}", monitor.clone().unwrap().name());
    }
    let mut w = -1; //= args.w;
    let mut h = -1; //= args.h;
    if w == -1 {
        w = monitor.clone().unwrap().width() as i32;
    }
    if h == -1 {
        h = monitor.clone().unwrap().height() as i32;
    }
    info!("Using size: {}x{}", w, h);

    let mut image_saved: Option<ImageBuffer<Rgba<u8>, Vec<u8>>> = None;
    let mut wand = MagickWand::new();
    loop {
        //debug!("Taking ss");
        let ss_time = Instant::now();
        // pacman -S libxcb libxrandr dbus
        let mut image = None;
        if let Ok(image_new) = monitor.clone().unwrap().capture_image() {
            image = Some(image_new);
            debug!("Captured image");
        } else {
            error!("Failed to capture image");
            exit(1);
        }
        let ss_end = Instant::now();
        debug!("ss time: {}", ss_end.duration_since(ss_time).as_millis());
        if image_saved.is_none() || image_saved.clone().unwrap() != image.clone().unwrap() {
            debug!("Sending image");

            wand.new_image(w as usize, h as usize, &PixelWand::new())
                .unwrap();
            wand.import_image_pixels(
                0,
                0,
                w as usize,
                h as usize,
                image.clone().unwrap().as_bytes(),
                "RGBA",
            )
            .expect("Failed to put image into imagemagick");
            wand.resize_image(
                args.kobo_w,
                args.kobo_h,
                magick_rust::bindings::FilterType_PointFilter,
            );
            // https://imagemagick.org/script/command-line-options.php
            // Not sure about measure error?
            wand.quantize_image(
                args.number_of_colors,
                magick_rust::bindings::ColorspaceType_RGBColorspace,
                0,
                magick_rust::bindings::DitherMethod_FloydSteinbergDitherMethod,
                0,
            )
            .expect("Failed to quantize_image");

            // Debugging
            wand.write_image("/tmp/mirKobo.png").unwrap();

            /*
                send_network(
                &network_handler.clone(),
                endpointSaved,
                FromServerMessage::Screen(file_converted),
            );
            */
            image_saved = image;
        }
    }
*/
// Logging
use log::{debug, error, info, warn};

// Arguments
use crate::Args;

// ss