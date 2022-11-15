use std::io::Write;
use std::net::TcpStream;

use rodio::buffer::SamplesBuffer;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;
use tokio::io::BufReader;
use tokio::net::TcpListener;
use rodio::{OutputStream, Sink};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8989").await?;

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    tokio::spawn(async move {
        write_mp3().await;
    });

    loop {
        let (mut socket, _) = listener.accept().await?;
        let sink = Sink::try_new(&stream_handle).unwrap();

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            let mut u16_buf = Vec::new();

            for byte in buf {
                u16_buf.push(byte as u16);
            }

            let source = SamplesBuffer::new(1, 44100, u16_buf.to_owned());
            sink.append(source);
            sink.sleep_until_end();

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // println!("{}", std::str::from_utf8(&buf).expect("Invalid UTF-8 chaxs"));
               // Write the data back
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }

                // socket.shutdown();
            }
        });
    }
}

async fn write_mp3() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8989")?;

    let mut file = File::open("example.mp3").await?;

    let mut buf = Vec::new();

    file.read_to_end(&mut buf).await?;

    stream.write_all(&buf);
    return Ok(())
}

