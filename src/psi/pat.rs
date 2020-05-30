use std::fmt;
use std::fmt::Write;
use byteorder::{ByteOrder, BigEndian};
use crate::packet;

// Constants
const PAT_TABLE_ID: u8 = 0x0;

#[derive(Debug, PartialEq)]
pub enum ProgramInfoType {
    Network,
    ProgramMap,
}

#[derive(Debug)]
pub struct ProgramInfo {
    program_number: u16,
    program_info_type: ProgramInfoType,
    pid: u16,
}

#[derive(Debug)]
pub struct Pat {
    syntax_section_indicator: bool,
    section_length: u16,
    transport_stream_id: u16,
    version_number: u8,
    current_next_indicator: bool,
    section_number: u8,
    last_section_number: u8,
    pub program_info: Vec<ProgramInfo>,
    pub crc: u32,
}

impl fmt::Display for Pat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut program_str = String::new();
        for p in &self.program_info {
            if p.program_info_type == ProgramInfoType::ProgramMap {
                write!(&mut program_str,
                    "\n\t=> Program num: {0:#X}, Program map PID: {1:#X} ({1})",
                    p.program_number, p.pid).unwrap();
            } else {
                write!(&mut program_str, "\n\t=> Network PID, not a program").unwrap();
            }
        }
        write!(f, "[PAT] Tranport Stream ID: {0:#X} ({}), Version: {1:#X}{2}",
            self.transport_stream_id, self.version_number, program_str)
    }
}

impl Pat {
    pub fn new(buf: &[u8]) -> Option<Pat> {
        if buf.len() != packet::PAYLOAD_SIZE || buf[0] != PAT_TABLE_ID {
            return None;
        }
        // Calculate length and index fields
        let section_length = BigEndian::read_u16(&[buf[1] & 0x0F, buf[2]]);
        let section_end = (super::PSI_SEC_START_INDEX + section_length) as usize;

        // Get program info
        let mut n = 8;
        let end_n = section_end - packet::CRC_SIZE;
        let mut prog_infos: Vec<ProgramInfo> = vec![];
        while n < end_n {
            let program_number = BigEndian::read_u16(&[buf[n], buf[n+1]]);
            prog_infos.push(ProgramInfo {
                program_number: program_number,
                program_info_type: if program_number == 0 {
                    ProgramInfoType::Network
                } else {
                    ProgramInfoType::ProgramMap
                },
                pid: BigEndian::read_u16(&[buf[n+2] & 0x1F, buf[n+3]]),
            });
            n += 4;
        }

        Some(Pat {
            syntax_section_indicator: packet::get_bit_at(buf[1], 7),
            section_length: section_length,
            transport_stream_id: BigEndian::read_u16(&[buf[3], buf[4]]),
            version_number: (buf[5] & 0x3E) >> 1,
            current_next_indicator: packet::get_bit_at(buf[5], 0),
            section_number: buf[6],
            last_section_number: buf[7],
            program_info: prog_infos,
            crc: BigEndian::read_u32(&buf[end_n..=section_end]),
        })
    }

    /// Get a list of PMT PIDs in this PAT packet
    pub fn get_pmt_pids(&self) -> Vec<crate::Pid> {
        let mut p: Vec<crate::Pid> = vec![];
        for i in &self.program_info {
            if i.program_info_type == ProgramInfoType::ProgramMap {
                p.push(crate::Pid {value: i.pid, count: 1});
            }
        }
        p
    }

    /// Print out PAT info. Only display each PAT once
    pub fn display(&self, last_crc: &mut u32) {
        // Only print it if the PAT has changed
        if self.crc != *last_crc {
            println!("{}", self);
            *last_crc = self.crc;
        }
    }
}
