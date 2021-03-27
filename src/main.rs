extern crate clap;
extern crate serial;

use clap::{Arg, App};
use show_image::{ImageView, ImageInfo, create_window};
use std::io;
use std::time::Duration;
use serial::prelude::*;

const C_DEFAULT_SYNC_WORD: &'static str = "XXSYNCXX";
const C_DEFAULT_IMAGE_WIDTH: &'static str = "96";
const C_DEFAULT_IMAGE_HEIGHT: &'static str = "96";
const C_DEFAULT_DEVICE: &'static str = "/dev/ttyACM0";

#[show_image::main]
fn main() {
    let matches = App::new("serial2screen")
        .version("0.0.1")
        .author("Piotr Goslawski <piotr@4point2.nl>")
        .about("Display ")
        .arg(Arg::new("dev").short('d').long("device").takes_value(true).about("serial device"))
        .arg(Arg::new("sync").short('s').long("sync").takes_value(true).about("sync word"))
        .arg(Arg::new("width").short('w').long("width").takes_value(true).about("image width"))
        .arg(Arg::new("height").short('h').long("height").takes_value(true).about("image height"))
        .get_matches();

    let dev = matches.value_of("dev").unwrap_or(C_DEFAULT_DEVICE);
    let sync_word = matches.value_of("sync").unwrap_or(C_DEFAULT_SYNC_WORD);
    let img_width = matches.value_of("width").unwrap_or(C_DEFAULT_IMAGE_WIDTH).parse::<u32>().expect("Invalid width value");
    let img_height = matches.value_of("height").unwrap_or(C_DEFAULT_IMAGE_HEIGHT).parse::<u32>().expect("Invalid height value");
    println!("Reading from: {}", dev);

    let mut port = serial::open(dev)
        .expect(&format!("Cannot open the serial device: {}", dev));

    configure_port(&mut port)
        .expect("Error while setting port config");

    let window = create_window("image", Default::default())
        .expect("Cannot create the window");

    loop {

        sync_data(&mut port, sync_word)
            .expect("Error synchronizing");

        let data = read_image_data(&mut port, img_width, img_height)
            .expect("Error reading serial data");

        if !data.is_empty() {
            let image = ImageView::new(ImageInfo::mono8(img_width, img_height), &data);

            // Create a window with default options and display the image.
            window.set_image("image", image)
                .expect("Error setting the image")
        }
    }
}

fn read_image_data<T: SerialPort>(port: &mut T, img_width : u32, img_height : u32) -> io::Result<Vec<u8>> {

    let mut buf: Vec<u8> = vec![0; (img_width * img_height) as usize];

    port.read_exact(&mut buf)?;
    Ok(buf)
}

fn sync_data<T: SerialPort>(port: &mut T, sync_word: &str) -> io::Result<()> {
    let sync_word_u8 = sync_word.as_bytes().to_vec();
    let mut sync_pos = 0;
    let mut char_buffer = [0];
    let mut str = String::new();

    while sync_pos < sync_word_u8.len() {
        port.read_exact(&mut char_buffer)?;
        let sync_char = char_buffer[0];
        if sync_char == sync_word_u8[sync_pos] {
            sync_pos = sync_pos + 1;
        } else {
            str.push(char::from(sync_char));
            sync_pos = 0;
        }
    }
    if !str.is_empty() {
        println!("{}", str);
    }
    Ok(())
}

fn configure_port<T: SerialPort> (port: &mut T) -> io::Result<()> {
    (port.reconfigure(&|settings| {
        settings.set_baud_rate(serial::Baud115200)?;
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    }))?;

    port.set_timeout(Duration::from_millis(1000))?;
    Ok(())
}


