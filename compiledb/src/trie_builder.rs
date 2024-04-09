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
use std::cmp::Ordering;

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
//
// Duplicate nodes in the list are removed so the trie forms a
// directed acyclic graph instead of a tree.

use std::num::NonZeroUsize;

struct Node {
    ch: char,

    // If we consider the trie to be a binary tree with the first
    // child as one branch and the next sibling as the other, then
    // this is the number of nodes in the tree starting from this
    // point. Calculated in a separate pass.
    size: usize,

    // The byte offset to the end of the file. Calculated in a
    // separate pass.
    offset: usize,

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
            size: 1,
            offset: 0,
            first_child: None,
            next_sibling: None,
        }
    }
}

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

    fn sorted_indices(&self) -> Vec<usize> {
        let mut indices = (0..self.nodes.len()).collect::<Vec<usize>>();

        indices.sort_by(|&a, &b| self.compare_nodes(a, b));

        indices
    }

    fn node_info(&self, index: usize, next_offset: usize) -> NodeInfo {
        let node = &self.nodes[index];

        let character_length = node.ch.len_utf8();

        let character_offset = next_offset + character_length;

        let child_offset = node.first_child
            .map(|index| character_offset - self.nodes[index.get()].offset)
            .unwrap_or(0);
        let sibling_offset = node.next_sibling
            .map(|index| character_offset - self.nodes[index.get()].offset)
            .unwrap_or(0);

        NodeInfo { child_offset, sibling_offset }
    }

    fn compare_equal_sized_nodes(
        &self,
        a: usize,
        b: usize,
    ) -> Ordering {
        let mut stack = vec![(a, b)];

        while let Some((entry_a, entry_b)) = stack.pop() {
            let node_a = &self.nodes[entry_a];
            let node_b = &self.nodes[entry_b];

            match node_a.ch.cmp(&node_b.ch)
                .then_with(|| {
                    node_a.first_child.is_some()
                        .cmp(&node_b.first_child.is_some())
                })
                .then_with(|| {
                    node_a.next_sibling.is_some()
                        .cmp(&node_b.next_sibling.is_some())
                })
            {
                Ordering::Equal => (),
                other => return other,
            }

            if let Some(sibling_a) = node_a.next_sibling {
                stack.push((
                    sibling_a.get(),
                    node_b.next_sibling.unwrap().get(),
                ));
            }

            if let Some(child_a) = node_a.first_child {
                stack.push((
                    child_a.get(),
                    node_b.first_child.unwrap().get(),
                ));
            }
        }

        Ordering::Equal
    }

    fn compare_nodes(&self, a: usize, b: usize) -> Ordering {
        self.nodes[b].size.cmp(&self.nodes[a].size)
            .then_with(|| self.compare_equal_sized_nodes(a, b))
    }

    fn calculate_file_positions(&mut self, sorted_indices: &[usize]) {
        for (pos, &index) in sorted_indices.iter().enumerate().rev() {
            // If this node is the same as the next node then just
            // reuse the same offset
            let next_offset = if let Some(&next_index) =
                sorted_indices.get(pos + 1)
            {
                if self.compare_nodes(index, next_index).is_eq() {
                    self.nodes[index].offset = self.nodes[next_index].offset;
                    continue;
                }

                self.nodes[next_index].offset
            } else {
                0
            };

            let info = self.node_info(index, next_offset);

            self.nodes[index].offset = next_offset
                + self.nodes[index].ch.len_utf8()
                + n_bytes_for_size(info.child_offset)
                + n_bytes_for_size(info.sibling_offset);
        }
    }

    pub fn into_dictionary(
        mut self,
        output: &mut impl Write,
    ) -> std::io::Result<()> {
        // Sort all the children of each node by character so that
        // it’s easier to compare them.
        self.sort_all_children_by_character();

        // Calculate the size of each node in the trie as if it was a
        // binary tree so that we can be sure to output the nodes
        // closer te the root first.
        self.calculate_size();

        // Get the order sorted by descending size and then by the
        // contents so that we can put the bigger nodes first and
        // easily detect duplicates.
        let sorted_indices = self.sorted_indices();

        // Calculate the position of each node in the final file and
        // detect duplicates.
        self.calculate_file_positions(&sorted_indices);

        self.write_nodes(&sorted_indices, output)
    }

    fn write_node(
        &self,
        index: usize,
        next_offset: usize,
        output: &mut impl Write,
    ) -> std::io::Result<()> {
        let info = self.node_info(index, next_offset);

        let node = &self.nodes[index];

        write_offset(info.sibling_offset, output)?;
        write_offset(info.child_offset, output)?;

        let mut ch_utf8 = [0u8; 4];

        output.write_all(node.ch.encode_utf8(&mut ch_utf8).as_bytes())
    }

    fn write_nodes(
        &self,
        sorted_indices: &[usize],
        output: &mut impl Write,
    ) -> std::io::Result<()> {
        for (pos, &index) in sorted_indices.iter().enumerate() {
            let node = &self.nodes[index];

            let next_offset = if let Some(&next_index)
                = sorted_indices.get(pos + 1)
            {
                // If this node is the same as the next one then skip it
                if self.nodes[next_index].offset == node.offset {
                    continue;
                }

                self.nodes[next_index].offset
            } else {
                0
            };

            self.write_node(index, next_offset, output)?;
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
    fn node_order() {
        let mut builder = TrieBuilder::new();

        builder.add_word("abc");
        builder.add_word("bbc");
        builder.add_word("cbe");

        builder.sort_all_children_by_character();
        builder.calculate_size();

        assert_eq!(builder.nodes[0].size, 13);
        assert_eq!(builder.nodes[1].size, 12);
        assert_eq!(builder.compare_nodes(0, 1), Ordering::Less);
        assert_eq!(builder.compare_nodes(1, 0), Ordering::Greater);
        assert_eq!(builder.nodes[2].size, 3);
        assert_eq!(builder.nodes[2].ch, 'b');
        assert_eq!(builder.nodes[6].size, 3);
        assert_eq!(builder.nodes[6].ch, 'b');
        assert_eq!(builder.compare_nodes(2, 6), Ordering::Equal);
        assert_eq!(builder.nodes[9].size, 4);
        assert_eq!(builder.nodes[9].ch, 'c');
        assert_eq!(builder.nodes[10].size, 3);
        assert_eq!(builder.nodes[10].ch, 'b');
        assert_eq!(builder.compare_nodes(2, 10), Ordering::Less);
        assert_eq!(builder.compare_nodes(10, 2), Ordering::Greater);
    }

    #[test]
    fn duplicates() {
        let mut builder = TrieBuilder::new();

        builder.add_word("abc");
        builder.add_word("bbc");

        let mut dictionary = Vec::<u8>::new();

        builder.into_dictionary(&mut dictionary).unwrap();

        // There should only be 6 nodes because the “bc” endings
        // should be combined into one. Each node takes up 3 bytes in
        // this small example.
        assert_eq!(dictionary.len(), 6 * 3);

        assert_eq!(
            &dictionary,
            &[
                0, 1, b'*',
                1, 4, b'a',
                0, 1, b'b',
                0, 1, b'b',
                0, 1, b'c',
                0, 0, b'\0',
            ],
        );
    }
}
