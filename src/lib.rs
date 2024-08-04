#![no_std]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use chumsky::prelude::*;

/// The Linux `candump` file format frame.
///
/// # Example
///
/// Input:
/// ```bash
/// (0000000000.000000) vcan0 211#616C75653A353131
/// (0000000000.000000) vcan0 212#0D0A4104
/// ```
#[derive(Debug)]
pub struct CandumpFrame {
    pub timestamp: core::time::Duration,
    pub interface: String,
    pub id: u32,
    pub data: Vec<u8>,
}

pub fn parser() -> impl Parser<char, CandumpFrame, Error = Simple<char>> {
    let time_unix = text::digits(10).map(|s: String| s.parse().unwrap());
    let time_frac = text::digits(10).map(|s: String| s.parse().unwrap());

    let timestamp = time_unix
        .then_ignore(just("."))
        .then(time_frac)
        .map(|(unix, frac): (u64, u32)| core::time::Duration::new(unix, frac * 1000u32))
        .delimited_by(just('('), just(')'));

    let interface = text::ident();

    let can_id = text::digits(16).map(|s: String| u32::from_str_radix(&s, 16).unwrap());
    let can_data = text::digits(16).map(|s: String| {
        s.as_bytes()
            .chunks(2)
            .map(|chunk| u8::from_str_radix(alloc::str::from_utf8(chunk).unwrap(), 16).unwrap())
            .collect()
    });

    let frame = can_id.then_ignore(just("#")).then(can_data);

    let expr = timestamp
        .then_ignore(just(" "))
        .then(interface)
        .then_ignore(just(" "))
        .then(frame)
        .map(|((timestamp, interface), (id, data))| CandumpFrame {
            timestamp,
            interface,
            id,
            data,
        });

    expr
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    #[test]
    fn test_parser() {
        let input = "(0000000000.100000) vcan0 211#616C75653A353131";
        let result = parser().parse(input);
        assert!(result.is_ok());
        let frame = result.unwrap();
        assert_eq!(frame.timestamp.as_secs(), 0);
        assert_eq!(frame.timestamp.subsec_millis(), 100);
        assert_eq!(frame.interface, "vcan0");
        assert_eq!(frame.id, 0x211);
        assert_eq!(
            frame.data,
            vec![0x61, 0x6C, 0x75, 0x65, 0x3A, 0x35, 0x31, 0x31]
        );
    }
}
