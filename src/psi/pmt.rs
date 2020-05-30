use std::fmt;
use std::fmt::Write;
use std::collections::HashMap;
use byteorder::{ByteOrder, BigEndian};
use super::{VideoStreamDescriptor,ElementaryStream};
use crate::packet;

// Constants
const PMT_TABLE_ID: u8 = 0x02;

#[derive(Debug)]
pub struct Pmt {
    section_syntax_indicator: bool,
    program_number: u16,
    version_number: u8,
    current_next_indicator: bool,
    section_number: u8,
    last_section_number: u8,
    pcr_pid: u16,
    program_info_length: u16,
    pub descriptors: Vec<VideoStreamDescriptor>,
    pub elementary_streams: Vec<ElementaryStream>,
    pub crc: u32,
}

impl fmt::Display for Pmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut des_str = String::new();
        let mut elem_str = String::new();
        // Generate top level descriptors string
        for i in &self.descriptors {
            write!(&mut des_str,
                "\n\t=> Descriptor Tag: {0:#X} ({1}), length: {2}",
                i.tag, i.to_string(), i.length).unwrap();
        }
        // Generate elementary streams string
        for i in &self.elementary_streams {
            write!(&mut elem_str,
                "\n\t=> Steam Type: {0:#X} ({1}), Elementary PID: {2:#X}",
                i.stream_type, i.to_string(), i.elementary_pid).unwrap();
            for j in &i.descriptors {
                write!(&mut elem_str,
                    "\n\t\t=> Descriptor Tag: {0:#X} ({1}), length: {2}",
                    j.tag, j.to_string(), j.length).unwrap();
            }
        }
        write!(f, "[PMT] Program Number: {0}, Version: {1}, PCR PID: {2:#X} ({2}){3}{4}",
            self.program_number, self.version_number, self.pcr_pid, des_str, elem_str)
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
        let section_end = (super::PSI_SEC_START_INDEX + section_length) as usize;

        // Get Top level descriptors
        let mut n = 12;
        let end_n = n + program_info_length as usize;
        let mut descriptors: Vec<VideoStreamDescriptor> = vec![];
        while n < end_n {
            let len_n = buf[n+1];
            descriptors.push(VideoStreamDescriptor {
                tag: buf[n],
                length: len_n,
            });
            n += 2 + len_n as usize;
        }

        // Get Stream info
        let mut n1 = end_n;
        let end_n1 = section_end - packet::CRC_SIZE;
        let mut elementary_streams: Vec<ElementaryStream> = vec![];
        while n1 < end_n1 {
            let stream_type = buf[n1];
            let elementary_pid = BigEndian::read_u16(&[buf[n1+1] & 0x1F, buf[n1+2]]);
            let es_info_length = BigEndian::read_u16(&[buf[n1+3] & 0x0F, buf[n1+4]]);

            // Get Bottom level descriptors
            let mut n2 = n1 + 5;
            let end_n2 = n2 + es_info_length as usize;
            let mut elementary_stream_descriptors: Vec<VideoStreamDescriptor> = vec![];
            while n2 < end_n2 {
                let len_n2 = buf[n2+1];
                elementary_stream_descriptors.push(VideoStreamDescriptor {
                    tag: buf[n2],
                    length: len_n2,
                });
                n2 += 2 + len_n2 as usize;
            }

            elementary_streams.push(ElementaryStream {
                stream_type: stream_type,
                elementary_pid: elementary_pid,
                descriptors: elementary_stream_descriptors,
            });

            n1 = end_n2;
        }

        Some(Pmt {
            section_syntax_indicator: packet::get_bit_at(buf[1], 7),
            program_number: BigEndian::read_u16(&[buf[3], buf[4]]),
            version_number: (buf[5] & 0x3E) >> 1,
            current_next_indicator: packet::get_bit_at(buf[5], 0),
            section_number: buf[6],
            last_section_number: buf[7],
            pcr_pid: BigEndian::read_u16(&[buf[8] & 0x1F, buf[9]]),
            program_info_length: program_info_length,
            descriptors: descriptors,
            elementary_streams: elementary_streams,
            crc: BigEndian::read_u32(&buf[end_n1..=section_end]),
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
