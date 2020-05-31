use std::{
    fmt,
    fs::File,
    io::{SeekFrom, prelude::*},
    collections::HashMap,
    collections::HashSet,
};
use byteorder::{ByteOrder, BigEndian};
use crate::psi::*;

// Constants
pub const SYNC_BYTE_VAL: u8 = 0x47;
pub const PACKET_SIZE: usize = 188;
pub const HEADER_SIZE: usize = 4;
pub const CRC_SIZE: usize = 4;
const NULL_PACKET_PID: u16 = 0x1FFF;

#[derive(Clone, Debug)]
pub struct Packet {
    transport_error_indicator: bool,
    payload_unit_start_indicator: bool,
    pub transport_priority: bool,
    pub pid: u16,
    pub transport_scrambling_control: u8,
    pub adaptation_field_control: u8,
    pub continuity_counter: u8,
    // TODO: Possibly implement adaptation_field, for now ignore
    pub payload: Vec<u8>,
}

impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[TS] PID {0:#7X}: {0:4}; Transport-error: {1}; Continuity: {2}",
            self.pid, self.transport_error_indicator, self.continuity_counter)
    }
}

impl PartialEq for Packet {
    fn eq(&self, other: &Self) -> bool {
        self.transport_error_indicator == other.transport_error_indicator &&
            self.payload_unit_start_indicator == other.payload_unit_start_indicator &&
            self.transport_priority == other.transport_priority &&
            self.pid == other.pid &&
            self.transport_scrambling_control == other.transport_scrambling_control &&
            self.adaptation_field_control == other.adaptation_field_control &&
            self.continuity_counter == other.continuity_counter
    }
}
impl Eq for Packet {}

impl Default for Packet {
    fn default() -> Self {
        Packet {
            transport_error_indicator: false,
            payload_unit_start_indicator: false,
            transport_priority: false,
            pid: 0,
            transport_scrambling_control: 0,
            adaptation_field_control: 0,
            continuity_counter: 0,
            payload: Vec::default(),
        }
    }
}

impl Packet {
    /// Parse a packet buffer into a Packet object and return it as Option<Packet>
    pub fn new(buf: &[u8]) -> Option<Packet> {
        if buf.len() != PACKET_SIZE || buf[0] != SYNC_BYTE_VAL{
            return None;
        }

        let pid = BigEndian::read_u16(&[buf[1] & 0x1F, buf[2]]);
        let adaptation_field_control = (buf[3] & 0x30) >> 4;
        let adaptation_field_len = buf[4] as usize;
        // A payload only exits in the packet if the adaptation_field_control indicates so
        let payload = if adaptation_field_control == 0x1 {
            buf[(HEADER_SIZE+1)..].to_vec()
        } else if adaptation_field_control == 0x3 {
            buf[(HEADER_SIZE+adaptation_field_len+2)..].to_vec()
        } else {
            vec![]
        };

        Some(Packet {
            transport_error_indicator: get_bit_at(buf[1], 7),
            payload_unit_start_indicator: get_bit_at(buf[1], 6),
            transport_priority: get_bit_at(buf[1], 5),
            pid: pid,
            transport_scrambling_control: (buf[3] & 0xC0) >> 6,
            adaptation_field_control: adaptation_field_control,
            continuity_counter: buf[3] & 0x0F,
            payload: payload,
        })
    }

    /// Update the counts and errors of a PidState object for a given packet
    pub fn update_state(&self, pid_states: &mut HashMap<u16, crate::PidState>,
        pmt_pids: &mut HashSet<u16>) {
        let mut created = false;
        // Get or create the state
        let s =
            match pid_states.get_mut(&self.pid) {
            Some(state) => state,
            None => {
                pid_states.insert(self.pid, crate::PidState {
                    count: 0,
                    duplicate_count: 0,
                    prev_packet: self.clone(),
                    errors: Default::default(),
                });
                created = true;
                pid_states.get_mut(&self.pid).unwrap()
            },
        };

        // Update
        let is_dup = *self == s.prev_packet;
        let last_cc = s.prev_packet.continuity_counter;
        s.count += 1;
        s.duplicate_count = if is_dup { s.duplicate_count + 1 } else { 0 };

        // Check for continuity errors
        if !created && self.pid != NULL_PACKET_PID &&
            self.has_continuity_error(last_cc, s.duplicate_count, is_dup) {
            s.errors.cc_errors += 1;
        }

        // Handle packet if it is a psi packet
        if let Some(psi) = Psi::new(self.payload.as_slice(), &self.pid, &pmt_pids) {
            self.update_pmt_pids(&psi, pmt_pids);
            // Display psi info (Only if it is new according to its crc)
            if let Some(old_psi) = Psi::new(s.prev_packet.payload.as_slice(),
                &self.pid, &pmt_pids) {
                let prev_crc = if !created { old_psi.get_crc() } else { 0 };
                psi.display(prev_crc);
            }

            // Check for crc errors
            if psi.get_crc_error() {
                s.errors.crc_errors += 1;
            }
        }

        // Set the previous packet
        s.prev_packet = self.clone();
    }

    /// Update a Vector of PIDs with the PMT PIDs found a given PAT packet.
    /// If the packet is not a pat, the pmt list won't be touched
    fn update_pmt_pids(&self, psi: &Psi, pmt_pids: &mut HashSet<u16>) {
        if let Psi::Pat(pat) = psi {
            // Add the new pmt pids into our list
            let new_pmt_pids = pat.get_pmt_pids();
            pmt_pids.extend(&new_pmt_pids);
        }
    }

    fn has_continuity_error(&self, last_cc: u8, dup_count: u32, is_dup: bool) -> bool {
        let next_cc = if last_cc == 15 { 0 } else { last_cc + 1};
        if self.continuity_counter != next_cc {
            if !(self.adaptation_field_control == 0x0 || self.adaptation_field_control == 0x2) &&
                !(is_dup && dup_count < 2) {
                    return true;
            }
        }
        false
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
