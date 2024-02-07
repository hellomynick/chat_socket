use core::time;
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Read, Write};
use std::thread;
use std::{net::TcpListener, sync::mpsc};

const MSG_SIZE: usize = 32;

fn sleep() {
    thread::sleep(time::Duration::from_millis(100));
}

fn main() {
    let listeners = TcpListener::bind("127.0.0.1:8080").expect("Listener failed to bind");
    listeners
        .set_nonblocking(true)
        .expect("failed initial nonblocking");

    let (sender, receiver) = mpsc::channel::<String>();
    let mut clients = vec![];

    for stream in listeners.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("Client connected: {:?}", stream);
                let sender = sender.clone();

                clients.push(stream.try_clone().unwrap());

                thread::spawn(move || loop {
                    let mut buffer = vec![0; MSG_SIZE];

                    match stream.read_exact(&mut buffer) {
                        Ok(_) => {
                            let msg = buffer.into_iter().take_while(|&x| x != 0).collect();
                            let msg = String::from_utf8(msg).expect("Cannot convert to string");
                            println!("{}", msg);
                            sender.send(msg).expect("Can not send to channel");
                        }
                        Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                        Err(_) => {
                            println!(
                                "Unexpected error closing connection with: {:?}",
                                stream.try_clone().unwrap()
                            );
                            break;
                        }
                    }
                    sleep();
                });
            }

            Err(ref e) if e.kind() == ErrorKind::WouldBlock => (),
            Err(err) => {
                eprintln!("Error accepting stream {}", err);
            }
        }

        match receiver.try_recv() {
            Ok(msg) => {
                println!("{}", msg);
                clients = clients
                    .into_iter()
                    .filter_map(|mut client| {
                        let mut buff = msg.clone().into_bytes();

                        buff.resize(MSG_SIZE, 0);
                        client.write_all(&buff).map(|_| client).ok()
                    })
                    .collect::<Vec<_>>();
            }
            Err(mpsc::TryRecvError::Empty) => (),
            Err(mpsc::TryRecvError::Disconnected) => {
                println!("Sender disconnected");
            }
        }
        sleep();
    }
}
