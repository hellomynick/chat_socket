use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::thread;
use std::{net::TcpListener, sync::mpsc};

struct AppState {
    client_connection: u32,
}

impl AppState {
    fn increase_client(&self) -> u32 {
        self.client_connection + 1
    }
}

fn main() {
    let app_state = AppState {
        client_connection: 0,
    };

    let listeners = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listeners.incoming() {
        println!("Client connected: {:?}", stream);
        app_state.increase_client();
        let (tx, rx) = mpsc::channel::<String>();

        match stream {
            Ok(mut stream) => {
                let stream_reader = stream.try_clone().unwrap();
                thread::spawn(move || {
                    let mut reader = BufReader::new(stream_reader);
                    let mut buffer = String::new();

                    while reader.read_line(&mut buffer).unwrap() > 0 {
                        println!("{}", buffer.trim());
                        tx.send(buffer.clone()).expect("Can not send to channel");
                    }
                });

                for mess in rx {
                    let mut writer = BufWriter::new(&stream);
                    writer.write_all(mess.as_bytes());
                }
            }
            Err(err) => {
                eprintln!("Error accepting stream {}", err);
            }
        }
    }
}
