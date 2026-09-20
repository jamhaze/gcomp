use std::collections::HashMap;
use std::collections::VecDeque;
use std::fs;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    version, 
    about = "This is a command line tool for compressing a .FASTA file", 
    long_about = None
)]
pub struct Cli {
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

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Node {
    symbol: Option<char>,
    score: u32,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Node {
    fn new(symbol: Option<char>, score: u32, left: Option<Box<Node>>, right: Option<Box<Node>>) -> Self {
        Self {
            symbol,
            score,
            left,
            right,
        }
    }
}

pub fn run(config: Cli) -> std::io::Result<()> {
    let text = fs::read_to_string(config.file_path)?;

    if text.is_empty() {
        println!("Error: The slice cannot be empty");
        return Ok(());
    }

    let freq_map = get_frequencies(&text);
    let tree = build_huffman_tree(freq_map);
    let mut huffman_codes: HashMap<char, String> = HashMap::new();
    generate_huffman_codes(Some(Box::new(tree)), String::new(), &mut huffman_codes);

    let mut encoded_bits = String::new();
    for ch in text.chars() {
        encoded_bits.push_str(huffman_codes.get(&ch).unwrap());
    }

    // pad to a multiple of 8 bits so it can be written as bytes
    let padding = (8 - (encoded_bits.len() % 8)) % 8;
    encoded_bits.push_str(&"0".repeat(padding));

    let mut bytes = Vec::new();
    for chunk in encoded_bits.as_bytes().chunks(8) {
        // each chunk is a slice of bytes e.g. [b'0', b'1', b'0', b'1', b'1', b'0', b'1', b'1']
        // from_utf8 converts that back into a &str
        let s = std::str::from_utf8(chunk).unwrap();
        // from_str_radix takes the &str and converts it to a base 2 integer.
        // the byte is then pushed to the vector.
        bytes.push(u8::from_str_radix(s, 2).unwrap());
    }

    fs::write(&config.output_path, bytes)?;

    println!("Compressed file written to: {}", &config.output_path);
    Ok(())
}

fn get_frequencies(text: &str) -> HashMap<char, u32> {
    let mut freq_map: HashMap<char, u32> = HashMap::new();
    for c in text.chars() {
        freq_map.entry(c).and_modify(|freq| *freq += 1).or_insert(1);
    }
    freq_map
}

fn build_huffman_tree(freq_map: HashMap<char, u32>) -> Node {
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

fn generate_huffman_codes(base: Option<Box<Node>>, code: String, huffman_codes: &mut HashMap<char, String>) {
    if let Some(node) = base {
        if let Some(x) = node.symbol {
            huffman_codes.insert(x, code.clone());
        }
        let left_code = code.clone() + "0";
        let right_code = code.clone() + "1";
        
        generate_huffman_codes(node.left, left_code, huffman_codes);
        generate_huffman_codes(node.right, right_code, huffman_codes);
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    fn example_freq_map() -> HashMap<char, u32> {
        HashMap::from([
            ('A', 4),
            ('G', 3),
            ('C', 2),
            ('T', 1),
        ])
    }

    fn example_tree() -> Node {
        Node {
            symbol: None,
            score: 10,
            left: Some(
                Box::new(
                    Node {
                        symbol: Some('A'),
                        score: 4,
                        left: None,
                        right: None,
                    },
                )
            ),
            right: Some(
                Box::new(
                    Node {
                        symbol: None,
                        score: 6,
                        left: Some(
                            Box::new(
                                Node {
                                    symbol: Some('G'),
                                    score: 3,
                                    left: None,
                                    right: None,
                                },
                            )
                        ),
                        right: Some(
                            Box::new(
                                Node {
                                    symbol: None,
                                    score: 3,
                                    left: Some(
                                        Box::new(
                                            Node {
                                                symbol: Some('T'),
                                                score: 1,
                                                left: None,
                                                right: None,
                                            },
                                        )
                                    ),
                                    right: Some(
                                        Box::new(
                                            Node {
                                                symbol: Some('C'),
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
        let test_text = "AGATACGCAG";
        let expected = example_freq_map();
        let freq_map = get_frequencies(test_text);
        assert_eq!(expected, freq_map);
    }

    #[test]
    fn test_build_huffman_tree() {
        let freq_map = example_freq_map();
        let expected = example_tree();
        let huffman_tree = build_huffman_tree(freq_map);
        assert_eq!(expected, huffman_tree);
    }

    #[test]
    fn test_generate_huffman_codes() {
        let tree = example_tree();
        let expected = HashMap::from([
            ('A', String::from("0")),
            ('G', String::from("10")),
            ('T', String::from("110")),
            ('C', String::from("111")),
        ]);
        let mut huffman_codes: HashMap<char, String> = HashMap::new();
        generate_huffman_codes(Some(Box::new(tree)), String::new(), &mut huffman_codes);
        assert_eq!(expected, huffman_codes);
    }
}