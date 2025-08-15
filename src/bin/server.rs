use std::{
    io::Read,
    net::{SocketAddr, TcpListener, TcpStream},
    thread,
};

fn main() {
    let listener = TcpListener::bind("172.30.1.7:10000").unwrap();
    loop {
        let (stream, address) = listener.accept().unwrap();
        thread::spawn(move || handle_connection(stream, address));
    }
}

fn handle_connection(mut stream: TcpStream, address: SocketAddr) {
    println!("Connected from {address:?}");
    loop {
        let mut length = [0_u8; 4];
        let n = stream.read(&mut length).unwrap();
        if n == 0 {
            println!("{address:?} disconnected..");
            break;
        }
        let length = u32::from_be_bytes(length);

        let mut buffer = vec![0_u8; length as usize];
        stream.read_exact(&mut buffer).unwrap();
        let message = String::from_utf8_lossy(&buffer);
        println!("{message:?}");
    }
}
