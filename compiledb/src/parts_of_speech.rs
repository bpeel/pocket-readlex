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

mod pos_table;

pub static NAMES: [&'static str; 39] = [
    "AJ0", "AJC", "AJS", "AT0", "AV0", "AVP", "AVQ", "CJC", "CJS",
    "CJT", "CRD", "DPS", "DT0", "DTQ", "EX0", "ITJ", "NN0", "NN1",
    "NN2", "NP0", "ORD", "PNI", "PNP", "PNQ", "PNX", "POS", "PRE",
    "PRF", "PRP", "TO0", "UNC", "VM0", "VVB", "VVD", "VVG", "VVN",
    "VVZ", "XX0", "ZZ0",
];

pub use pos_table::N_POS;

pub const PNP: u8 = 22;
pub const NP0: u8 = 19;
pub const START_OF_SENTENCE: u8 = N_POS as u8 - 1;

pub fn pair_priority(left: u8, right: u8) -> u8 {
    pos_table::PAIR_PRIORITIES[left as usize * N_POS + right as usize]
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn constants() {
        assert_eq!(NAMES[NP0 as usize], "NP0");
        assert_eq!(NAMES[PNP as usize], "PNP");
    }
}
