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
