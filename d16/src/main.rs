use std::cmp;
use std::fmt;
use std::io;
use std::io::BufRead;
use std::ops;

#[derive(Clone, Copy)]
pub enum LengthTypeId {
    Bits(usize),
    Count(usize),
}

#[derive(Clone, Copy)]
pub enum Header {
    Literal(PacketVersion),
    Operator(PacketVersion, OperatorId),
}

impl Header {
    pub fn version(&self) -> PacketVersion {
        match self {
            Header::Literal(v) => *v,
            Header::Operator(v, _) => *v,
        }
    }

    pub fn is_literal(&self) -> bool {
        match self {
            Header::Literal(_) => true,
            _ => false,
        }
    }

    pub fn is_operator(&self) -> bool {
        !self.is_literal()
    }
}

#[derive(Clone, Copy)]
pub enum Packet {
    Literal(Header, u32),
    Operator(Header, OperatorId, LengthTypeId),
    //OperatorEnd,
}

impl Packet {
    pub fn version(&self) -> PacketVersion {
        match self {
            Packet::Literal(h, _) => h,
            Packet::Operator(h, _ , _) => h,
        }.version()
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Packet::Literal(h, v) => write!(f, "({}):{}", h.version(), v),
            Packet::Operator(h, id, l) => {
                let (prefix, n) = match l {
                    LengthTypeId::Bits(b) => ("b", b),
                    LengthTypeId::Count(c) => ("", c),
                };
                write!(f, "({}):BEGIN_{}({}{})", h.version(), id, prefix, n)
            },
            //Packet::OperatorEnd => write!(f, "END"),
        }
    }
}

type PacketVersion = u8;
type OperatorId = u8;
type BitsCount = usize;

#[derive(Clone)]
pub struct BitsReader<'a> {
    stream: &'a [u8],
    n: usize,
    b: usize,
}

impl<'a> BitsReader<'a> {
    pub fn bits_pos(&self) -> usize {
        self.n * 8 + self.b
    }

    pub fn bits_pos_add(&self, bits: usize) -> usize {
        self.bits_pos() + bits
    }

    pub fn read_header(&mut self) -> Option<(Header, BitsCount)> {
        let mut b = [0];
        match self.read_into(&mut b) {
            0 => None,
            _ => {
                let b = b[0];
                let version = (b & 0b_1110_0000) >> 5;
                *self += 6;
                let header = match (b & 0b_0001_1100) >> 2 {
                    4 => Header::Literal(version),
                    n => Header::Operator(version, n),
                };
                Some((header, 6))
            }
        }
    }

    fn read_packet_count(&mut self) -> LengthTypeId {
        let mut b = [0;2];
        self.read_into(&mut b);
        if b[0] < 0b_1000_0000 {
            let l = (((b[0] & 0b_0111_1111) as usize) << 8) | b[1] as usize;
            *self += 16;
            LengthTypeId::Bits(l)
        } else {
            let c = (((b[0] & 0b_0111_1111) as usize) << 4) | (b[1] >> 4) as usize;
            *self += 12;
            LengthTypeId::Count(c)
        }
    }

    pub fn read_literal(&mut self) -> (u32, BitsCount) {
        let mut v = 0;
        let mut buf = [0];
        let mut bits_read = 0;
        loop {
            self.read_into(&mut buf);
            let byte = buf[0];
            v = (v << 4) | ((byte & 0b_0111_1000) >> 3) as u32;
            *self += 5;
            bits_read += 5;
            if byte < 0b_1000_0000 {
                break;
            }
        }
        (v, bits_read)
    }

    fn read_into(&self, buf: &mut [u8]) -> usize {
        if self.n + cmp::min(1, self.b) >= self.stream.len() {
            return 0
        }

        if self.b == 0 {
            let n = cmp::min(buf.len(), self.stream.len() - self.n);
            buf[0..n].copy_from_slice(&self.stream[self.n..self.n+n]);
            n
        } else {
            let n = cmp::min(buf.len(), self.stream.len() - self.n - 1);
            for i in 0..n {
                let src = &self.stream[self.n..cmp::min(self.n+n+1, self.stream.len())];
                buf[i] = (src[i] << self.b) | (src[i+1] >> (8 - self.b));
            }
            n
        }
    }
}

impl<'a> ops::AddAssign<BitsCount> for BitsReader<'a> {
    fn add_assign(&mut self, inc: BitsCount) {
        let b = self.b + inc;
        self.n = self.n + (b / 8);
        self.b = b % 8;
    }
}

impl<'a> Iterator for BitsReader<'a> {
    type Item = Packet;

    fn next(&mut self) -> Option<Packet> {
        let header = match self.read_header() {
            Some((h, _)) => h,
            None => return None,
        };

        match header {
            Header::Literal(_) => {
                let (v, _) = self.read_literal();
                Some(Packet::Literal(header, v))
            }
            Header::Operator(_, id) => {
                let length_type = self.read_packet_count();
                Some(Packet::Operator(header, id, length_type))
            }
        }
    }
}

pub trait IntoBitsReader {
    fn read_bits<'a>(&'a self) -> BitsReader<'a>;
}

impl<B> IntoBitsReader for B where B: AsRef<[u8]>
{
    fn read_bits<'a>(&'a self) -> BitsReader<'a> {
        BitsReader { stream: self.as_ref(), n: 0, b: 0 }
    }
}

pub struct PacketNode(Packet, Vec<PacketNode>);

impl PacketNode {
    pub fn from_bits<'a>(reader: &'a mut BitsReader) -> Option<PacketNode> {
        let packet = match reader.next() {
            Some(p) => p,
            None => return None,
        };

        //println!("{:?}", &packet);

        Some(match packet {
            Packet::Operator(_, _, LengthTypeId::Count(len)) => {
                let nodes = PacketNode::take_until_count(reader, len);
                PacketNode(packet, nodes)
            },
            Packet::Operator(_, _, LengthTypeId::Bits(bits)) => {
                PacketNode(packet, PacketNode::take_until_bits(reader, bits))
            },
            _ => {
                PacketNode(packet, vec!())
            },
        })
    }

    fn take_until_count<'a>(reader: &'a mut BitsReader, count: usize) -> Vec<PacketNode> {
        let mut v = Vec::with_capacity(count);
        for _ in 0..count {
            v.push(PacketNode::from_bits(reader).unwrap());
        }
        v
    }

    fn take_until_bits<'a>(reader: &'a mut BitsReader, len: BitsCount) -> Vec<PacketNode> {
        let end = reader.bits_pos_add(len);
        let mut packets = vec!();

        loop {
            let node = match PacketNode::from_bits(reader) {
                Some(p) => p,
                None => { break }
            };
            packets.push(node);

            if reader.bits_pos() >= end {
                break;
            }
        }

        packets
    }

    pub fn version_sum(&self) -> u32 {
        self.1
            .iter()
            .map(|n| n.version_sum())
            .sum::<u32>() + self.0.version() as u32
    }
}

impl fmt::Debug for PacketNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Packet::Literal(_, _) => write!(f, "{:?}", self.0),
            Packet::Operator(_, _, _) => write!(f, "{:?} {:?}", self.0, self.1),
        }
    }
}

fn bytes_from_hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn main() {
    let stdin = io::stdin();
    let line = stdin.lock().lines().next().unwrap().unwrap();

    let data = bytes_from_hex(&line);

    let root = PacketNode::from_bits(&mut data.read_bits()).unwrap();
    println!("{:?}", &root);

    /*
    for packet in data.read_bits() {
        print!("{:?} ", packet);
        sum += packet.version();
    }
    */
    println!();
    println!("sum: {}", &root.version_sum());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reader_increments() {
        let d = [0u8; 10];
        let mut reader = d.read_bits();

        assert_eq!(reader.n, 0);
        assert_eq!(reader.b, 0);

        reader += 11;
        assert_eq!(reader.n, 1);
        assert_eq!(reader.b, 3);
    }

    #[test]
    fn reader_reads() {
        let mut buf = [0; 10];
        let src = [0, 1, 2, 3, 4];
        let mut reader = src.read_bits();
        reader += 8;

        let c = reader.read_into(&mut buf);

        assert_eq!(c, 4);
        assert_eq!(buf[0..4], src[1..5]);
    }

    #[test]
    fn reader_reads_bits_offset() {
        let mut buf = [0; 10];
        let src = [0b_0000_0001, 0b_1001_0000, 0b0010_0000, 0b_0011_0000, 0b_0100_0000];
        let mut reader = src.read_bits();
        reader += 4;

        let c = reader.read_into(&mut buf);

        assert_eq!(c, 4);
        assert_eq!(buf[0..4], [0b_0001_1001, 2, 3, 4]);
    }

    #[test]
    fn parse_literal_sample() {
        let input = bytes_from_hex("D2FE28");
        let mut reader = input.read_bits();

        let (header, _) = reader.read_header().unwrap();
        assert!(header.is_literal());
        let (v, _) = reader.read_literal();
        assert_eq!(v, 0b_0111_1110_0101);
    }

    #[test]
    fn pass_test1() {
        let input = bytes_from_hex("8A004A801A8002F478");
        let packet = PacketNode::from_bits(&mut input.read_bits()).unwrap();
        let sum = packet.version_sum();

        assert_eq!(sum, 16);
    }

    #[test]
    fn pass_test2() {
        let input = bytes_from_hex("620080001611562C8802118E34");
        let packet = PacketNode::from_bits(&mut input.read_bits()).unwrap();
        let sum = packet.version_sum();

        assert_eq!(sum, 12);
    }

    #[test]
    fn pass_test3() {
        let input = bytes_from_hex("C0015000016115A2E0802F182340");
        let packet = PacketNode::from_bits(&mut input.read_bits()).unwrap();
        let sum = packet.version_sum();

        assert_eq!(sum, 23);
    }

    #[test]
    fn pass_test4() {
        let input = bytes_from_hex("A0016C880162017C3686B18A3D4780");
        let packet = PacketNode::from_bits(&mut input.read_bits()).unwrap();
        let sum = packet.version_sum();

        assert_eq!(sum, 31);
    }
}