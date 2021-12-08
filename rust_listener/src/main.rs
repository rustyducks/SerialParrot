mod ld06;
mod transport;
mod ducklink;
use ld06::LD06Transport;
use ducklink::DuckLinkTransport;
use std::time::{Duration, Instant};
use std::io::{self, Write};
use std::fs::File;
use std::usize;
use byteorder::{LittleEndian, WriteBytesExt};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use clap::{Arg, App};
use transport::Transport;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("Serduck")
    .version("0.1.0")
    .about("UDP/Serial message server.")
    .arg(Arg::with_name("port")
             .short("p")
             .long("port")
             .takes_value(true)
             .required(true)
             .help("<port>"))
    .arg(Arg::with_name("baudrate")
             .short("b")
             .long("baudrate")
             .takes_value(true)
             .required(true)
             .help("230400"))
    .arg(Arg::with_name("file")
             .short("f")
             .long("file")
             .takes_value(true)
             .required(true)
             .help("<filename>"))
    .arg(Arg::with_name("transport")
             .short("t")
             .long("transport")
             .takes_value(true)
             .default_value("LD06")
             .possible_values(&["LD06", "Ducklink"])
             .help("<transport>"))
    .get_matches();

    let port = matches.value_of("port").unwrap();
    let baudrate: u32 = matches.value_of("baudrate").unwrap().parse()?;
    let filename = matches.value_of("file").unwrap();
    let transport = matches.value_of("transport").unwrap();

    let mut serial = serialport::new(port, baudrate)
        .timeout(Duration::from_millis(3))
        .open()?;

    let mut transport: Box<dyn Transport> = 
        match transport {
            "LD06" => Box::new(LD06Transport::new()),
            "Ducklink" => Box::new(DuckLinkTransport::new()),
            t => panic!("Unknown transport {}", t),
        };

    let start_time = Instant::now();
    let mut print_time = Instant::now();

    let mut file = File::create(filename)?;
    file.write_u32::<LittleEndian>(baudrate).unwrap();

    println!("Recording {} to {}", port, filename);

    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term))?;

    let mut total_size: usize = 0;

    while !term.load(Ordering::Relaxed) {
        let mut buffer: [u8; 47] = [0; 47];
        match serial.read(&mut buffer) {
            Ok(nb) => {
                for c in &buffer[0..nb] {
                    if let Some(data) = transport.put(*c) {
                        let dt = start_time.elapsed().as_secs_f64();
                        let data_len = data.len() as u16;
                        total_size += data_len as usize;

                        file.write_f64::<LittleEndian>(dt).unwrap();
                        file.write_u16::<LittleEndian>(data_len).unwrap();
                        file.write(&data[..]).unwrap();
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        };
        if print_time.elapsed().as_secs() >= 1 {
            println!("{}K", total_size/1000);
            print_time = Instant::now();
        }
    }


    Ok(())
}
