use std::{
    io::{Read, Write},
    net::TcpStream,
    thread,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sender = TcpStream::connect("localhost:10000")?;
    let mut receiver = sender.try_clone()?;

    thread::spawn(move || {
        loop {
            let mut length = [0_u8; 4];
            let n = receiver.read(&mut length).unwrap();
            if n == 0 {
                break;
            }
            let length = u32::from_be_bytes(length);

            let mut buffer = vec![0_u8; length as usize];
            receiver.read_exact(&mut buffer).unwrap();
            let message = String::from_utf8_lossy(&buffer);
            println!("{message:?}");
        }
    });

    loop {
        let mut message = String::new();
        std::io::stdin().read_line(&mut message)?;
        let mut buffer = Vec::new();

        let length = message.len() as u32;
        buffer.extend_from_slice(&length.to_be_bytes());
        buffer.extend(message.as_bytes());
        sender.write_all(&buffer)?;
    }
}
