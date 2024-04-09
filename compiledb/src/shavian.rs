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

pub fn is_shavian(ch: char) -> bool {
    static ALPHABET: [char; 48] = [
        '\u{10450}', /* 𐑐 */ '\u{10451}', /* 𐑑 */ '\u{10452}', /* 𐑒 */
        '\u{10453}', /* 𐑓 */ '\u{10454}', /* 𐑔 */ '\u{10455}', /* 𐑕 */
        '\u{10456}', /* 𐑖 */ '\u{10457}', /* 𐑗 */ '\u{10458}', /* 𐑘 */
        '\u{10459}', /* 𐑙 */ '\u{1045a}', /* 𐑚 */ '\u{1045b}', /* 𐑛 */
        '\u{1045c}', /* 𐑜 */ '\u{1045d}', /* 𐑝 */ '\u{1045e}', /* 𐑞 */
        '\u{1045f}', /* 𐑟 */ '\u{10460}', /* 𐑠 */ '\u{10461}', /* 𐑡 */
        '\u{10462}', /* 𐑢 */ '\u{10463}', /* 𐑣 */ '\u{10464}', /* 𐑤 */
        '\u{10465}', /* 𐑥 */ '\u{10466}', /* 𐑦 */ '\u{10467}', /* 𐑧 */
        '\u{10468}', /* 𐑨 */ '\u{10469}', /* 𐑩 */ '\u{1046a}', /* 𐑪 */
        '\u{1046b}', /* 𐑫 */ '\u{1046c}', /* 𐑬 */ '\u{1046d}', /* 𐑭 */
        '\u{1046e}', /* 𐑮 */ '\u{1046f}', /* 𐑯 */ '\u{10470}', /* 𐑰 */
        '\u{10471}', /* 𐑱 */ '\u{10472}', /* 𐑲 */ '\u{10473}', /* 𐑳 */
        '\u{10474}', /* 𐑴 */ '\u{10475}', /* 𐑵 */ '\u{10476}', /* 𐑶 */
        '\u{10477}', /* 𐑷 */ '\u{10478}', /* 𐑸 */ '\u{10479}', /* 𐑹 */
        '\u{1047a}', /* 𐑺 */ '\u{1047b}', /* 𐑻 */ '\u{1047c}', /* 𐑼 */
        '\u{1047d}', /* 𐑽 */ '\u{1047e}', /* 𐑾 */ '\u{1047f}', /* 𐑿 */
    ];

    ALPHABET.binary_search(&ch).is_ok()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn alphabet() {
        assert!(
            "𐑐𐑑𐑒𐑓𐑔𐑕𐑖𐑗𐑘𐑙𐑚𐑛𐑜𐑝𐑞𐑟𐑠𐑡𐑢𐑣𐑤𐑥𐑦𐑧𐑨𐑩𐑪𐑫𐑬𐑭𐑮𐑯𐑰𐑱𐑲𐑳𐑴𐑵𐑶𐑷𐑸𐑹𐑺𐑻𐑼𐑽𐑾𐑿"
                .chars()
                .all(|ch| is_shavian(ch))
        );

        assert!(!is_shavian(' '));
        assert!(!is_shavian('a'));
    }
}
