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

use std::io::Write;

pub struct BitWriter<'a, W: Write> {
    queue: u8,
    queue_length: u8,
    output: &'a mut W,
}

impl<'a, W: Write> BitWriter<'a, W> {
    pub fn new(output: &'a mut W) -> BitWriter<'a, W> {
        BitWriter {
            queue: 0,
            queue_length: 0,
            output,
        }
    }

    pub fn add_bits(
        &mut self,
        mut bits: u32,
        mut n_bits: u8,
    ) -> std::io::Result<()> {
        if n_bits <= 0 {
            return Ok(());
        }

        if self.queue_length > 0 {
            let align_bits = 8 - self.queue_length;

            if align_bits > n_bits {
                self.queue |= (bits << self.queue_length) as u8;
                self.queue_length += n_bits;
                return Ok(());
            }

            self.output.write_all(
                &[
                    self.queue |
                    ((bits as u8
                      & ((1 << align_bits) - 1)) << self.queue_length)
                ],
            )?;

            n_bits -= align_bits;
            bits >>= align_bits;
        }

        let n_bytes = n_bits / 8;

        let le = bits.to_le_bytes();

        self.output.write_all(&le[0..n_bytes as usize])?;

        n_bits -= n_bytes * 8;
        bits >>= n_bytes * 8;

        self.queue_length = n_bits;
        self.queue = bits as u8 & ((1 << n_bits) - 1);

        Ok(())
    }

    pub fn done(self) -> std::io::Result<()> {
        if self.queue_length > 0 {
            self.output.write_all(&[self.queue])?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    static TEST_VALUES: [u32; 5] = [
        0, u32::MAX, 0x10101010, 0x05050505, 0x87654321
    ];

    #[test]
    fn one_bit_at_a_time() {
        for &value in TEST_VALUES.iter() {
            let mut value = value;
            let mut result = Vec::new();
            let expected = value.to_le_bytes();
            let mut writer = BitWriter::new(&mut result);

            for _ in 0..u32::BITS {
                writer.add_bits(value, 1).unwrap();
                value >>= 1;
            }

            writer.done().unwrap();

            assert_eq!(&expected[..], &result);
        }
    }

    #[test]
    fn bytes_in_the_middle() {
        for &value in TEST_VALUES.iter() {
            let mut result = Vec::new();
            let expected = value.to_le_bytes();
            let mut writer = BitWriter::new(&mut result);

            writer.add_bits(value, 1).unwrap();
            writer.add_bits(value >> 1, 30).unwrap();
            writer.add_bits(value >> 31, 1).unwrap();

            writer.done().unwrap();

            assert_eq!(&expected[..], &result);
        }
    }

    #[test]
    fn each_byte() {
        for &value in TEST_VALUES.iter() {
            let mut result = Vec::new();
            let expected = value.to_le_bytes();
            let mut writer = BitWriter::new(&mut result);

            writer.add_bits(value, 8).unwrap();
            writer.add_bits(value >> 8, 8).unwrap();
            writer.add_bits(value >> 16, 8).unwrap();
            writer.add_bits(value >> 24, 8).unwrap();

            writer.done().unwrap();

            assert_eq!(&expected[..], &result);
        }
    }

    #[test]
    fn dangling_data() {
        let mut result = Vec::new();
        let mut writer = BitWriter::new(&mut result);

        writer.add_bits(0x1e, 8).unwrap();
        writer.add_bits(0x0265, 10).unwrap();
        writer.done().unwrap();

        assert_eq!(&result, &[0x1e, 0x65, 0x02]);
    }
}
