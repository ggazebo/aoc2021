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
pub enum OperatorId {
    Sum,
    Product,
    Min,
    Max,
    GreaterThan,
    LessThan,
    Equal,
}

impl TryFrom<u8> for OperatorId {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => OperatorId::Sum,
            1 => OperatorId::Product,
            2 => OperatorId::Min,
            3 => OperatorId::Max,
            5 => OperatorId::GreaterThan,
            6 => OperatorId::LessThan,
            7 => OperatorId::Equal,
            _ => return Err("invalid operator id"),
        })
    }
}

impl fmt::Debug for OperatorId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            OperatorId::Sum => "SUM",
            OperatorId::Product => "PROD",
            OperatorId::Min => "MIN",
            OperatorId::Max => "MAX",
            OperatorId::GreaterThan => "GT",
            OperatorId::LessThan => "LT",
            OperatorId::Equal => "EQ",
        })
    }
}

#[derive(Clone, Copy)]
pub enum PacketData {
    Literal(Header, LiteralValue),
    Operator(Header, OperatorId, LengthTypeId),
}

impl PacketData {
    pub fn version(&self) -> PacketVersion {
        match self {
            PacketData::Literal(h, _) => h,
            PacketData::Operator(h, _ , _) => h,
        }.version()
    }
}

impl fmt::Debug for PacketData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PacketData::Literal(h, v) => write!(f, "{}({})", v, h.version()),
            PacketData::Operator(_, id, l) => {
                let (prefix, n) = match l {
                    LengthTypeId::Bits(b) => ("b", b),
                    LengthTypeId::Count(c) => ("", c),
                };
                write!(f, "{:?}`{}{}({})", id, prefix, n, self.version())
            },
        }
    }
}

type PacketVersion = u8;
type BitsCount = usize;
type LiteralValue = u64;

#[derive(Clone)]
pub struct BitsReader<'a> {
    stream: &'a [u8],
    i: usize,
}

impl<'a> BitsReader<'a> {
    pub fn bits_pos(&self) -> usize {
        //self.n * 8 + self.b
        self.i
    }

    pub fn bits_pos_add(&self, bits: usize) -> usize {
        //self.bits_pos() + bits
        self.i + bits
    }

    pub fn read_header(&mut self) -> Option<(Header, BitsCount)> {
        let mut b = [0];
        match self.read_to(&mut b, 6) {
            None => None,
            Some(d) => {
                let b = d[0];
                let version = b >> 3;
                let header = match b & 0b_0000_0111 {
                    4 => Header::Literal(version),
                    n => Header::Operator(version, n.try_into().unwrap()),
                };
                Some((header, 6))
            }
        }
    }

    fn read_packet_count(&mut self) -> LengthTypeId {
        let mut buf = [0;2];
        let b = self.read_to(&mut buf, 12).unwrap();
        if b[0] < 0b_0000_1000 {
            let l = (((b[0] & 0b_0000_0111) as usize) << 12) | ((b[1] as usize) << 4) as usize;
            let b = self.read_to(&mut buf, 4).unwrap();
            let l = l | (b[0] & 0x0f) as usize;
            LengthTypeId::Bits(l)
        } else {
            let c = (((b[0] & 0b_0000_0111) as usize) << 8) | b[1] as usize;
            LengthTypeId::Count(c)
        }
    }

    pub fn read_literal(&mut self) -> LiteralValue {
        let mut v = 0;
        let mut buf = [0];
        loop {
            let b = self.read_to(&mut buf, 5).unwrap();
            let byte = b[0];
            println!("lit: {}", byte);
            v = (v << 4) | (byte & 0b_0000_1111) as LiteralValue;
            buf[0] = 0;
            if byte < 0b_1_0000 {
                break;
            }
        }
        v
    }

    /*
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
    */

    fn read_to<'buf>(&mut self, buf: &'buf mut [u8], len: BitsCount) -> Option<&'buf [u8]> {
        if (self.i + len) / 8 >= self.stream.len() {
            return None;
        }
        let rot = ((8 - (self.i + len) % 8) % 8) as u32;
        let b_start = self.i / 8;
        let bytes = (self.i + len + rot as usize) / 8 - b_start;

        println!("read_to({}): i={} {}..+{} >>{}", len, self.i, b_start, bytes, rot);

        if rot == 0 {
            buf[0..bytes].copy_from_slice(&self.stream[b_start..b_start+bytes]);
            buf[0] &= 0xff >> (8 - len % 8);
        } else {
            assert!(len <= 16, "can't read more than 16 bits at a time");

            let src = &self.stream[b_start..b_start+bytes];
            let value = (u32::from_be_bytes(match src.len() {
                1 => [0, 0, 0, src[0]],
                2 => [0, 0, src[0], src[1]],
                3 => [0, src[0], src[1], src[2]],
                _ => panic!(),
            }) >> rot) & (0xffffffff >> (32 - len));

            println!("{:024b}", value);

            let v_bytes = value.to_be_bytes();
            match cmp::max(0, len / 8) {
                0 => { buf[0] = v_bytes[3]; },
                1 => { buf[0] = v_bytes[2]; buf[1] = v_bytes[3]; },
                _ => panic!(),
            }
            /*
            let src = &self.stream[b_start..b_start+bytes];
            //let mask = ((0xff >> (self.i % 8)) as u8).rotate_right(rot);
            let leading_bits = match (len + 32 - (self.i % 8)) % 8 { 0 => 8, n => n };
            let mask = ((0xff << (8 - leading_bits)) as u8).rotate_right((self.i % 8) as u32);
            //buf[0] = src[0].rotate_right(rot) & mask;
            buf[0] = (src[0] & mask).rotate_right(rot);
            println!("0b_{:08b} -> 0b_{:08b} {}bits mask=0b_{:08b}", src[0], buf[0], leading_bits, mask);
            for i in 1..bytes {
                buf[i-1] |= src[i] >> rot;
                println!("{}: 0b_{:08b} 0b_{:08b} -> 0b_{:08b}", i, src[i-1], src[i], buf[i-1]);
            }
            */
        }

        self.i += len;
        Some(&buf[0..=(len / 8)])
    }
}

impl<'a> ops::AddAssign<BitsCount> for BitsReader<'a> {
    fn add_assign(&mut self, inc: BitsCount) {
        self.i += inc
    }
}

impl<'a> Iterator for BitsReader<'a> {
    type Item = PacketData;

    fn next(&mut self) -> Option<PacketData> {
        let header = match self.read_header() {
            Some((h, _)) => h,
            None => return None,
        };

        match header {
            Header::Literal(_) => {
                let v = self.read_literal();
                Some(PacketData::Literal(header, v))
            }
            Header::Operator(_, id) => {
                let length_type = self.read_packet_count();
                Some(PacketData::Operator(header, id, length_type))
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
        BitsReader { stream: self.as_ref(), i: 0 }
    }
}

pub struct Packet(PacketData, Vec<Packet>);

impl Packet {
    pub fn value(&self) -> LiteralValue {
        let values = &mut self.1.iter().map(|p| p.value());

        match self.0 {
            PacketData::Literal(_, v) => v,
            PacketData::Operator(_, OperatorId::Sum, _) => values.sum(),
            PacketData::Operator(_, OperatorId::Product, _) => values.product(),
            PacketData::Operator(_, OperatorId::Min, _) => values.min().unwrap(),
            PacketData::Operator(_, OperatorId::Max, _) => values.max().unwrap(),
            PacketData::Operator(_, OperatorId::GreaterThan, _) => {
                let a = values.next().unwrap();
                let b = values.next().unwrap();
                if a > b { 1 } else { 0 }
            },
            PacketData::Operator(_, OperatorId::LessThan, _) => {
                let a = values.next().unwrap();
                let b = values.next().unwrap();
                if a < b { 1 } else { 0 }
            },
            PacketData::Operator(_, OperatorId::Equal, _) => {
                let a = values.next().unwrap();
                let b = values.next().unwrap();
                if a == b { 1 } else { 0 }
            },
        }

    }

    pub fn from_bits<'a>(reader: &'a mut BitsReader) -> Option<Packet> {
        let packet = match reader.next() {
            Some(p) => p,
            None => return None,
        };

        println!("{:?}", &packet);

        Some(match packet {
            PacketData::Operator(_, _, LengthTypeId::Count(len)) => {
                let nodes = Packet::take_until_count(reader, len);
                Packet(packet, nodes)
            },
            PacketData::Operator(_, _, LengthTypeId::Bits(bits)) => {
                Packet(packet, Packet::take_until_bits(reader, bits))
            },
            _ => {
                Packet(packet, vec!())
            },
        })
    }

    fn take_until_count<'a>(reader: &'a mut BitsReader, count: usize) -> Vec<Packet> {
        let mut v = Vec::with_capacity(count);
        for _ in 0..count {
            v.push(Packet::from_bits(reader).unwrap());
        }
        v
    }

    fn take_until_bits<'a>(reader: &'a mut BitsReader, len: BitsCount) -> Vec<Packet> {
        let end = reader.bits_pos_add(len);
        let mut packets = vec!();

        loop {
            let node = match Packet::from_bits(reader) {
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

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            PacketData::Literal(_, _) => write!(f, "{:?}", self.0),
            PacketData::Operator(_, _, _) => write!(f, "{:?} {:?}", self.0, self.1),
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

    let root = Packet::from_bits(&mut data.read_bits()).unwrap();
    println!("{:?}", &root);

    /*
    for packet in data.read_bits() {
        print!("{:?} ", packet);
        sum += packet.version();
    }
    */
    println!();
    println!("sum: {}", &root.version_sum());
    println!("value: {}", &root.value());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reader_increments() {
        let d = [0u8; 10];
        let mut reader = d.read_bits();

        assert_eq!(reader.i, 0);

        reader += 11;
        assert_eq!(reader.i, 11);
    }

    /*
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
    */

    /*
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
    */

    #[test]
    fn parse_literal_sample() {
        let input = bytes_from_hex("D2FE28");
        let packet = Packet::from_bits(&mut input.read_bits()).unwrap();

        match packet.0 {
            PacketData::Literal(_, v) => assert_eq!(v, 0b_0111_1110_0101),
            _ => assert!(false, "failed to parse value"),
        };
    }

    #[test]
    fn parse_op_sample_1() {
        let input = bytes_from_hex("38006F45291200");
        let packet = Packet::from_bits(&mut input.read_bits()).unwrap();

        match packet.0 {
            PacketData::Operator(_, _, LengthTypeId::Bits(n)) => assert_eq!(n, 27),
            _ => assert!(false, "failed to parse value"),
        };
    }

    #[test]
    fn parse_op_sample_2() {
        let input = bytes_from_hex("EE00D40C823060");
        let packet = Packet::from_bits(&mut input.read_bits()).unwrap();

        match packet.0 {
            PacketData::Operator(_, _, LengthTypeId::Count(n)) => assert_eq!(n, 3),
            _ => assert!(false, "failed to parse value"),
        };
    }

    #[test]
    fn pass_test1() {
        let input = bytes_from_hex("8A004A801A8002F478");
        let packet = Packet::from_bits(&mut input.read_bits()).unwrap();
        let sum = packet.version_sum();

        assert_eq!(sum, 16);
    }

    #[test]
    fn pass_test2() {
        let input = bytes_from_hex("620080001611562C8802118E34");
        let packet = Packet::from_bits(&mut input.read_bits()).unwrap();
        let sum = packet.version_sum();

        assert_eq!(sum, 12);
    }

    #[test]
    fn pass_test3() {
        let input = bytes_from_hex("C0015000016115A2E0802F182340");
        let packet = Packet::from_bits(&mut input.read_bits()).unwrap();
        let sum = packet.version_sum();

        assert_eq!(sum, 23);
    }

    #[test]
    fn pass_test4() {
        let input = bytes_from_hex("A0016C880162017C3686B18A3D4780");
        let packet = Packet::from_bits(&mut input.read_bits()).unwrap();
        let sum = packet.version_sum();

        assert_eq!(sum, 31);
    }
}