use std::fmt;
use std::fmt::Write;
use byteorder::{ByteOrder, BigEndian};
use crate::packet;

const PAT_TABLE_ID: u8 = 0x0;
const PROGRAM_INFO_START_INDEX: u16 = 8;

#[derive(Debug)]
pub struct ProgramInfo {
    program_number: u16,
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
    program_info: Vec<ProgramInfo>,
}

impl fmt::Display for Pat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut program_str = String::new();
        for p in &self.program_info {
            if p.program_number != 0 {
                write!(&mut program_str,
                    "\t\t=> Program num: {:#4X}, Program map PID: {:#4X}\n", p.program_number, p.pid).unwrap();
            } else {
                write!(&mut program_str, " => Network PID, not a program").unwrap();
            }
        }
        write!(f, "\tPAT | Tranport Stream ID: {:#4X}, Version: {:#2X}\n{}",
            self.transport_stream_id, self.version_number, program_str)
    }
}

impl Pat {
    pub fn new(buf: &[u8]) -> Option<Pat> {
        if buf.len() != packet::PAYLOAD_SIZE || buf[0] != PAT_TABLE_ID {
            println!("Payload len: {}", buf.len());
            return None;
        }
        let section_length = BigEndian::read_u16(&[buf[1] & 0x0F, buf[2]]);
        let program_info_end = (2 + section_length) - (packet::CRC_SIZE as u16 - 1);
        let mut prog_infos: Vec<ProgramInfo> = vec![];
        for i in PROGRAM_INFO_START_INDEX..program_info_end {
            let x = i as usize;
            prog_infos.push(ProgramInfo {
                program_number: BigEndian::read_u16(&[buf[x], buf[x+1]]),
                pid: BigEndian::read_u16(&[buf[x+2] & 0x1F, buf[x+3]]),
            });
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
        })
    }

    // pub fn display_pat(pk: &[u8], last_crc: &mut u32) {
    //     let sec = &pk[5..];
    //     let table_id = sec[0];
    //     if Pid::get_packet_pid(&pk) != PAT_PID || table_id != PAT_TID {
    //         return;
    //     }
    //     // Get section info
    //     let section_length = BigEndian::read_u16(&[sec[1] & 0x0F, sec[2]]);
    //     let section_end = 2 + section_length;
    //     let crc = BigEndian::read_u32(&sec[(section_end as usize - 3)..=section_end as usize]);
    //     *last_crc = if crc == *last_crc { return } else { crc };
    //     // Get PAT data
    //     let transport_stream_id = BigEndian::read_u16(&[sec[3], sec[4]]);
    //     let version_num = (sec[5] & 0x3E) >> 1;

    //     println!("Table ID: {:#2X}, Tranport Stream ID: {:#4X}, Version: {:#2X}",
    //         table_id, transport_stream_id, version_num);
    //     // Get program data
    //     let mut i = 8u16;
    //     let program_section_end = (section_length - PAT_SECTION_SCALAR) + i;
    //     while i < program_section_end {
    //         let j = i as usize;
    //         let program_num = BigEndian::read_u16(&[sec[j], sec[j+1]]);
    //         if program_num != 0 {
    //             let program_map_pid = BigEndian::read_u16(&[sec[j+2] & 0x1F, sec[j+3]]);
    //             println!(" => Program num: {:#4X}, Program map PID: {:#4X}", program_num, program_map_pid);
    //         } else {
    //             println!(" => Network PID, not a program");
    //         }
    //         i += 4;
    //     }
    // }
}
