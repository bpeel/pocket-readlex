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

// The trie on disk is stored as a list of trie nodes. A trie node is
// stored as the following parts:
//
// • A byte offset to move to the next sibling, or zero if there is no
//   next sibling.
// • A byte offset to move to the first child, or zero if there are no
//   children of this node.
// • 1-6 bytes of UTF-8 encoded data to represent the character of
//   this node.
//
// The two byte offsets are always positive and count from the point
// between the offsets and the character data. The number is stored as
// a variable-length integer. Each byte contains the next
// most-significant 7 bits. The topmost bit of the byte determines
// whether there are more bits to follow.
//
// The first entry in the list is the root node. Its character value
// should be ignored.
//
// If the character is '\0' then it means the letters in the chain of
// parents leading up to this node are a valid word.

use std::num::NonZeroUsize;

struct Node {
    ch: char,

    // The size in bytes of this node including all of its children
    // and next siblings. Calculated in a separate pass.
    size: usize,

    // Index of the first child if there is one
    first_child: Option<NonZeroUsize>,
    // Index of the next sibling if there is one
    next_sibling: Option<NonZeroUsize>,
}

// Trie node info calculated on the fly
struct NodeInfo {
    child_offset: usize,
    sibling_offset: usize,
}

impl Node {
    fn new(ch: char) -> Node {
        Node {
            ch,
            size: ch.len_utf8(),
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
            nodes: vec![Node::new('*')],
        }
    }

    pub fn add_word(&mut self, word: &str) {
        let mut node = 0;

        for ch in word.chars().chain(std::iter::once('\0')) {
            node = 'find_node: {
                let mut child = self.nodes[node].first_child;

                while let Some(this_child) = child {
                    if self.nodes[this_child.get()].ch == ch {
                        break 'find_node this_child.get();
                    }

                    child = self.nodes[this_child.get()].next_sibling;
                }

                let new_node_pos = self.nodes.len();
                let mut new_node = Node::new(ch);

                let old_node = &mut self.nodes[node];

                new_node.next_sibling = old_node.first_child;
                old_node.first_child = NonZeroUsize::new(new_node_pos);
                // The nodes list is never empty, so the new_node_pos
                // shouldn’t be zero
                assert!(old_node.first_child.is_some());

                self.nodes.push(new_node);

                new_node_pos
            }
        }
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
        child_indices.sort_by_key(|&child_index| self.nodes[child_index].ch);

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
                    let node_info = self.node_info(entry.node);

                    self.nodes[entry.node].size +=
                        n_bytes_for_size(node_info.child_offset)
                        + n_bytes_for_size(node_info.sibling_offset);

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

    fn node_info(&self, index: usize) -> NodeInfo {
        let node = &self.nodes[index];

        let character_length = node.ch.len_utf8();

        let child_offset = match node.first_child {
            Some(_) => character_length,
            None => 0,
        };

        let sibling_offset = match node.next_sibling {
            Some(_) => {
                let child_size = match node.first_child {
                    Some(index) => self.nodes[index.get()].size,
                    None => 0,
                };
                character_length + child_size
            },
            None => 0,
        };

        NodeInfo { child_offset, sibling_offset }
    }

    pub fn into_dictionary(
        mut self,
        output: &mut impl Write,
    ) -> std::io::Result<()> {
        // Sort all the children of each node by character so that
        // it’s easier to compare them.
        self.sort_all_children_by_character();

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
        let info = self.node_info(index);

        let node = &self.nodes[index];

        write_offset(info.sibling_offset, output)?;
        write_offset(info.child_offset, output)?;

        let mut ch_utf8 = [0u8; 4];

        output.write_all(node.ch.encode_utf8(&mut ch_utf8).as_bytes())
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

        builder.add_word("abc");
        builder.add_word("bbc");

        let mut dictionary = Vec::<u8>::new();

        builder.into_dictionary(&mut dictionary).unwrap();

        // There should be 9 nodes because the “bc” endings shouldn’t
        // be combined into one. Each node takes up 3 bytes in this
        // small example.
        assert_eq!(dictionary.len(), 9 * 3);

        assert_eq!(
            &dictionary,
            &[
                0, 1, b'*',
                10, 1, b'a',
                0, 1, b'b',
                0, 1, b'c',
                0, 0, b'\0',
                0, 1, b'b',
                0, 1, b'b',
                0, 1, b'c',
                0, 0, b'\0',
            ],
        );
    }
}
