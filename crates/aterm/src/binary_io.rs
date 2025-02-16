// Author(s): Jan Friso Groote
// Distributed under the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at
// http://www.boost.org/LICENSE_1_0.txt)

use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::sync::Arc;
use std::cell::RefCell;

const BAF_MAGIC: u16 = 0x8baf;
const BAF_VERSION: u16 = 0x8308;

pub struct BinaryAtermOStream<W: Write> {
    stream: W,
    function_symbols: HashSet<Symbol>,
    terms: HashSet<ATerm>,
}

impl<W: Write> BinaryAtermOStream<W> {
    pub fn new(mut stream: W) -> Self {
        stream.write_all(&[0]).unwrap();
        stream.write_all(&BAF_MAGIC.to_be_bytes()).unwrap();
        stream.write_all(&BAF_VERSION.to_be_bytes()).unwrap();

        let mut function_symbols = HashSet::new();
        function_symbols.insert(Symbol {
            name: "end_of_stream".to_string(),
            arity: 0,
        });

        Self {
            stream,
            function_symbols,
            terms: HashSet::new(),
        }
    }

    pub fn put(&mut self, term: &ATerm) {
        let mut stack = vec![term.clone()];
        while let Some(current) = stack.pop() {
            if !self.terms.contains(&current) {
                match &current {
                    ATerm::Int(value) => {
                        self.stream.write_all(&[2]).unwrap();
                        self.stream.write_all(&value.to_be_bytes()).unwrap();
                    }
                    ATerm::Appl(symbol, args) => {
                        if !self.function_symbols.contains(symbol) {
                            self.stream.write_all(&[0]).unwrap();
                            self.stream.write_all(symbol.name.as_bytes()).unwrap();
                            self.stream.write_all(&(symbol.arity as u32).to_be_bytes()).unwrap();
                            self.function_symbols.insert(symbol.clone());
                        }
                        self.stream.write_all(&[1]).unwrap();
                        for arg in args {
                            stack.push(arg.clone());
                        }
                    }
                }
                self.terms.insert(current);
            }
        }
    }
}

pub struct BinaryAtermIStream<R: Read> {
    stream: R,
    function_symbols: Vec<Symbol>,
    terms: Vec<ATerm>,
}

impl<R: Read> BinaryAtermIStream<R> {
    pub fn new(mut stream: R) -> Self {
        let mut header = [0; 1];
        stream.read_exact(&mut header).unwrap();
        let mut magic = [0; 2];
        stream.read_exact(&mut magic).unwrap();
        let mut version = [0; 2];
        stream.read_exact(&mut version).unwrap();

        if u16::from_be_bytes(magic) != BAF_MAGIC {
            panic!("Error while reading: missing the BAF_MAGIC control sequence.");
        }

        if u16::from_be_bytes(version) != BAF_VERSION {
            panic!("The BAF version of the input file is incompatible with the version of this tool. The input file must be regenerated.");
        }

        let mut function_symbols = Vec::new();
        function_symbols.push(Symbol {
            name: "end_of_stream".to_string(),
            arity: 0,
        });

        Self {
            stream,
            function_symbols,
            terms: Vec::new(),
        }
    }

    pub fn get(&mut self) -> ATerm {
        loop {
            let mut header = [0; 1];
            self.stream.read_exact(&mut header).unwrap();
            match header[0] {
                0 => {
                    let mut name = vec![];
                    self.stream.read_to_end(&mut name).unwrap();
                    let mut arity = [0; 4];
                    self.stream.read_exact(&mut arity).unwrap();
                    let symbol = Symbol {
                        name: String::from_utf8(name).unwrap(),
                        arity: u32::from_be_bytes(arity) as usize,
                    };
                    self.function_symbols.push(symbol);
                }
                1 => {
                    let symbol_index = self.stream.read_u8().unwrap() as usize;
                    let symbol = self.function_symbols[symbol_index].clone();
                    let mut args = Vec::new();
                    for _ in 0..symbol.arity {
                        let term_index = self.stream.read_u8().unwrap() as usize;
                        args.push(self.terms[term_index].clone());
                    }
                    let term = ATerm::Appl(symbol, args);
                    self.terms.push(term.clone());
                    return term;
                }
                2 => {
                    let mut value = [0; 4];
                    self.stream.read_exact(&mut value).unwrap();
                    let term = ATerm::Int(i32::from_be_bytes(value));
                    self.terms.push(term.clone());
                    return term;
                }
                _ => panic!("Unknown packet type"),
            }
        }
    }
}

pub fn write_term_to_binary_stream<W: Write>(term: &ATerm, stream: W) {
    let mut ostream = BinaryAtermOStream::new(stream);
    ostream.put(term);
}

pub fn read_term_from_binary_stream<R: Read>(stream: R) -> ATerm {
    let mut istream = BinaryAtermIStream::new(stream);
    istream.get()
}