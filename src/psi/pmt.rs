use std::fmt;
use std::fmt::Write;
use std::collections::HashMap;
use byteorder::{ByteOrder, BigEndian};
use crate::packet;

const PMT_TABLE_ID: u8 = 0x02;
/// Start index of the variable section of the payload
/// ex. for(i=0;i<N;i++) { ... }
const VARIABLE_SEC_START_INDEX: u16 = 12;

#[derive(Debug)]
pub struct ElementaryStream {
    // TODO: change stream type to enum
    stream_type: u8,
    elementary_pid: u16,
    // TODO: add descriptors. For now we skip them
}

#[derive(Debug)]
pub struct Pmt {
    section_syntax_indicator: bool,
    section_length: u16,
    program_number: u16,
    version_number: u8,
    current_next_indicator: bool,
    section_number: u8,
    last_section_number: u8,
    pcr_pid: u16,
    program_info_length: u16,
    pub elementary_streams: Vec<ElementaryStream>,
    pub crc: u32,
}

impl fmt::Display for Pmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut elem_str = String::new();
        for e in &self.elementary_streams {
            write!(&mut elem_str,
                "\t=> Steam Type: {:#2X}, Elementary PID: {:#4X}\n", e.stream_type, e.elementary_pid).unwrap();
        }
        write!(f, "[PMT] Program Number: {:#4X}, Version: {:#2X}, PCR PID: {:#4X}\n{}",
            self.program_number, self.version_number, self.pcr_pid, elem_str)
    }
}

impl Pmt {
    pub fn new(buf: &[u8]) -> Option<Pmt> {
        if buf.len() != packet::PAYLOAD_SIZE || buf[0] != PMT_TABLE_ID {
            return None;
        }
        // Calculate length and index fields
        let section_length = BigEndian::read_u16(&[buf[1] & 0x0F, buf[2]]);
        let program_info_length = BigEndian::read_u16(&[buf[10] & 0x0F, buf[11]]);
        let section_end_index = super::PSI_SEC_START_INDEX + section_length - 1;
        let variable_section_end_index = section_end_index - (packet::CRC_SIZE as u16);

        // Skip past descriptor() fields (program info)
        let mut variable_index = VARIABLE_SEC_START_INDEX + program_info_length;
        let mut elementary_streams: Vec<ElementaryStream> = vec![];
        while variable_index <= variable_section_end_index {
            let x = variable_index as usize;
            elementary_streams.push(ElementaryStream {
                stream_type: buf[x],
                elementary_pid: BigEndian::read_u16(&[buf[x+1] & 0x1F, buf[x+2]]),
            });
            // Skip past the nested descriptor() fields
            // TODO: Stop ignoring descriptor fields
            let es_info_length = BigEndian::read_u16(&[buf[x+3] & 0x0F, buf[x+4]]);
            variable_index += 5 + es_info_length;
        }

        Some(Pmt {
            section_syntax_indicator: packet::get_bit_at(buf[1], 7),
            section_length: section_length,
            program_number: BigEndian::read_u16(&[buf[3], buf[4]]),
            version_number: (buf[5] & 0x3E) >> 1,
            current_next_indicator: packet::get_bit_at(buf[5], 0),
            section_number: buf[6],
            last_section_number: buf[7],
            pcr_pid: BigEndian::read_u16(&[buf[8] & 0x1F, buf[9]]),
            program_info_length: program_info_length,
            elementary_streams: elementary_streams,
            crc: BigEndian::read_u32(&buf[(variable_section_end_index as usize)+1..=(section_end_index as usize)]),
        })
    }

    /// Print out PMT info. Only display each PMT once
    pub fn display(&self, pid: &u16, crc_map: &mut HashMap<u16, u32>) {
        // If this PID has been seen before, then..., else note that we have seen it and display it
        if let Some(last_crc) = crc_map.get_mut(pid) {
            // If this PMT has changed, then display it, else ignore it
            if self.crc != *last_crc {
                *last_crc = self.crc;
                println!("{}", self);
            }
        } else {
            crc_map.insert(*pid, self.crc);
            println!("{}", self);
        }
    }
}
