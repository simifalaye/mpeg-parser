use std::fmt;
use byteorder::{ByteOrder, BigEndian};

pub mod packet;
pub mod psi;

#[derive(Debug)]
pub struct Pid {
    pub value: u16,
    pub count: u32,
}

impl Pid {
    pub fn get_packet_pid(buffer: &[u8]) -> u16 {
        return BigEndian::read_u16(&[buffer[1] & 0x1F, buffer[2]]);
    }

    pub fn print_pids(pids: &Vec<Pid>) {
        println!("");
        println!("PIDs:");
        for pid in pids {
            println!("{}, {:2.2}%", pid, (pid.count as f64 / pids.len() as f64) * 100f64);
        }
    }
}

impl fmt::Display for Pid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PID {0:#7X}: {0:4}; count: {1:7}", self.value, self.count)
    }
}
