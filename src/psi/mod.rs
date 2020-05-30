use std::fmt;
use crate::packet;
pub mod pat;
pub mod pmt;

/// Start index of the psi section:
/// The index starting immediately following "section_length" field
const PSI_SEC_START_INDEX: u16 = 3;

pub enum Psi {
    Pat(pat::Pat),
    Pmt(pmt::Pmt),
}

impl fmt::Display for Psi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            Psi::Pat(p) => write!(f, "{}", p),
            Psi::Pmt(p) => write!(f, "{}", p),
        }
    }
}

impl Psi {
    pub fn new(buf: &[u8], pid: &u16, pmt_pids: &mut Vec<crate::Pid>) -> Option<Psi> {
        // generate the PSI struct according to the pid value
        match pid {
            x if Psi::is_pat(&x) => {
                if let Some(pt) = Psi::parse_pat(&buf) {
                    if let Psi::Pat(p) = pt {
                        // Get the PMT pids from the packet and add the new ones to the list
                        let pids = p.get_pmt_pids();
                        for p in pids {
                            match pmt_pids.binary_search_by(|x| x.value.cmp(&p.value)) {
                                Ok(pos) => pmt_pids[pos].count += 1,
                                Err(pos) => pmt_pids.insert(pos, p),
                            }
                        }
                        return Some(Psi::Pat(p));
                    }
                }
                None

            },
            x if Psi::is_network_program_elementary(&x) => {
                // a PMT packet MUST be in the list of pids provided by the PATs
                if Psi::is_pmt(&x, &pmt_pids) {
                    return Psi::parse_pmt(&buf);
                }
                None
            },
            _ => None,
        }
    }

    /// Checks if a PID is a PAT
    pub fn is_pat(pid: &u16) -> bool { *pid == 0x0 }
    /// Checks if a PID is a Network, Program map, or Elementary
    pub fn is_network_program_elementary(pid: &u16) -> bool { *pid >= 0x0010 && *pid <= 0x1FFE }
    /// Checks if a PID is a PMT
    fn is_pmt(pid: &u16, pids: &Vec<crate::Pid>) -> bool {
        match pids.binary_search_by(|x| x.value.cmp(pid)) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Parse a PAT buffer into an Option<Psi>
    fn parse_pat(buf: &[u8]) -> Option<Psi> {
        match pat::Pat::new(&buf[packet::HEADER_SIZE..]) {
            Some(p) => {
                Some(Psi::Pat(p))
            },
            None => None,
        }
    }

    /// Parse a PMT buffer into an Option<Psi>
    fn parse_pmt(buf: &[u8]) -> Option<Psi> {
        match pmt::Pmt::new(&buf[packet::HEADER_SIZE..]) {
            Some(p) => {
                Some(Psi::Pmt(p))
            },
            None => None,
        }
    }
}


#[derive(Debug)]
pub struct ElementaryStream {
    stream_type: u8,
    elementary_pid: u16,
    pub descriptors: Vec<VideoStreamDescriptor>,
}

impl ElementaryStream {
    // TODO: Look into making something more efficient (maybe a macro)
    pub fn to_string(&self) -> &'static str {
        match self.stream_type {
            0x00 => "Reserved",
            0x01 => "MPEG-1 video stream",
            0x02 => "MPEG-2 video stream",
            0x03 => "MPEG-1 audio stream",
            0x04 => "MPEG-2 audio stream",
            0x05 => "MPEG-2 private sections",
            0x06 => "MPEG-2 PES packets",
            0x07 => "MHEG-5 Audio-Visual streams",
            0x08 => "DSM-CC ISO/IEC 13818-1 Annex A",
            0x09 => "ITU-T Satellite Audio-Visual streams",
            0x0A => "MPEG-2 Video Clip A streams",
            0x0B => "MPEG-2 Video Clip B streams",
            0x0C => "MPEG-2 Video Clip C streams",
            0x0D => "MPEG-2 Video Clip D streams",
            0x0E => "MPEG-2 Auxiliary streams",
            0x0F => "MPEG-2 Audio with ADTS transport syntax",
            0x10 => "MPEG-4 Visual",
            0x11 => "MPEG-4 Audio",
            0x12 => "ISO/IEC 14496-1",
            0x13 => "ISO/IEC 14496-1",
            0x14 => "ISO/IEC 13818-6",
            0x15..=0x7F => "ITU-T Rec. H.222.0 | ISO/IEC 13818-1 Reserved",
            0x80..=0xFF => "User Private",
        }
    }
}

#[derive(Debug)]
pub struct VideoStreamDescriptor {
    tag: u8,
    length: u8,
}

impl VideoStreamDescriptor {
    // TODO: Look into making something more efficient (maybe a macro)
    pub fn to_string(&self) -> &'static str {
        match self.tag {
            0 => "Reserved",
            1 => "Reserved",
            2 => "Video Stream",
            3 => "Audio Stream",
            4 => "Hierarchy",
            5 => "Registration",
            6 => "Data Stream Alignment",
            7 => "Target Background Grid",
            8 => "Video Window",
            9 => "CA",
            10 => "ISO 639 Language",
            11 => "System Clock",
            12 => "Multiplex Buffer Utilization",
            13 => "Copyright",
            14 => "Maximum Bitrate",
            15 => "Private Data Indicator",
            16 => "Smoothing Buffer",
            17 => "STD",
            18 => "IBP",
            19..=26 => "Defined in ISO/IEC 13818-6",
            27 => "MPEG-4 Video",
            28 => "MPEG-4 Audio",
            29 => "IOD",
            30 => "SL",
            31 => "FMC",
            32 => "External ES ID",
            33 => "MuxCode",
            34 => "FmxBufferSize",
            35 => "Multiplexbuffer",
            36 => "Content Labeling",
            37 => "Metadata Pointer",
            38 => "Metadata",
            39 => "Metadata STD",
            40 => "AVC Video",
            41 => "IPMP",
            42 => "AVC Timing and HRD",
            43 => "MPEG-2 AAC Audio",
            44 => "FlexMuxTiming",
            45..=63 => "ITU-T Rec. H.222.0 | ISO/IEC 13818-1 Reserved",
            64..=255 => "User Private",
        }
    }
}
