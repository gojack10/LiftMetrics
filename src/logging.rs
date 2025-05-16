use std::io::Write;
use std::sync::mpsc::Sender;

// A custom writer that sends log messages through an MPSC channel.
pub struct ChannelWriter {
    sender: Sender<String>,
}

impl ChannelWriter {
    pub fn new(sender: Sender<String>) -> Self {
        ChannelWriter { sender }
    }
}

impl Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Ok(message) = String::from_utf8(buf.to_vec()) {
            // Send the message through the channel.
            // We ignore the error here, as the receiver might be dropped if the app exits.
            let _ = self.sender.send(message);
            Ok(buf.len())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid UTF-8 data",
            ))
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // No-op for this writer
        Ok(())
    }
}

// TODO: Add logger initialization function here

use env_logger::Builder;
use log::LevelFilter;

pub fn init_logger(sender: Sender<String>) {
    let writer = ChannelWriter::new(sender);

    let mut builder = Builder::new();
    builder
        .filter(None, LevelFilter::Info) // Set the default log level
        .format(|buf, record| {
            // Customize the log message format
            writeln!(
                buf,
                "$ [{}] {}",
                record.level(),
                record.args()
            )
        })
        .target(env_logger::Target::Pipe(Box::new(writer)))
        .init();
}
