extern crate png;
extern crate getopts;
extern crate rand;

use std::env;
use std::fs;
use std::path;
use std::iter::FromIterator;
use std::string::String;

use getopts::{Options, Matches};

mod world;
mod image;
mod window;
mod world_builder;

use image::Image;

fn print_usage(program: &str, opts: Options) {
    let short_message = format!("Usage: {} [options] <input_file>", program);
    println!("{}", opts.usage(short_message.as_str()));
}

fn get_u32_opt(matches: &Matches, opt_name: &str) -> Option<u32> {
    match matches.opt_str(opt_name) {
        Some(string) => match string.trim().parse::<u32>() {
            Ok(value) => Some(value),
            Err(_) => panic!("Bad unsigned int arg"),
        },
        None => None,
    }
}

fn main() {
    // Parse program arguments.
    let args: Vec<String> = Vec::from_iter(env::args());
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optopt("", "smin", "minimum neighbors for existing cell to survive", "UINT");
    opts.optopt("", "smax", "maximum neighbors for existing cell to survive", "UINT");
    opts.optopt("", "rmin", "minimum neighbors for new cell to be born", "UINT");
    opts.optopt("", "rmax", "maximum neighbors for new cell to be born", "UINT");
    opts.optopt("f", "frames", "number of frames to render", "UINT");
    opts.optflag("w", "wrap", "treat image space as toroidal");
    opts.optflag("p", "proportional", "weight neighbors by how many neighbors they have");
    opts.optopt("o", "output-prefix", "write output frames to this file instead of rendering to screen", "STRING");
    opts.optflag("h", "help", "print usage information");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(program.as_str(), opts);
        return;
    }
    let input = if matches.free.len() == 1 {
        matches.free[0].clone()
    } else {
        print_usage(program.as_str(), opts);
        return;
    };
    let is_interactive = !matches.opt_present("output-prefix");

    let frames = get_u32_opt(&matches, "frames").unwrap_or(100);
    let wrap = matches.opt_present("w");
    let proportional = matches.opt_present("p");

    let smin = get_u32_opt(&matches, "smin").unwrap_or(2);
    let smax = get_u32_opt(&matches, "smax").unwrap_or(3);
    let rmin = get_u32_opt(&matches, "rmin").unwrap_or(3);
    let rmax = get_u32_opt(&matches, "rmax").unwrap_or(3);

    // Load example PNG image.
    // let file = "examples/hex_square_tri_large.png";
    // let file = "examples/cartesian_grid.png";
    // let file = "examples/hex_grid.png";
    println!("Loading '{}'.", input);
    let image = Image::load_png(&path::Path::new(&input));

    // TODO: instead of these long constructors, have a big old 'worldspec' that gets passed to most.
    let builder = world_builder::WorldBuilder::new(
        image,
        wrap,
        smin,
        smax,
        rmin,
        rmax,
        proportional,
    );
    let mut world = builder.build();

    // Either show an interactive window, or run the world for a set amount
    // of frames, writing them out to files as we go.
    if is_interactive {
        let mut win = window::Window::new(world);
        win.run();
    } else {
        // Ensure output directory exists.
        let res = fs::create_dir_all(&path::Path::new("./image_out"));
        match res {
            Err(e) => {
                panic!("Couldn't create output directory! {}", e)
            },
            _ => {},
        }

        let output_prefix = matches.opt_str("output-prefix").unwrap_or("frame_".to_string());

        for frame in 0..frames {
            world.update_world_image();

            let frame_file = format!("image_out/{}{:0>8}.png", output_prefix, frame);
            println!("Writing frame to '{}'.", frame_file);
            world.image().save_png(&path::Path::new(&frame_file));

            world.step();
        }
    }
}

