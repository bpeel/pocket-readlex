// Pocket ReadLex – An offline app for ReadLex
// Copyright (C) 2024  Neil Roberts
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

#[derive(Clone)]
pub struct BitReader<'a> {
    slice: &'a [u8],
    queue: u8,
    queue_length: u8,
    bytes_consumed: usize,
}

impl<'a> BitReader<'a> {
    pub fn new(slice: &'a [u8]) -> BitReader<'a> {
        BitReader {
            slice,
            queue: 0,
            queue_length: 0,
            bytes_consumed: 0,
        }
    }

    fn read_byte(&mut self) -> Option<u8> {
        let &byte = self.slice.get(self.bytes_consumed)?;
        self.bytes_consumed += 1;
        Some(byte)
    }

    pub fn read_bits(&mut self, n_bits: u8) -> Option<u32> {
        assert!(n_bits as u32 <= u32::BITS);

        let mut got = n_bits.min(self.queue_length);

        let mut result = (self.queue as u32) & ((u8::MAX as u32) >> (8 - got));

        self.queue >>= got;
        self.queue_length -= got;

        while n_bits - got >= 8 {
            result |= (self.read_byte()? as u32) << got;
            got += 8;
        }

        let remainder = n_bits - got;

        if remainder > 0 {
            let byte = self.read_byte()?;
            self.queue_length = 8 - remainder;
            result |= ((byte & (u8::MAX >> self.queue_length)) as u32) << got;
            self.queue = byte >> remainder;
        }

        Some(result)
    }

    pub fn bytes_consumed(&self) -> usize {
        self.bytes_consumed
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn one_bit_at_a_time() {
        let magic = 0x8182f719u32;
        let bytes = magic.to_le_bytes();
        let mut reader = BitReader::new(&bytes[..]);
        let mut result = 0u32;

        for i in 0..u32::BITS {
            result |= reader.read_bits(1).unwrap() << i;
        }

        assert_eq!(result, magic);

        assert!(reader.read_bits(1).is_none());
        assert_eq!(reader.bytes_consumed(), u32::BITS as usize / 8);
    }

    #[test]
    fn one_byte_at_a_time() {
        let magic = [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0x3];
        let mut reader = BitReader::new(&magic[..]);

        for i in 0..magic.len() {
            assert_eq!(magic[i] as u32, reader.read_bits(8).unwrap());
            assert_eq!(reader.bytes_consumed(), i + 1);
        }

        assert!(reader.read_bits(1).is_none());
    }

    #[test]
    fn bytes_in_the_middle() {
        let magic = [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0x3];
        let mut reader = BitReader::new(&magic[..]);

        assert_eq!(reader.read_bits(4).unwrap(), 0x2);
        assert_eq!(reader.bytes_consumed(), 1);
        assert_eq!(reader.read_bits(32).unwrap(), 0xa7856341);
        assert_eq!(reader.bytes_consumed(), 5);
        assert_eq!(reader.read_bits(4).unwrap(), 0x9);
        assert_eq!(reader.bytes_consumed(), 5);
        assert_eq!(reader.read_bits(12).unwrap(), 0xebc);
        assert_eq!(reader.bytes_consumed(), 7);
        assert_eq!(reader.read_bits(12).unwrap(), 0x03d);
        assert_eq!(reader.bytes_consumed(), 8);
        assert!(reader.read_bits(1).is_none());
    }
}
