use std::{
    fmt,
    fs::File,
    io::{SeekFrom, prelude::*},
};
use byteorder::{ByteOrder, BigEndian};
use crate::psi;

// Constants
pub const SYNC_BYTE_VAL: u8 = 0x47;
pub const PACKET_SIZE: usize = 188;
pub const HEADER_SIZE: usize = 5;
pub const CRC_SIZE: usize = 4;
pub const PAYLOAD_SIZE: usize = PACKET_SIZE - HEADER_SIZE;

#[allow(dead_code)]
pub struct Packet {
    transport_error_indicator: bool,
    payload_unit_start_indicator: bool,
    transport_priority: bool,
    pub pid: u16,
    transport_scrambling_control: u8,
    adaptation_field_control: u8,
    continuity_counter: u8,
    pub psi: Option<psi::Psi>,
    // TODO: Implement adaptation fields
}

impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut psi_str = String::from("\t=> No psi info!");
        if let Some(p) = &self.psi {
            psi_str = format!("\t{}", p);
        }
        write!(f, "[TS] PID {0:#7X}: {0:4}; Transport-error: {1}; Continuity: {2}\n{3}",
            self.pid, self.transport_error_indicator, self.continuity_counter, psi_str)
    }
}

impl Packet {
    pub fn new(buf: &[u8], pmt_pids: &mut Vec<crate::Pid>) -> Option<Packet> {
        if buf.len() != PACKET_SIZE || buf[0] != SYNC_BYTE_VAL{
            return None;
        }

        let pid = BigEndian::read_u16(&[buf[1] & 0x1F, buf[2]]);
        let psi = psi::Psi::new(&buf, &pid, pmt_pids);
        Some(Packet {
            transport_error_indicator: get_bit_at(buf[1], 7),
            payload_unit_start_indicator: get_bit_at(buf[1], 6),
            transport_priority: get_bit_at(buf[1], 5),
            pid: pid,
            transport_scrambling_control: (buf[3] & 0xC0) >> 6,
            adaptation_field_control: (buf[3] & 0x30) >> 4,
            continuity_counter: buf[3] & 0x0F,
            psi: psi,
        })
    }

}

/// Moves file pointer to sync byte of a transport stream file
pub fn advance_file_to_sync_byte(file: &mut File) -> bool {
    // Only look in the first 1024 packets at most
    // (if we can't find the sync byte by then, this ts file sucks)
    let mut buffer = [0u8; PACKET_SIZE * 1024];
    let n = file.read(&mut buffer[..]).expect("Unable to read file.");

    for i in 0..n {
        // we only found a "potential" sync byte (might be a erroneous sync byte)
        // confirm it is, in fact, the sync byte by checking the next n packets' first byte
        // to make sure they are also sync bytes
        if buffer[i] == SYNC_BYTE_VAL {
            let mut is_valid = true;
            println!("Found potential sync byte at index={}", i);
            for j in 1..=3 {
                let val_index = i + (j * PACKET_SIZE);
                if val_index > n {
                    println!("No sync byte could be found!");
                    return false;
                }

                print!("Checking next sync byte at index={}...", val_index);
                if buffer[val_index] != SYNC_BYTE_VAL {
                    is_valid = false;
                    print!(" => !! INVALID !! val=0x{:X?}\n", buffer[val_index]);
                    break;
                }
                print!(" => VALID; val=0x{:X?}\n", buffer[val_index]);
            }
            if is_valid {
                // Seek back to the position we found
                file.seek(SeekFrom::Start(i as u64)).unwrap();
                print!("\n");
                return true;
            }
        }
    }
    false
}

/// Gets the bit at position `n`.
/// Bits are numbered from 0 (least significant) to 7 (most significant).
pub fn get_bit_at(input: u8, n: u8) -> bool {
    if n < 8 {
        input & (1 << n) != 0
    } else {
        false
    }
}
