pub mod app;
pub mod buffer;
pub mod irq;
use bbqueue::Error as BbqError;

#[derive(Debug)]
pub enum Error {
    Bbqueue(BbqError),
}
