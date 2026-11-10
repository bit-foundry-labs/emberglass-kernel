//! Pure, testable diagnostic transmission policy for early boot.
//! UART register access remains in the privileged kernel entry module.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SerialError {
    TransmitTimeout,
}

/// Send message bytes through an injected transmitter.
///
/// The first transmit error stops the operation; subsequent bytes are never
/// submitted. The caller decides whether to halt or recover.
pub fn write_message(
    message: &str,
    mut transmit: impl FnMut(u8) -> Result<(), SerialError>,
) -> Result<(), SerialError> {
    for byte in message.bytes() {
        transmit(byte)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sends_each_byte_in_order() {
        let mut sent = [0_u8; 4];
        let mut count = 0;
        let result = write_message("boot", |byte| {
            sent[count] = byte;
            count += 1;
            Ok(())
        });
        assert_eq!(result, Ok(()));
        assert_eq!(&sent[..count], b"boot");
    }

    #[test]
    fn timeout_does_not_transmit_later_bytes() {
        let mut attempts = 0;
        let result = write_message("kernel", |byte| {
            attempts += 1;
            if byte == b'r' {
                Err(SerialError::TransmitTimeout)
            } else {
                Ok(())
            }
        });
        assert_eq!(result, Err(SerialError::TransmitTimeout));
        assert_eq!(attempts, 3);
    }

    #[test]
    fn empty_message_requires_no_uart_access() {
        let result = write_message("", |_| panic!("no bytes were expected"));
        assert_eq!(result, Ok(()));
    }
}
