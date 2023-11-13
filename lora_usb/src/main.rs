use std::{
    io::{self, Write},
    time::Duration,
};

use serialport::SerialPort;

fn serial_write(port: &mut Box<dyn SerialPort>, command: &str) {
    let string = String::from(command) + "\r\n";
    match port.write(string.as_bytes()) {
        Ok(_) => {
            print!("\n>>{}", string);
            std::io::stdout().flush().unwrap();
        }
        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
        Err(e) => eprintln!("{:?}", e),
    }
    std::thread::sleep(Duration::from_millis((1000.0 / 9600_f32) as u64));
}

fn main() {
    println!("Hello, world!");

    let ports = serialport::available_ports().expect("No ports found!");
    for p in ports {
        println!("{}", p.port_name);
    }

    let port_name = "/dev/ttyUSB0";
    let baud_rate = 9600;
    let mut joined = false;

    let port = serialport::new(port_name, 9600)
        .timeout(Duration::from_millis(10))
        .open();

    match port {
        Ok(mut port) => {
            let mut serial_buf: Vec<u8> = vec![0; 1000];

            println!("Receiving data on {} at {} baud:", &port_name, &baud_rate);
            //serial_write(&mut port, "AT+SENDB=0,2,4,11223344");
            serial_write(&mut port, "AT+JOIN?");
            let mut string = String::new();
            loop {
                match port.read(serial_buf.as_mut_slice()) {
                    Ok(t) => {
                        string.push_str(&String::from_utf8(serial_buf[..t].to_vec()).unwrap());
                        io::stdout().write_all(&serial_buf[..t]).unwrap();

                        if !joined {
                            if string.ends_with("\r\nAT_NO_NET_JOINED") {
                                string = String::new();
                                serial_write(&mut port, "AT+JOIN");
                            }
                            if string.ends_with("\r\nOK") || string.ends_with("\r\nJOINED") {
                                joined = true;
                                string = String::new();
                                serial_write(&mut port, "AT+SENDB=5,2,4,11223344");
                            }
                        } else {
                            if string.ends_with("Run AT+RECVB=?") {
                                string = String::new();
                                serial_write(&mut port, "AT+RECVB=?");
                            }
                            if string.ends_with("\r\ntxDone") {
                                // TODO
                            }
                            if string.ends_with("\r\nrxDone") {
                                string = String::new();
                                serial_write(&mut port, "AT+SENDB=5,2,4,11223344");
                            }
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            ::std::process::exit(1);
        }
    }
}
