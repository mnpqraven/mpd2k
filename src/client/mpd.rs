use crate::error::{AppError, MpdError};
use std::fmt::Debug;
use tokio::{
    io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf},
    net::{TcpStream, ToSocketAddrs},
};
use tracing::{info, instrument};

pub struct MpdClient {
    r: BufReader<ReadHalf<TcpStream>>,
    w: WriteHalf<TcpStream>,
}

#[derive(Debug)]
pub enum MpdCommand {
    Status,
    Play(usize),
}

impl MpdCommand {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            MpdCommand::Status => b"status\n".to_vec(),
            MpdCommand::Play(index) => format!("play {index}\n").into(),
        }
    }
}

impl MpdClient {
    #[instrument]
    pub async fn init<T>(addr: T) -> Result<Self, AppError>
    where
        T: ToSocketAddrs + Debug,
    {
        let stream = TcpStream::connect(addr).await.unwrap();
        let (r, w) = split(stream);
        let mut client = Self {
            r: BufReader::new(r),
            w,
        };
        // let buf = &mut [0; 7];
        let mut buf = String::new();
        client.r.read_line(&mut buf).await?;

        // expects OK MPD [version] or ACK [reasons]
        if !buf.starts_with("OK MPD ") {
            return Err(MpdError::MpdClient(format!("[init connection] {}", buf)).into());
        }

        info!("MPD client connected");
        Ok(client)
    }

    #[instrument(ret, skip(self))]
    pub async fn command(&mut self, command: MpdCommand) -> Result<Vec<String>, AppError> {
        self.w.write_all(&command.as_bytes()).await?;
        let mut lines = (&mut self.r).lines();
        let mut messages: Vec<String> = vec![];

        while let Some(line) = lines.next_line().await? {
            let (has_ok, has_err) = (line.eq("OK"), line.starts_with("ACK"));

            match (has_ok, has_err) {
                (false, false) => {
                    messages.push(line);
                }
                (true, _) => {
                    return Ok(messages);
                }
                (_, true) => {
                    return Err(MpdError::MpdProtocol(messages, line).into());
                }
            };
        }

        Ok(messages)
    }
}
