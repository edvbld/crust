# Copyright 2016 Erik Duveblad
#
# This file is part of crust.
#
# crust is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# crust is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with crust.  If not, see <http://www.gnu.org/licenses/>.

use std::str::FromStr;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
enum Token<'a> {
	LeftParen,
    RightParen,
	Number(u64),
    Symbol(&'a str)
}

#[derive(Debug)]
struct Fun<'a> {
	name: &'a str,
	args: Vec<Box<Node<'a>>>
}

#[derive(Debug)]
enum Node<'a> {
	Symbol(&'a str),
	Application(Fun<'a>),
	Number(u64)	
}


fn separate<'a>(s: &'a str, separators: &str) -> Vec<&'a str> {
	let mut v = Vec::new();
	let mut from = 0;
	let mut to = 0;
	let mut char_indices = s.char_indices();
	while let Some((i, c)) = char_indices.next() {
		if separators.contains(c) {
			if to > from {
				v.push(&s[from..to]);
			}
			v.push(&s[i..i+1]);
			to = i + 1;
			from = to;
        } else {
			to += 1;
		}
	}
	if to > from {
		v.push(&s[from..to]);
	}
	return v;
}

#[test]
fn test_separate() {
	assert_eq!(vec!["apa"], separate("apa", "()"));
	assert_eq!(vec!["("], separate("(", "()"));
	assert_eq!(vec!["(", ")"], separate("()", "()"));
	assert_eq!(vec!["(", "apa", ")"], separate("(apa)", "()"));
}

fn lex<'a>(s: &'a str) -> Vec<Token<'a>> {
	s.trim().split_whitespace().flat_map(|w| separate(w, "()")).map(|w| {
		match w {
			"(" 	=> Token::LeftParen,
			")" 	=> Token::RightParen,
			_	    => match u64::from_str(w) {
					       Ok(n) => Token::Number(n),
						   _     => Token::Symbol(w)		
					   }
		}
	}).collect()
}

fn parse_exp<'a>(tokens: &'a [Token]) -> (usize, Node<'a>) {
	if tokens.len() == 0 {
		panic!("No tokens!");
	}
	match tokens[0] {
		Token::LeftParen => {
			if tokens.len() < 3 {
				panic!("Too few tokens!");
			}

			let name = match tokens[1] {
				Token::Symbol(s) => s,
				_	         => panic!("Unexpected token: {:?}", tokens[1])
			};
			let mut args = Vec::new();
			let mut i = 2;
			while tokens[i] != Token::RightParen {
				let (n, arg) = parse_exp(&tokens[i..]);
				args.push(Box::new(arg));
				i += n;
			}
			(i + 1, Node::Application(Fun { name: name, args: args }))
		}
        	Token::Number(n) => (1, Node::Number(n)),
		Token::Symbol(s) => (1, Node::Symbol(s)),
		_ 				  => panic!("Unexpected token: {:?}", tokens[0])
	}		
}

fn parse<'a>(tokens: &'a Vec<Token>) -> Vec<Node<'a>> {
	let mut n = 0;
	let mut v = Vec::new();
	while n < tokens.len() {
		let (nn, root) = parse_exp(&tokens[n..]);
		v.push(root);
		n += nn;
		
	}
	assert!(n == tokens.len(), "Did not parse all tokens");
	return v;
}

fn eval<'a>(root: &'a Node<'a>, env: &mut HashMap<&'a str, &'a Node<'a>>) -> u64 {
	match root {
		&Node::Symbol(name) => eval(env.get(name).unwrap(), env),
		&Node::Number(n) => n,
		&Node::Application(ref f) => {
			match f.name {
				"+" => {
			let args = f.args.iter().map(|a| eval(&*a, env));
			args.fold(0, |acc, a| acc + a)},
				"-" => {
					let args = f.args.iter().map(|a| eval(&*a, env));
					args.fold(0, |acc, a| acc - a)},
				"*" => {
					let args = f.args.iter().map(|a| eval(&*a, env));
					args.fold(1, |acc, a| acc * a)},
				"/" => {
					let mut args = f.args.iter().map(|a| eval(&*a, env));
					let mut res = args.next().unwrap();
					for n in args {
						res = res / n;
					}	
					res
				},
				"define" => {
					let name = match *f.args[0] {
						Node::Symbol(s) => s,
						ref n @ _		 => panic!("Unexpected node: {:?}", n)
					};
					env.insert(name, &*f.args[1]);
					0
				},
				_	=> panic!("Uknown function")
			}
		}
	}
}

fn eval_program(roots: &[Node]) -> u64 {
  let mut res = 0;
  let mut env = HashMap::new();
  for root in roots {
    println!("root: {:?}", root);
    res = eval(root, &mut env);
    println!("res is {}", res);
  }
  return res;
}

fn main() {
	let mut args = std::env::args();
	if args.len() != 2 {
		panic!("Usage: crust <program>");
	}

	let source = args.nth(1).unwrap();
	let tokens = lex(&source);
	let asts = parse(&tokens);
	println!("{}", eval_program(&asts));
}
