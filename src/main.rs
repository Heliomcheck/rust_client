use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use std::io;
use anyhow::{Context, Result};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await
        .context("Cat't connect to server")?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)
        .context("Read with error")?; 

    stream.write_all(input.as_bytes()).await
        .context("Write with error")?;
          
    Ok(())
}