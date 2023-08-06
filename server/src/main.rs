use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

const LOCAL: &str = "127.0.0.1:6000"; // localhost with port
const MSG_SIZE: usize = 32; // buffer size of msgs

// so that loop rest when not receving msgs
fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

fn main() {
    // initialize server with localhost
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind"); // if it fails, panics with this msg

    // push server to non-blocking mode

    /* "non-blocking mode" => handling multiple client connections, can be performed without causing the server to wait for a single operation to complete before moving on to the next one.
     */

    server
        .set_nonblocking(true)
        .expect("failed to initialize non-blocking"); // let server constantly check for msgs

    let mut clients = vec![]; // to have multiple clients

    // instantiate a channel to send strings(msgs) through it
    let (tx, rx) = mpsc::channel::<String>();

    loop {
        // server.accept allows to accept connections from clients, returns a tuple of (socket, address)
        // socket => actual tcp-stream thats connecting
        // address => address of socket
        if let Ok((mut socket, addr)) = server.accept() {
            // its OK
            println!("Client {} connected", addr);

            let tx = tx.clone();

            // we push the socket to clients vector if Ok(connection is ok) else "failed to clone client"
            clients.push(socket.try_clone().expect("failed to clone client"));

            // we are cloning stuffs(tx, socket) so we can use it in multiple threads

            // spawn a new thread for each client
            thread::spawn(move || loop {
                // buffer for msgs with 0s inside of it and of MSG_SIZE
                let mut buff = vec![0; MSG_SIZE];

                // this reads msg into our buffer
                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        // take the msg from buffer put it into a iterator take all the character which are not white space and collect it into a vector
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();

                        // convert the vector into a string
                        let msg = String::from_utf8(msg).expect("invalid utf8 message");

                        // address that sent msg
                        println!("{}: {:?}", addr, msg);

                        // sending msg to rx through tx
                        tx.send(msg).expect("failed to send msg to rx");
                    }

                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (), // if would block, continue by sending unit type

                    Err(_) => {
                        println!("closing connection with: {}", addr);
                        break;
                    }
                }

                sleep(); // to let thread sleep for 100ms between each loop
            });
        }

        // if rx has a msg => we try to receive it through channel
        if let Ok(msg) = rx.try_recv() {
            clients = clients
                .into_iter()
                .filter_map(|mut client| {
                    //  convert msg into bytes
                    let mut buff = msg.clone().into_bytes();

                    // resize buffer to MSG_SIZE
                    buff.resize(MSG_SIZE, 0);

                    // write entire buffer than map it to client
                    client.write(&buff).map(|_| client).ok()
                })
                .collect::<Vec<_>>(); // collect all clients into a vector
        }
        sleep();
    }
}
