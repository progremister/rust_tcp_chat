use std::{
    io::{self, ErrorKind, Write, Read},
    net::TcpStream,
    sync::mpsc::{self, TryRecvError},
    time::Duration,
    thread
};

const LOCAL: &str = "127.0.0.1:5000";
const MAX_MSG_SIZE: usize = 50;

fn main() {
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect!"); 
    client.set_nonblocking(true).expect("Failed to initiate non-blocking!");

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buff = vec![0; MAX_MSG_SIZE];

        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                println!("Message: {:?}", msg);
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Connection with server was severed!");
                break;
            }
        }

        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MAX_MSG_SIZE, 0);
                client.write_all(&buff).expect("Writing to socket faiiled!");
                println!("Message sent {:?}", msg);
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }

        thread::sleep(Duration::from_millis(100));
    }); 

    println!("Write a message:");
    loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("Reading from stdin failed");
        let msg = buff.trim().to_string();
        if msg == ":quit" || tx.send(msg).is_err() {break}
    }

    println!("See you later!")
}

