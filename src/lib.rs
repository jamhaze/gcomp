use std::collections::HashMap;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::fs;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    version, 
    about = "This is a command line tool for compressing and decompressing a file", 
    long_about = None
)]
pub struct Config {
    #[arg(
        short,
        long,
        help = "Specify decode mode to decompress a previously compressed file",
    )]
    decode: bool,

    #[arg(
        short,
        long, 
        help = "Enter the path to the file which should be compressed",
    )]
    file_path: String,

    #[arg(
        short,
        long, 
        help = "Enter the path to the compressed output file",
    )]
    output_path: String,
}    

// Node is a recursive data type because each Node can hold up to two child Nodes inside itself.
// The child nodes, left and right, must therefore be wrapped in a smart pointer Box type so that the size of
// a Node can be known at compile time.
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Node {
    byte: Option<u8>,
    score: u32,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Node {
    fn new(byte: Option<u8>, score: u32, left: Option<Box<Node>>, right: Option<Box<Node>>) -> Self {
        Self {
            byte,
            score,
            left,
            right,
        }
    }
}

pub fn run(config: Config) -> std::io::Result<()> {
    if config.decode {
        let data: Vec<u8> = fs::read(config.file_path)?;
        let num_unique_bytes_bytes = <[u8; 4]>::try_from(&data[0..4]).unwrap();
        let num_unique_bytes: u32 = u32::from_be_bytes(num_unique_bytes_bytes);
        println!("Number of unique bytes: {}", num_unique_bytes);
        let total_bytes_bytes = <[u8; 4]>::try_from(&data[4..8]).unwrap();
        let total_bytes: u32 = u32::from_be_bytes(total_bytes_bytes);
        println!("Total bytes: {}", total_bytes);
        let mut freq_map: BTreeMap<u8, u32> = BTreeMap::new();
        let mut i = 0;
        let mut offset = 8;
        while i < num_unique_bytes {
            let unique_byte = data[offset];
            let freq_bytes = <[u8; 4]>::try_from(&data[offset + 1..offset + 5]).unwrap();
            let freq: u32 = u32::from_be_bytes(freq_bytes);
            freq_map.insert(unique_byte, freq);
            i += 1;
            offset += 5;
        }
        //println!("{:?}", &freq_map);
        let tree = build_huffman_tree(&freq_map);
        //println!("{:?}", &tree);
        let mut current_node = &tree;
        let mut text_as_bytes = Vec::new();
        let mut bytes_read = 0;
        while bytes_read < total_bytes {
            let reading_frame = &data[offset];
            for shift_amt in (0..8).rev() {
                let bit = (reading_frame >> shift_amt) & 1;
                if bit == 0 {
                    current_node = &current_node.left.as_ref().unwrap();
                } else {
                    current_node = &current_node.right.as_ref().unwrap();
                }
                if let Some(byte) = current_node.byte {
                    text_as_bytes.push(byte);
                    current_node = &tree;
                    bytes_read += 1;
                    if bytes_read == total_bytes {
                        break;
                    }
                }
            }
            offset += 1;
        }
        let text = match str::from_utf8(&text_as_bytes[..]) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence {}", e),
        };
        fs::write(&config.output_path, text)?;

    } else {
        let text = fs::read_to_string(config.file_path)?;
        if text.is_empty() {
            println!("Error: The slice cannot be empty");
            return Ok(());
        }
        let text_as_bytes = text.as_bytes();
        let freq_map = get_byte_frequencies(&text_as_bytes);
        let tree = build_huffman_tree(&freq_map);
        let mut huffman_codes: HashMap<u8, Vec<u8>> = HashMap::new();
        generate_huffman_codes(Some(Box::new(tree)), Vec::new(), &mut huffman_codes);
        let compressed_file_bytes = compress_to_file(text_as_bytes, freq_map, huffman_codes);
        fs::write(&config.output_path, compressed_file_bytes)?;
        println!("Compressed file written to: {}", &config.output_path);
    }
    
    Ok(())
}

fn get_byte_frequencies(text_as_bytes: &[u8]) -> BTreeMap<u8, u32> {
    let mut freq_map: BTreeMap<u8, u32> = BTreeMap::new();
    for byte in text_as_bytes {
        freq_map.entry(*byte).and_modify(|freq| *freq += 1).or_insert(1);
    }
    freq_map
}

fn build_huffman_tree(freq_map: &BTreeMap<u8, u32>) -> Node {
    let mut nodes = vec![];

    for (k, v) in freq_map.iter() {
        nodes.push(Node::new(Some(*k), *v, None, None));
    }
    
    nodes.sort_by(|a, b| a.score.cmp(&b.score));
    let mut node_queue: VecDeque<Node> = VecDeque::from(nodes);
    
    while node_queue.len() > 1 {
        let left_child = node_queue.pop_front().unwrap();
        let right_child = node_queue.pop_front().unwrap();
        let merged_node = Node::new(
            None, 
            left_child.score + right_child.score,
            Some(Box::new(left_child)),
            Some(Box::new(right_child)),
        );
        
        let mut pos = node_queue.len();
        for (i, node) in node_queue.iter().enumerate() {
            if merged_node.score < node.score {
                pos = i;
                break;
            }
        }
        node_queue.insert(pos, merged_node);

    }
    node_queue.pop_front().unwrap()
}

fn generate_huffman_codes(base: Option<Box<Node>>, code: Vec<u8>, huffman_codes: &mut HashMap<u8, Vec<u8>>) {
    if let Some(node) = base {
        if let Some(x) = node.byte {
            huffman_codes.insert(x, code.clone());
        }
        let mut left_code = code.clone();
        left_code.push(0);
        let mut right_code = code.clone();
        right_code.push(1);
        
        generate_huffman_codes(node.left, left_code, huffman_codes);
        generate_huffman_codes(node.right, right_code, huffman_codes);
    }
}

fn compress_to_file(text_as_bytes: &[u8], freq_map: BTreeMap<u8, u32>, huffman_codes: HashMap<u8, Vec<u8>>) -> Vec<u8> {
    let num_unique_bytes: u32 = freq_map.keys().len().try_into().unwrap();
    let total_bytes: u32 = text_as_bytes.len().try_into().unwrap();
    let mut bytes = Vec::new();
    for byte in num_unique_bytes.to_be_bytes() {
        bytes.push(byte);
    }
    for byte in total_bytes.to_be_bytes() {
        bytes.push(byte);
    }
    for (k, v) in freq_map {
        bytes.push(k);
        for byte in v.to_be_bytes() {
            bytes.push(byte);
        }
    }
    let mut current_byte: u8 = 0;
    let mut bits_in_current_byte: u8 = 0; 
    for byte in text_as_bytes {
        let huffman_code = huffman_codes.get(&byte).unwrap();
        for bit in huffman_code {
            current_byte = (current_byte << 1) | bit;
            bits_in_current_byte += 1;
            if bits_in_current_byte == 8 {
                bytes.push(current_byte);
                current_byte = 0;
                bits_in_current_byte = 0;
            }
        }
    }
    if bits_in_current_byte > 0 {
        current_byte <<= 8 - bits_in_current_byte;
        bytes.push(current_byte);
    }

    bytes
}



#[cfg(test)]
mod tests {
    use super::*;

    fn example_freq_map() -> BTreeMap<u8, u32> {
        BTreeMap::from([
            (b'A', 4),
            (b'G', 3),
            (b'C', 2),
            (b'T', 1),
        ])
    }

    fn example_tree() -> Node {
        Node {
            byte: None,
            score: 10,
            left: Some(
                Box::new(
                    Node {
                        byte: Some(b'A'),
                        score: 4,
                        left: None,
                        right: None,
                    },
                )
            ),
            right: Some(
                Box::new(
                    Node {
                        byte: None,
                        score: 6,
                        left: Some(
                            Box::new(
                                Node {
                                    byte: Some(b'G'),
                                    score: 3,
                                    left: None,
                                    right: None,
                                },
                            )
                        ),
                        right: Some(
                            Box::new(
                                Node {
                                    byte: None,
                                    score: 3,
                                    left: Some(
                                        Box::new(
                                            Node {
                                                byte: Some(b'T'),
                                                score: 1,
                                                left: None,
                                                right: None,
                                            },
                                        )
                                    ),
                                    right: Some(
                                        Box::new(
                                            Node {
                                                byte: Some(b'C'),
                                                score: 2,
                                                left: None,
                                                right: None,
                                            },
                                        )
                                    ),
                                },
                            )
                        ),
                    },
                )
            ),
        }
    }

    #[test]
    fn test_get_frequencies() {
        let text_as_bytes = "AGATACGCAG".as_bytes();
        let expected = example_freq_map();
        let freq_map = get_byte_frequencies(text_as_bytes);
        assert_eq!(expected, freq_map);
    }

    #[test]
    fn test_build_huffman_tree() {
        let freq_map = example_freq_map();
        let expected = example_tree();
        let huffman_tree = build_huffman_tree(&freq_map);
        assert_eq!(expected, huffman_tree);
    }

    #[test]
    fn test_generate_huffman_codes() {
        let tree = example_tree();
        let expected = HashMap::from([
            (b'A', vec![0]),
            (b'G', vec![1, 0]),
            (b'T', vec![1, 1, 0]),
            (b'C', vec![1, 1, 1]),
        ]);
        let mut huffman_codes: HashMap<u8, Vec<u8>> = HashMap::new();
        generate_huffman_codes(Some(Box::new(tree)), vec![], &mut huffman_codes);
        assert_eq!(expected, huffman_codes);
    }
}