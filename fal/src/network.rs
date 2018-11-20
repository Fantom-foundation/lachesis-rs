pub trait Transport<T: Message> {
    fn send_message(msg: T) -> Result<T, TransportError>;
}

pub trait Message {

}

pub struct TransportError;
