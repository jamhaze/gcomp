use std::collections::HashMap;
use std::collections::VecDeque;

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

pub fn run(text: &str) {
    if !text.is_empty() {
        
        let binary = text.as_bytes();
        let mut binary_rep = "".to_string();
        for c in binary {
            binary_rep += &format!("0{:b}", c);
        }
        println!("Number of bits pre-compression: {}", binary_rep.len());
        let freq_map = get_frequencies(text);
        println!("Freq map: {:?}", freq_map);
        let tree = build_huffman_tree(freq_map);
        let mut huffman_codes: HashMap<char, String> = HashMap::new();
        generate_huffman_codes(Some(Box::new(tree)), String::new(), &mut huffman_codes);
        println!("Huffman codes: {:?}", huffman_codes);
        let mut compressed_binary = "".to_string();
        for c in text.chars() {
            let char_code = huffman_codes.get(&c);
            compressed_binary += char_code.unwrap();
        }
        println!("Number of bits post-compression: {}", compressed_binary.len());

    } else {
        println!("Error: The slice cannot be empty");
    }
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