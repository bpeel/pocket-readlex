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
use super::bit_writer::BitWriter;

// The trie on disk is stored as a list of trie nodes. A trie node is
// stored as the following parts:
//
// • A number stored with a variable length. Each byte represents 7
//   more bits of data where the first byte has the least-significant
//   bits. The most significant bit of each byte indicates whether
//   more bits follow. The first bit of the resulting number indicates
//   whether this node has children. The rest of the bits are a byte
//   offset to the next sibling, or zero if there are no more
//   siblings. The offset is calculated from the point between this
//   number and the following character data.
// • 1-4 bytes of UTF-8 encoded data to represent the character of
//   this node.
//
// The first entry in the list is the root node. Its character value
// should be ignored.
//
// If the character is '\0' then it means the letters in the chain of
// parents leading up to this node are a valid word.
//
// After the 0 is a variable-length list of translations of the word.
// Each translation is stored as follows:
//
// • A byte where the seven lowest bits represent the data passed to
//   add_word(). The highest bit is set if another translation follows
//   this one.
//
// • A series of variable-bit-length numbers representing the path to
//   take to navigate the tree to find the translation. Each number
//   represents the number of siblings to skip across before going to
//   a child node. The number of bits used for each number is the
//   minimum number of bits needed to store the number of children of
//   each node minus one (ie, the maximum sibling index value). This
//   can be zero bits if there is only one child. The list of numbers
//   ends when a terminating node is reached. That means the only way
//   to know the length of this list is by iterating through the trie.
//   After the numbers the data is padded with zeroes to the nearest
//   byte.

use std::num::NonZeroUsize;

struct Translation {
    payload_byte: u8,
    value: String,
}

struct Terminator {
    translations: Vec<Translation>,
    payload: Vec<u8>,
}

impl Terminator {
    fn new() -> Terminator {
        Terminator {
            translations: Vec::new(),
            payload: Vec::new(),
        }
    }
}

enum NodeData {
    Char(char),
    Terminator(Terminator),
}

impl NodeData {
    fn ch(&self) -> char {
        match self {
            NodeData::Char(ch) => *ch,
            NodeData::Terminator(_) => '\0',
        }
    }
}

struct Node {
    data: NodeData,

    // The size in bytes of this node including all of its children
    // and next siblings. Calculated in a separate pass.
    size: usize,

    // Index of the first child if there is one
    first_child: Option<NonZeroUsize>,
    // Index of the next sibling if there is one
    next_sibling: Option<NonZeroUsize>,
}

impl Node {
    fn new(data: NodeData) -> Node {
        let size = data.ch().len_utf8();

        Node {
            data,
            size,
            first_child: None,
            next_sibling: None,
        }
    }
}

#[derive(PartialEq, Eq)]
enum NextNode {
    FirstChild,
    NextSibling,
    Backtrack,
}

struct StackEntry {
    node: usize,
    // Next node to visit
    next_node: NextNode,
}

impl StackEntry {
    fn new(node: usize) -> StackEntry {
        StackEntry {
            node,
            next_node: NextNode::FirstChild,
        }
    }
}

pub struct TrieBuilder {
    nodes: Vec<Node>,
}

impl TrieBuilder {
    pub fn new() -> TrieBuilder {
        TrieBuilder {
            nodes: vec![Node::new(NodeData::Char('*'))],
        }
    }

    fn add_word_one_direction(
        &mut self,
        word: &str,
        translation: String,
        payload_byte: u8
    ) {
        assert!(payload_byte < 0x80);

        let mut node = 0;

        for data in word.chars()
            .map(NodeData::Char)
            .chain(std::iter::once(NodeData::Terminator(Terminator::new())))
        {
            node = 'find_node: {
                let mut child = self.nodes[node].first_child;

                while let Some(this_child) = child {
                    if self.nodes[this_child.get()].data.ch() == data.ch() {
                        break 'find_node this_child.get();
                    }

                    child = self.nodes[this_child.get()].next_sibling;
                }

                let new_node_pos = self.nodes.len();
                let mut new_node = Node::new(data);

                let old_node = &mut self.nodes[node];

                new_node.next_sibling = old_node.first_child;
                old_node.first_child = NonZeroUsize::new(new_node_pos);
                // The nodes list is never empty, so the new_node_pos
                // shouldn’t be zero
                assert!(old_node.first_child.is_some());

                self.nodes.push(new_node);

                new_node_pos
            };

            if let NodeData::Terminator(ref mut terminator) =
                &mut self.nodes[node].data
            {
                terminator.translations.push(Translation {
                    payload_byte,
                    value: translation,
                });
                break;
            }
        }
    }

    pub fn add_word(
        &mut self,
        word: &str,
        translation: &str,
        payload_byte: u8,
    ) {
        self.add_word_one_direction(
            word,
            translation.to_string(),
            payload_byte,
        );
        self.add_word_one_direction(
            translation,
            word.to_string(),
            payload_byte,
        );
    }

    fn sort_children_by_character(
        &mut self,
        parent: usize,
        child_indices: &mut Vec<usize>,
    ) {
        child_indices.clear();

        let mut child_index = self.nodes[parent].first_child;

        // Gather up indices of all the children
        while let Some(child) = child_index {
            child_indices.push(child.get());
            child_index = self.nodes[child.get()].next_sibling;
        }

        // Sort by character
        child_indices.sort_by_key(|&child_index| {
            self.nodes[child_index].data.ch()
        });

        self.nodes[parent].first_child = None;

        // Put the list in the right order
        for &child in child_indices.iter().rev() {
            let first_child = self.nodes[parent].first_child;
            self.nodes[child].next_sibling = first_child;
            self.nodes[parent].first_child = NonZeroUsize::new(child);
            assert!(self.nodes[parent].first_child.is_some());
        }
    }

    fn sort_all_children_by_character(&mut self) {
        let mut child_indices = Vec::<usize>::new();

        for i in 0..self.nodes.len() {
            self.sort_children_by_character(i, &mut child_indices);
        }
    }

    fn next_node(&self, entry: &mut StackEntry) -> Option<usize> {
        loop {
            let next_node = match entry.next_node {
                NextNode::Backtrack => {
                    break None;
                },
                NextNode::FirstChild => {
                    entry.next_node = NextNode::NextSibling;
                    self.nodes[entry.node].first_child
                },
                NextNode::NextSibling => {
                    entry.next_node = NextNode::Backtrack;
                    self.nodes[entry.node].next_sibling
                },
            };

            if let Some(next_node) = next_node {
                break Some(next_node.get());
            }
        }
    }

    fn calculate_size(&mut self) {
        let mut stack = vec![StackEntry::new(0)];

        while let Some(mut entry) = stack.pop() {
            match self.next_node(&mut entry) {
                Some(next_child) => {
                    stack.push(entry);
                    stack.push(StackEntry::new(next_child));
                },
                None => {
                    let node_data_number = self.node_data_number(entry.node);

                    self.nodes[entry.node].size +=
                        n_bytes_for_size(node_data_number);

                    if let NodeData::Terminator(ref terminator) =
                        self.nodes[entry.node].data
                    {
                        let size = terminator.payload.len();
                        self.nodes[entry.node].size += size;
                    }

                    if let Some(&StackEntry { node: parent, .. })
                        = stack.last()
                    {
                        let child_size = self.nodes[entry.node].size;
                        self.nodes[parent].size += child_size;
                    }
                },
            };
        }
    }

    fn write_path<W: Write>(
        &self,
        word: &str,
        writer: &mut BitWriter<W>,
    ) -> std::io::Result<usize> {
        let mut node = 0;
        let mut bits_written = 0;

        for ch in word.chars().chain(std::iter::once('\0')) {
            let n_children = self.n_children(node);

            assert!(n_children > 0);

            let n_bits = u32::BITS - (n_children as u32 - 1).leading_zeros();

            node = self.nodes[node].first_child.unwrap().get();
            let mut skips = 0;

            loop {
                if self.nodes[node].data.ch() == ch {
                    break;
                }

                node = self.nodes[node].next_sibling.unwrap().get();
                skips += 1;
            }

            writer.add_bits(skips, n_bits as u8)?;
            bits_written += n_bits as usize;
        }

        Ok((bits_written + 7) / 8)
    }

    fn calculate_payload(&self, terminator: &Terminator) -> Vec<u8> {
        let mut payload = Vec::new();

        for (i, translation) in terminator.translations.iter().enumerate() {
            let mut payload_byte = translation.payload_byte;

            if i + 1 < terminator.translations.len() {
                payload_byte |= 1 << 7;
            }

            payload.push(payload_byte);

            let mut writer = BitWriter::new(&mut payload);
            self.write_path(&translation.value, &mut writer).unwrap();
            writer.done().unwrap();
        }

        payload
    }

    fn calculate_payloads(&mut self) {
        let mut stack = vec![StackEntry::new(0)];

        while let Some(mut entry) = stack.pop() {
            if let Some(next_child) = self.next_node(&mut entry) {
                stack.push(entry);
                stack.push(StackEntry::new(next_child));

                if let NodeData::Terminator(ref terminator) =
                    self.nodes[next_child].data
                {
                    let payload = self.calculate_payload(terminator);
                    if let NodeData::Terminator(ref mut terminator) =
                        self.nodes[next_child].data
                    {
                        terminator.payload = payload;
                    }
                }
            }
        }
    }

    fn node_data_number(&self, index: usize) -> usize {
        let node = &self.nodes[index];

        let sibling_offset = match node.next_sibling {
            Some(_) => {
                let data_size = match node.data {
                    NodeData::Terminator(ref terminator) => {
                        terminator.payload.len()
                    },
                    NodeData::Char(_) => {
                        let child_index = node.first_child.unwrap().get();
                        self.nodes[child_index].size
                    },
                };
                node.data.ch().len_utf8() + data_size
            },
            None => 0,
        };

        let mut data_number = sibling_offset << 1;

        if node.first_child.is_some() {
            data_number |= 1;
        }

        data_number
    }

    pub fn into_dictionary(
        mut self,
        output: &mut impl Write,
    ) -> std::io::Result<()> {
        // Sort all the children of each node by character so that
        // it’s easier to compare them.
        self.sort_all_children_by_character();

        self.calculate_payloads();

        // Calculate the size of each node in the trie as if it was a
        // binary tree so that we can work out the offsets.
        self.calculate_size();

        self.write_nodes(output)
    }

    fn write_node(
        &self,
        index: usize,
        output: &mut impl Write,
    ) -> std::io::Result<()> {
        let data_number = self.node_data_number(index);

        let node = &self.nodes[index];

        write_offset(data_number, output)?;

        let mut ch_utf8 = [0u8; 4];

        output.write_all(node.data.ch().encode_utf8(&mut ch_utf8).as_bytes())?;

        if let NodeData::Terminator(ref terminator) = node.data {
            output.write_all(&terminator.payload)?;
        }

        Ok(())
    }

    fn n_children(&self, parent: usize) -> usize {
        let mut count = 0;
        let mut node = self.nodes[parent].first_child;

        while let Some(this_node) = node {
            count += 1;
            node = self.nodes[this_node.get()].next_sibling;
        }

        count
    }

    fn write_nodes(
        &self,
        output: &mut impl Write,
    ) -> std::io::Result<()> {
        let mut stack = vec![StackEntry::new(0)];

        while let Some(mut entry) = stack.pop() {
            if entry.next_node == NextNode::FirstChild {
                self.write_node(entry.node, output)?;
            }

            if let Some(next_child) = self.next_node(&mut entry) {
                stack.push(entry);
                stack.push(StackEntry::new(next_child));
            }
        }

        Ok(())
    }
}

fn n_bytes_for_size(size: usize) -> usize {
    // Count the number of bits needed to store this number
    let n_bits = (usize::BITS - size.leading_zeros()).max(1);
    // We can store 7 of the bits per byte
    (n_bits as usize + 6) / 7
}

fn write_offset(
    mut offset: usize,
    output: &mut impl Write,
) -> std::io::Result<()> {
    let mut buf = [0u8; (usize::BITS as usize + 6) / 7];
    let mut length = 0;

    loop {
        buf[length] = offset as u8 & ((1 << 7) - 1);

        offset >>= 7;

        if offset == 0 {
            length += 1;
            break;
        }

        buf[length] |= 1 << 7;
        length += 1;
    }

    output.write_all(&buf[0..length])
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_n_bytes_for_size() {
        assert_eq!(n_bytes_for_size(0), 1);
        assert_eq!(n_bytes_for_size(1), 1);
        assert_eq!(n_bytes_for_size(0x7f), 1);
        assert_eq!(n_bytes_for_size(0x80), 2);
        assert_eq!(n_bytes_for_size(u32::MAX as usize), 5);
    }

    #[test]
    fn test_write_offset() {
        fn offset_to_vec(offset: usize) -> Vec<u8> {
            let mut result = Vec::new();

            write_offset(offset, &mut result).unwrap();

            result
        }

        assert_eq!(&offset_to_vec(0), &[0]);
        assert_eq!(&offset_to_vec(1), &[1]);
        assert_eq!(&offset_to_vec(0x7f), &[0x7f]);
        assert_eq!(&offset_to_vec(0x80), &[0x80, 0x01]);
        assert_eq!(
            &offset_to_vec(u32::MAX as usize),
            &[0xff, 0xff, 0xff, 0xff, 0x0f],
        );
    }

    #[test]
    fn duplicates() {
        let mut builder = TrieBuilder::new();

        builder.add_word("abc", "bbc", 0x7e);

        let mut dictionary = Vec::<u8>::new();

        builder.into_dictionary(&mut dictionary).unwrap();

        // There should be 9 nodes because the “bc” endings shouldn’t
        // be combined into one. Each node takes up 2 bytes in this
        // small example, plus 2 bytes for each terminator.
        assert_eq!(dictionary.len(), 9 * 2 + 2 * 2);

        assert_eq!(
            &dictionary,
            &[
                1, b'*',
                19, b'a',
                1, b'b',
                1, b'c',
                0, b'\0',
                0x7e, 1,
                1, b'b',
                1, b'b',
                1, b'c',
                0, b'\0',
                0x7e, 0,
            ],
        );
    }
}
