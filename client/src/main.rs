use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;

fn main() {
    // connectin to local host
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");

    // nonblocking for multithreading
    client
        .set_nonblocking(true)
        .expect("failed to initiate non-blocking");

    // creating a channel for sending data
    let (tx, rx) = mpsc::channel::<String>();

    // spwaning thread
    thread::spawn(move || loop {
        // creating buffer msg
        let mut buff = vec![0; MSG_SIZE];

        // to read msg though the buffer
        match client.read_exact(&mut buff) {
            // if read successfull
            Ok(_) => {
                // remove all 0's from the buffer and collect it into a vector
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();

                // print that we recevied this msg
                println!("message recv {:?}", msg);
            }

            // if error is WouldBlock(block our thread) return unit tyoe
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),

            // if any other error break the loop and say "connection with server was severed"
            Err(_) => {
                println!("connection with server was severed");
                break;
            }
        }

        // checking if server sends back the confirmation that it got the msg from client
        match rx.try_recv() {
            // yes it did case
            Ok(msg) => {
                // clone msg into our buffer after converting it into bytes
                let mut buff = msg.clone().into_bytes();

                buff.resize(MSG_SIZE, 0);

                // write buffer to client
                client.write_all(&buff).expect("writing to socket failed");
                // print that we sent this msg
                println!("message sent {:?}", msg);
            }
            // try_recv comes empty
            Err(TryRecvError::Empty) => (),
            // try_recv comes disconnected then break
            Err(TryRecvError::Disconnected) => break,
        }
        thread::sleep(Duration::from_millis(100));
    });

    println!("write a msg: ");

    // loop to read msgs from client
    loop {
        let mut buff = String::new();

        // read from stdin into buffer
        io::stdin()
            .read_line(&mut buff)
            .expect("reading from stdin failed");

        // trim the buffer and convert it into string
        let msg = buff.trim().to_string();

        // if msg is :quit then break or on error break
        if msg == ":quit" || tx.send(msg).is_err() {
            break;
        }
    }

    println!("bye bye")
}
