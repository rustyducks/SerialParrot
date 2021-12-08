use std::mem;
use crate::Transport;

enum RcvState {
    START1,
    START2,
    LEN,
    PAYLOAD(u8),    // nb bytes remaining to complete the message
    CHK,
}

pub struct DuckLinkTransport {
    state: RcvState,
    buffer: Box<Vec<u8>>,
}

impl DuckLinkTransport {

    pub fn new() -> Self {
        DuckLinkTransport{state:RcvState::START1, buffer: Box::new(Vec::new())}
    }

    fn checksum(buffer: &[u8]) -> u8 {
        buffer.iter().fold(0, |acc, elt| acc ^ elt)
    }

    
}

impl Transport for DuckLinkTransport {
    fn put(&mut self, c: u8) -> Option<Box<Vec<u8>>> {
        self.buffer.push(c);
        match self.state {
            RcvState::START1 => {
                if c == 0xFF {
                    self.buffer.clear();
                    self.buffer.push(c);
                    self.state = RcvState::START2;
                }
            },
            RcvState::START2 => {
                if c == 0xFF {
                    self.state = RcvState::LEN;
                } else {
                    self.state = RcvState::START1;
                }
            },
            RcvState::LEN => {
                self.state = RcvState::PAYLOAD(c);
            },
            RcvState::PAYLOAD(n) => {
                let n = n-1;
                if n > 0 {
                    self.state = RcvState::PAYLOAD(n);    
                }
                else {
                    self.state = RcvState::CHK;
                }
            },
            RcvState::CHK => {
                self.state = RcvState::START1;
                let buf_len = self.buffer.len();
                let chk = Self::checksum(&self.buffer[3..buf_len-1]);
                if chk == c {
                    // swap buffer to a new buffer
                    let mut b:Box<Vec<u8>>=Box::new(Vec::with_capacity(10));
                    mem::swap(&mut self.buffer, &mut b);
                    return Some(b);
                } else {
                    println!("checksum failed: {} {}", c, chk);
                }
            },
        }
        None
    }
}
