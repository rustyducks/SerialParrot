mod ld06;
use ld06::LD06Transport;
use std::time::{Duration, Instant};
use std::io::{self, Write};
use std::fs::File;
use byteorder::{LittleEndian, WriteBytesExt};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use clap::{Arg, App};


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
    .get_matches();

    let port = matches.value_of("port").unwrap();
    let baudrate: u32 = matches.value_of("baudrate").unwrap().parse()?;
    let filename = matches.value_of("file").unwrap();

    let mut serial = serialport::new(port, baudrate)
        .timeout(Duration::from_millis(3))
        .open()?;

    let mut transport = LD06Transport::new();

    let start_time = Instant::now();

    let mut file = File::create(filename)?;
    file.write_u32::<LittleEndian>(baudrate).unwrap();

    println!("Recording {} to {}", port, filename);

    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term))?;
    while !term.load(Ordering::Relaxed) {
        let mut buffer: [u8; 47] = [0; 47];
        match serial.read(&mut buffer) {
            Ok(nb) => {
                for c in &buffer[0..nb] {
                    if let Some(data) = transport.put(*c) {
                        println!("got data: {:?}", data);

                        let dt = start_time.elapsed().as_secs_f64();
                        let data_len = data.len() as u16;

                        file.write_f64::<LittleEndian>(dt).unwrap();
                        file.write_u16::<LittleEndian>(data_len).unwrap();
                        file.write(&data[..]).unwrap();
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        };
    }


    Ok(())
}
