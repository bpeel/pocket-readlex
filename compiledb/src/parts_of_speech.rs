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

pub static NAMES: [&'static str; 40] = [
    "AJ0", "AJC", "AJS", "AT0", "AV0", "AVP", "AVQ", "CJC", "CJS",
    "CJT", "CRD", "DPS", "DT0", "DTQ", "EX0", "ITJ", "NN0", "NN1",
    "NN2", "NP0", "ORD", "PNI", "PNP", "PNQ", "PNX", "POS", "PRE",
    "PRF", "PRP", "TO0", "UNC", "VM0", "VVB", "VVD", "VVG", "VVI",
    "VVN", "VVZ", "XX0", "ZZ0",
];

pub const PNP: u8 = 22;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn constants() {
        assert_eq!(NAMES[PNP as usize], "PNP");
    }
}
