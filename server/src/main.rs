use std::{
    io::{ErrorKind, Read, Write },
    net::TcpListener,
    sync::mpsc,
    thread
};

const LOCAL: &str = "127.0.0.1:5000";
const MSG_MAX_SIZE: usize = 50;

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

fn main() {
    let server = TcpListener::bind(LOCAL).expect("Listener ailed to bind to the adress!");
    server.set_nonblocking(true).expect("Failed to initiate the non-blocking!");

    let mut clients = vec![];
    let (tx, rx) = mpsc::channel::<String>();

    loop {
        if let Ok((mut socket, address)) = server.accept() {
            println!("Client {} connected.", address);

            let tx = tx.clone();
            clients.push(socket.try_clone().expect("Failed to clone client!"));

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_MAX_SIZE];

                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message!");

                        println!("{} {:?}", address, msg);
                        tx.send(msg).expect("Failed to send a message to the receiver!");
                    },
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Closing connection with: {}", address);
                        break;
                    }
                }

                sleep();
            });
        }

        if let Ok(msg) = rx.try_recv() {
            clients = clients.into_iter().filter_map(|mut client| {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_MAX_SIZE, 0);

                client.write_all(&buff).map(|_| client).ok()

            }).collect::<Vec<_>>();
        }

        sleep();
    }
}
