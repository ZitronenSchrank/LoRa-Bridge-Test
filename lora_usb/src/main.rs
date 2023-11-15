pub mod uplink_message;

use std::{
    io::{self, Write},
    process,
    sync::mpsc::{channel, Sender},
    thread,
    time::Duration,
};

use mqtt::{Client, Message, Receiver};
use paho_mqtt as mqtt;

use serialport::SerialPort;
use uplink_message::UplinkMessage;

fn mqtt_connect(host: &str) -> (Client, Receiver<Option<Message>>) {
    // Create the client. Use an ID for a persistent session.
    // A real system should try harder to use a unique ID.
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id("rust_sync_consumer")
        .finalize();

    let cli = mqtt::Client::new(create_opts).unwrap_or_else(|e| {
        println!("Error creating the client: {:?}", e);
        process::exit(1);
    });

    // Initialize the consumer before connecting
    let rx = cli.start_consuming();

    // Define the set of options for the connection
    let lwt = mqtt::MessageBuilder::new()
        .topic("test")
        .payload("Sync consumer lost connection")
        .finalize();

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
        .will_message(lwt)
        .finalize();

    let subscriptions = ["application/+/device/70b3d5581000003d/event/up"];
    let qos = [0];

    // Make the connection to the broker
    match cli.connect(conn_opts) {
        Ok(rsp) => {
            if let Some(conn_rsp) = rsp.connect_response() {
                println!(
                    "Connected to: '{}' with MQTT version {}",
                    conn_rsp.server_uri, conn_rsp.mqtt_version
                );
                if conn_rsp.session_present {
                    // Since our persistent session is already on the broker
                    // we don't need to subscribe to the topics.
                    println!("  w/ client session already present on broker.");
                } else {
                    // The server doesn't have a persistent session already
                    // stored for us (1st connection?), so we need to subscribe
                    // to the topics we want to receive.
                    println!(
                        "Subscribing to topics {:?} with requested QoS: {:?}...",
                        subscriptions, qos
                    );

                    cli.subscribe_many(&subscriptions, &qos)
                        .and_then(|rsp| {
                            rsp.subscribe_many_response()
                                .ok_or(mqtt::Error::General("Bad response"))
                        })
                        .map(|vqos| {
                            println!("QoS granted: {:?}", vqos);
                        })
                        .unwrap_or_else(|err| {
                            println!("Error subscribing to topics: {:?}", err);
                            cli.disconnect(None).unwrap();
                            process::exit(1);
                        });
                }
            }
        }
        Err(e) => {
            println!("Error connecting to the broker: {:?}", e);
            process::exit(1);
        }
    }

    // ^C handler will stop the consumer, breaking us out of the loop, below
    let ctrlc_cli = cli.clone();
    ctrlc::set_handler(move || {
        ctrlc_cli.stop_consuming();
    })
    .expect("Error setting Ctrl-C handler");

    (cli, rx)
}

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

fn start_mqtt_thread(
    sender: Sender<UplinkMessage>,
    rx: Receiver<Option<Message>>,
) -> std::thread::JoinHandle<()> {
    thread::spawn(move || {
        for msg in rx.iter().flatten() {
            let payload = msg.payload_str().to_string();
            let payload: UplinkMessage = serde_json::from_str(&payload).unwrap();
            sender.send(payload).unwrap();
        }
    })
}

fn main() {
    println!("Hello, world!");

    let port_name = "/dev/ttyUSB0";
    let baud_rate = 9600;
    let mut joined = false;

    let port = serialport::new(port_name, 9600)
        .timeout(Duration::from_millis(10))
        .open();

    let (_, rx) = mqtt_connect("lorastuff:1883");
    let (sender, receiver) = channel::<UplinkMessage>();

    start_mqtt_thread(sender, rx);

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

                        if let Ok(msg) = receiver.try_recv() {
                            println!("{:?}", msg);
                        }

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
                    Err(e) => {
                        eprintln!("{:?}", e);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            ::std::process::exit(1);
        }
    }
}
