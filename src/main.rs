#![allow(while_true)]

extern crate serialport;
#[macro_use]
extern crate clap;

extern crate nmea;

use std::time::Duration;

use serialport::*;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::string::String;

use nmea::Nmea;

fn print_typename<T>(_: T) {
    println!("{}", std::any::type_name::<T>());
}

fn get_tty_on_mac() -> String {
    let entries = fs::read_dir("/dev").unwrap();
    let mut ret1: String = String::from("");
    let mut ret2: String = String::from("");

    for f in entries {
        let s = f.unwrap().path().display().to_string();
        if s.contains("cu.SLAB") {
            ret1 = s;
        } else if s.contains("cu.usbserial") {
            ret2 = s;
        }
    }
    if ret1 != "" {
        return ret1;
    }
    ret2
}

fn display_nema_content( s: &nmea::SentenceType, n: &nmea::Nmea) {
    match s {
        nmea::SentenceType::GGA => {
        },
        nmea::SentenceType::GSA => {

        },
        nmea::SentenceType::RMC =>{
            println!("Time {}", n.fix_time.unwrap());
            println!("Date {}", n.fix_date.unwrap());
            if n.fix_type.as_ref().unwrap() != &nmea::FixType::Invalid {
                println!("Position {:.5}, {:.5}", n.latitude.unwrap(), n.longitude.unwrap());
                println!("Speed {}", n.speed_over_ground.unwrap());
                println!("Direction {}", n.true_course.unwrap());
            }
        },
        nmea::SentenceType::VTG =>{

        },
        _ => {},
    }
}

fn main() {
    let matches = clap_app!(myapp =>
        (version: "1.0")
        (about: "GPS reader")
        (@arg port_name: --port -p +takes_value  "serial port")
        (@arg baudrate: --baud -b +takes_value "baudrate(default 9600)")
    )
    .get_matches();

    let d;
    let default_port = if cfg!(target_os = "windows") {
        "COM0"
    } else if cfg!(target_os = "linux") {
        "/dev/ttyUSB0"
    } else if cfg!(target_os = "macos") {
        d = Some(get_tty_on_mac());
        d.as_ref().unwrap()
    } else {
        "/dev/ttyUSB0"  // Linux 
    };

    let port_name = matches.value_of("port_name").unwrap_or(default_port);
    let baudrate = matches.value_of("baudrate").unwrap_or("9600");

    println!("port {}, baudrate {}", port_name, baudrate);

    let settings = serialport::SerialPortSettings {
        baud_rate: baudrate.parse().unwrap(),
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(1000),
    };

    let port = match serialport::open_with_settings(port_name, &settings) {
        Ok(p) => p,
        Err(_) => panic!("Error"),
    };

    let mut reader = BufReader::new(port);
    let mut line = String::new();
    print_typename(Nmea::new());

    let mut nmea = Nmea::new();

    // main loop
    loop {
        match reader.read_line(&mut line) {
            Ok(bytes) => {
                if bytes >= 1 {
                    //println!("[{:?}]", line);

                    match nmea.parse(&line) {
                        Ok(s) => {
                            //println!("{:?}{:?}", s, nmea);
                            display_nema_content(&s, &nmea);
                        },
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        }
        line.clear();
    }
}
