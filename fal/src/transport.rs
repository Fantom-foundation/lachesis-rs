pub trait Transport<T: Message, U: TransportError> {
    fn send_message(msg: T) -> Result<T, U>;
}

pub trait Message {}

pub trait TransportError {}
