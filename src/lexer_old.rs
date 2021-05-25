#![allow(dead_code)]

#[derive(Debug, Clone)]
pub enum KeywordType {
    Break,
    Char,
    Double,
    Else,
    For,
    If,
    Int,
    Return,
    Struct,
    Void,
    While,
}

#[derive(Debug, Clone)]
pub enum TokenType {
    CtReal(f32),
    CtInt(isize),
    CtChar(char),
    CtString(String),
    Id(String),
    End,
    Div,
    Add,
    Sub,
    Mul,
    Dot,
    And,
    Or,
    Not,
    NotEq,
    Equal,
    Assign,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    Comma,
    Semicolon,
    Lpar,
    Rpar,
    Lbracket,
    Rbracket,
    Lacc,
    Racc,
    Error,
    Keyword(KeywordType),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
}


pub fn get_next_token(text: &Vec<char>, text_idx: &mut usize, current_line: &mut usize) -> Token {
    // consumes an atom and returns the code of the atom

    let mut state = 0;
    let mut token_string = String::new();
    let mut token_char: char = 0 as char;
    loop {
        // Infinite loop because we don't know the char length of a token
        // We stop only when we reach a final state
        // If the final state is decided after we consume a character from the next token we decrement *text_idx
        // if *text_idx == text.len(){
        //     return Token {
        //         token_type: TokenType::End,
        //         line: *current_line,
        //     };
        // }
        let c = text[*text_idx];
        *text_idx += 1;
        // println!(
        //     "state: {}, i: {}, character: {:?}, token_string: {:?}",
        //     state, text_idx, c, token_string
        // );
        match state {
            // Start
            0 => match c {
                // Id
                'a'..='z' | 'A'..='Z' | '_' => {
                    state = 30;
                    token_string.push(c);
                }
                // End
                '\0' => {
                    *text_idx -= 1;
                    return Token {
                        token_type: TokenType::End,
                        line: *current_line,
                    };
                }

                // Operators and delimitators
                '+' => {
                    //state = 37;
                    return Token {
                        token_type: TokenType::Add,
                        line: *current_line,
                    };
                }
                '-' => {
                    //state = 38;
                    return Token {
                        token_type: TokenType::Sub,
                        line: *current_line,
                    };
                }
                '*' => {
                    //state = 39;
                    return Token {
                        token_type: TokenType::Mul,
                        line: *current_line,
                    };
                }
                '.' => {
                    //state = 40;
                    return Token {
                        token_type: TokenType::Dot,
                        line: *current_line,
                    };
                }
                ',' => {
                    //state = 40;
                    return Token {
                        token_type: TokenType::Comma,
                        line: *current_line,
                    };
                }
                ';' => {
                    //state = 40;
                    return Token {
                        token_type: TokenType::Semicolon,
                        line: *current_line,
                    };
                }
                '(' => {
                    //state = 40;
                    return Token {
                        token_type: TokenType::Lpar,
                        line: *current_line,
                    };
                }
                ')' => {
                    //state = 40;
                    return Token {
                        token_type: TokenType::Rpar,
                        line: *current_line,
                    };
                }
                '[' => {
                    //state = 40;
                    return Token {
                        token_type: TokenType::Lbracket,
                        line: *current_line,
                    };
                }
                ']' => {
                    //state = 40;
                    return Token {
                        token_type: TokenType::Rbracket,
                        line: *current_line,
                    };
                }
                '{' => {
                    //state = 40;
                    return Token {
                        token_type: TokenType::Lacc,
                        line: *current_line,
                    };
                }
                '}' => {
                    //state = 40;
                    return Token {
                        token_type: TokenType::Racc,
                        line: *current_line,
                    };
                }

                '&' => {
                    state = 15;
                }
                '|' => {
                    state = 16;
                }
                '!' => {
                    state = 17;
                }
                '=' => {
                    state = 18;
                }
                '<' => {
                    state = 19;
                }
                '>' => {
                    state = 20;
                }

                // Spaces, Comments, etc
                ' ' | '\r' | '\n' | '\t' => {
                    state = 0;
                    if c == '\n' {
                        *current_line += 1;
                    }
                }
                '/' => {
                    state = 12;
                }
                '0' => {
                    state = 2;
                    token_string.push(c);
                }
                '1'..='9' => {
                    state = 1;
                    token_string.push(c);
                }
                // CT_Char
                '\'' => {
                    state = 21;
                }
                // CT_String
                '\"' => {
                    state = 25;
                }

                _ => {
                    *text_idx -= 1;
                    println!("Not implemented yet");
                    return Token {
                        token_type: TokenType::Error,
                        line: *current_line,
                    };
                }
            },
            // Operators and delimitators
            15 => match c {
                '&' => {
                    //state = 37;
                    return Token {
                        token_type: TokenType::And,
                        line: *current_line,
                    };
                }
                _ => {
                    *text_idx -= 1;
                    return Token {
                        token_type: TokenType::Error,
                        line: *current_line,
                    };
                }
            },
            16 => match c {
                '|' => {
                    //state = 37;
                    return Token {
                        token_type: TokenType::Or,
                        line: *current_line,
                    };
                }
                _ => {
                    *text_idx -= 1;
                    return Token {
                        token_type: TokenType::Error,
                        line: *current_line,
                    };
                }
            },
            17 => match c {
                '=' => {
                    //state = 37;
                    return Token {
                        token_type: TokenType::NotEq,
                        line: *current_line,
                    };
                }
                _ => {
                    *text_idx -= 1;
                    return Token {
                        token_type: TokenType::Not,
                        line: *current_line,
                    };
                }
            },
            18 => match c {
                '=' => {
                    //state = 37;
                    return Token {
                        token_type: TokenType::Equal,
                        line: *current_line,
                    };
                }
                _ => {
                    *text_idx -= 1;
                    return Token {
                        token_type: TokenType::Assign,
                        line: *current_line,
                    };
                }
            },
            19 => match c {
                '=' => {
                    //state = 37;
                    return Token {
                        token_type: TokenType::LessEq,
                        line: *current_line,
                    };
                }
                _ => {
                    *text_idx -= 1;
                    return Token {
                        token_type: TokenType::Less,
                        line: *current_line,
                    };
                }
            },
            20 => match c {
                '=' => {
                    //state = 37;
                    return Token {
                        token_type: TokenType::GreaterEq,
                        line: *current_line,
                    };
                }
                _ => {
                    *text_idx -= 1;
                    return Token {
                        token_type: TokenType::Greater,
                        line: *current_line,
                    };
                }
            },
            // Id
            30 => match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                    state = 30;
                    token_string.push(c);
                }
                _ => {
                    *text_idx -= 1;
                    // Check for keyword
                    match token_string.as_str() {
                        "break" => {
                            return Token {
                                token_type: TokenType::Keyword(KeywordType::Break),
                                line: *current_line,
                            }
                        }
                        "char" => {
                            return Token {
                                token_type: TokenType::Keyword(KeywordType::Char),
                                line: *current_line,
                            }
                        }
                        "double" => {
                            return Token {
                                token_type: TokenType::Keyword(KeywordType::Double),
                                line: *current_line,
                            }
                        }
                        "else" => {
                            return Token {
                                token_type: TokenType::Keyword(KeywordType::Else),
                                line: *current_line,
                            }
                        }
                        "for" => {
                            return Token {
                                token_type: TokenType::Keyword(KeywordType::For),
                                line: *current_line,
                            }
                        }
                        "if" => {
                            return Token {
                                token_type: TokenType::Keyword(KeywordType::If),
                                line: *current_line,
                            }
                        }
                        "int" => {
                            return Token {
                                token_type: TokenType::Keyword(KeywordType::Int),
                                line: *current_line,
                            }
                        }
                        "return" => {
                            return Token {
                                token_type: TokenType::Keyword(KeywordType::Return),
                                line: *current_line,
                            }
                        }
                        "struct" => {
                            return Token {
                                token_type: TokenType::Keyword(KeywordType::Struct),
                                line: *current_line,
                            }
                        }
                        "void" => {
                            return Token {
                                token_type: TokenType::Keyword(KeywordType::Void),
                                line: *current_line,
                            }
                        }
                        "while" => {
                            return Token {
                                token_type: TokenType::Keyword(KeywordType::While),
                                line: *current_line,
                            }
                        }
                        _ => {
                            return Token {
                                token_type: TokenType::Id(token_string),
                                line: *current_line,
                            }
                        }
                    }
                    //state = 35;
                }
            },
            // Comments
            12 => match c {
                '*' => {
                    state = 13;
                }
                '/' => {
                    state = 29;
                }
                _ => {
                    *text_idx -= 1;
                    return Token {
                        token_type: TokenType::Div,
                        line: *current_line,
                    };
                }
            },
            13 => match c {
                '*' => {
                    state = 14;
                }
                '\n' => {
                    // anything else stays in state 13
                    *current_line += 1;
                }
                _ => {
                    // anything else stays in state 13
                }
            },
            14 => match c {
                '*' => {
                    state = 14;
                }
                '/' => {
                    state = 0;
                }
                '\n' => {
                    // anything else stays in state 13
                    state = 13;
                    *current_line += 1;
                }
                _ => {
                    // anything except `*` or `/` goes in state 13
                    state = 13;
                }
            },
            29 => match c {
                '\n' | '\r' | '\0' => {
                    state = 0;
                }
                _ => {
                    // anything else stays in state 29
                }
            },
            //CT_INT
            1 => match c {
                '0'..='9' => {
                    // state doesnt change
                    token_string.push(c);
                }
                '.' => {
                    state = 7;
                    token_string.push(c);
                }
                'e' | 'E' => {
                    state = 9;
                    token_string.push(c);
                }
                _ => {
                    *text_idx -= 1;
                    let int_value = isize::from_str_radix(&token_string, 10).unwrap_or(0);
                    return Token {
                        token_type: TokenType::CtInt(int_value),
                        line: *current_line,
                    };
                }
            },
            2 => match c {
                // Hex
                'x' => {
                    state = 4;
                    token_string.push(c);
                }
                // Octal
                '0'..='7' => {
                    state = 3;
                    token_string.push(c);
                }
                '8' | '9' => {
                    state = 6;
                    token_string.push(c);
                }
                'e' | 'E' => {
                    state = 9;
                    token_string.push(c);
                }
                '.' => {
                    state = 7;
                    token_string.push(c);
                }
                _ => {
                    return Token {
                        token_type: TokenType::CtInt(0),
                        line: *current_line,
                    };
                }
            },
            4 => match c {
                'a'..='f' | 'A'..='F' | '0'..='9' => {
                    state = 5;
                    token_string.push(c);
                }
                _ => {
                    return Token {
                        token_type: TokenType::Error,
                        line: *current_line,
                    };
                }
            },
            5 => match c {
                'a'..='f' | 'A'..='F' | '0'..='9' => {
                    state = 5; // more numbers
                    token_string.push(c);
                }
                _ => {
                    *text_idx -= 1;
                    //println!("hex string {}", token_string);
                    let int_value = isize::from_str_radix(&token_string[2..], 16).unwrap_or(0);
                    return Token {
                        token_type: TokenType::CtInt(int_value),
                        line: *current_line,
                    };
                }
            },
            6 => match c {
                '0'..='9' => {
                    token_string.push(c);
                }
                '.' => {
                    state = 7;
                    token_string.push(c);
                }
                'e' | 'E' => {
                    state = 9;
                    token_string.push(c);
                }
                _ => {
                    return Token {
                        token_type: TokenType::Error,
                        line: *current_line,
                    };
                }
            },
            // Octal
            3 => match c {
                '0'..='7' => {
                    token_string.push(c);
                }
                '8' | '9' => {
                    state = 6;
                    token_string.push(c);
                }
                'e' | 'E' => {
                    state = 9;
                    token_string.push(c);
                }
                '.' => {
                    state = 7;
                    token_string.push(c);
                }
                _ => {
                    *text_idx -= 1;
                    let int_value = isize::from_str_radix(&token_string[1..], 8).unwrap_or(0);
                    return Token {
                        token_type: TokenType::CtInt(int_value),
                        line: *current_line,
                    };
                }
            },

            //CT_REAL
            7 => match c {
                '0'..='9' => {
                    state = 8;
                    token_string.push(c);
                }
                _ => {
                    return Token {
                        token_type: TokenType::Error,
                        line: *current_line,
                    };
                }
            },
            8 => match c {
                '0'..='9' => {
                    state = 8;
                    token_string.push(c);
                }
                'e' | 'E' => {
                    state = 9;
                    token_string.push(c);
                }
                _ => {
                    *text_idx -= 1;
                    let float_value = token_string.parse::<f32>().unwrap_or(0.);
                    return Token {
                        token_type: TokenType::CtReal(float_value),
                        line: *current_line,
                    };
                }
            },

            9 => match c {
                '+' | '-' => {
                    state = 10;
                    token_string.push(c);
                }
                '0'..='9' => {
                    state = 11;
                    token_string.push(c);
                }
                _ => {
                    return Token {
                        token_type: TokenType::Error,
                        line: *current_line,
                    };
                }
            },
            10 => match c {
                '0'..='9' => {
                    state = 11;
                    token_string.push(c);
                }
                _ => {
                    return Token {
                        token_type: TokenType::Error,
                        line: *current_line,
                    };
                }
            },
            11 => match c {
                '0'..='9' => {
                    state = 11;
                    token_string.push(c);
                }
                _ => {
                    *text_idx -= 1;
                    let float_value = token_string.parse::<f32>().unwrap_or(0.);
                    return Token {
                        token_type: TokenType::CtReal(float_value),
                        line: *current_line,
                    };
                }
            },
            //Ct_Char
            21 => match c {
                '\\' => state = 22,
                _ => {
                    state = 24;
                    token_char = c;
                }
            },
            22 => match c {
                'a' => {
                    state = 23;
                    token_char = '\x07';
                }
                'b' => {
                    state = 23;
                    token_char = '\x08';
                }
                't' => {
                    state = 23;
                    token_char = '\x09';
                }
                'n' => {
                    state = 23;
                    token_char = '\x0A';
                }
                'v' => {
                    state = 23;
                    token_char = '\x0B';
                }
                'f' => {
                    state = 23;
                    token_char = '\x0C';
                }
                'r' => {
                    state = 23;
                    token_char = '\x0D';
                }
                '0' => {
                    state = 23;
                    token_char = '\0';
                }
                '?' | '\"' | '\'' | '\\' => {
                    state = 23;
                    token_char = c;
                }
                _ => {
                    *text_idx -= 1;
                    return Token {
                        token_type: TokenType::Error,
                        line: *current_line,
                    };
                }
            },
            23 => match c {
                '\'' => {
                    return Token {
                        token_type: TokenType::CtChar(token_char),
                        line: *current_line,
                    };
                }
                _ => {
                    *text_idx -= 1;
                    return Token {
                        token_type: TokenType::Error,
                        line: *current_line,
                    };
                }
            },
            24 => match c {
                '\'' => {
                    return Token {
                        token_type: TokenType::CtChar(token_char),
                        line: *current_line,
                    };
                }
                _ => {
                    *text_idx -= 1;
                    return Token {
                        token_type: TokenType::Error,
                        line: *current_line,
                    };
                }
            },
            // Ct_String
            25 => match c {
                '\\' => {
                    state = 26;
                }
                '\"' => {
                    return Token {
                        token_type: TokenType::CtString(String::from("")),
                        line: *current_line,
                    };
                }
                '\n' => {
                    //state = 28;
                    //token_string.push(c);
                    *current_line += 1;
                    //println!("No multiline string");
                    return Token {
                        token_type: TokenType::Error,
                        line: *current_line,
                    };
                }
                _ => {
                    state = 28;
                    token_string.push(c);
                }
            },
            26 => match c {
                'a' => {
                    state = 27;
                    token_string.push('\x07');
                }
                'b' => {
                    state = 27;
                    token_string.push('\x08');
                }
                't' => {
                    state = 27;
                    token_string.push('\x09');
                }
                'n' => {
                    state = 27;
                    token_string.push('\x0A');
                }
                'v' => {
                    state = 27;
                    token_string.push('\x0B');
                }
                'f' => {
                    state = 27;
                    token_string.push('\x0C');
                }
                'r' => {
                    state = 27;
                    token_string.push('\x0D');
                }
                '0' => {
                    state = 27;
                    token_string.push('\0');
                }
                '?' | '\"' | '\'' | '\\' => {
                    state = 27;
                    token_string.push(c);
                }
                _ => {
                    return Token {
                        token_type: TokenType::Error,
                        line: *current_line,
                    };
                }
            },
            27 => match c {
                '\\' => {
                    state = 26;
                }
                '\"' => {
                    return Token {
                        token_type: TokenType::CtString(token_string.clone()),
                        line: *current_line,
                    };
                }
                '\n' => {
                    // state = 28;
                    // token_string.push(c);
                    *current_line += 1;
                    //println!("No multiline string");
                    return Token {
                        token_type: TokenType::Error,
                        line: *current_line,
                    };
                }
                _ => {
                    token_string.push(c);
                    state = 28;
                }
            },
            28 => match c {
                '\\' => state = 26,
                '\"' => {
                    return Token {
                        token_type: TokenType::CtString(token_string.clone()),
                        line: *current_line,
                    };
                }
                '\n' => {
                    // state = 28;
                    // token_string.push(c);
                    *current_line += 1;
                    //println!("No multiline string");
                    return Token {
                        token_type: TokenType::Error,
                        line: *current_line,
                    };
                }
                _ => token_string.push(c),
            },
            _ => {
                println!("Invalid state");
                return Token {
                    token_type: TokenType::Error,
                    line: *current_line,
                };
            }
        }
    }
}

pub fn get_tokens(text: String) -> Vec<Token> {
    // Transform into chars
    let mut chars = text.chars().collect::<Vec<char>>();
    chars.push('\0');

    // Init token vector
    let mut token_vec: Vec<Token> = Vec::new();
    let mut text_idx = 0;
    let mut current_line = 1;

    loop {
        let t: Token = get_next_token(&chars, &mut text_idx, &mut current_line);
        token_vec.push(t.clone());
        match t.token_type {
            TokenType::End => break,
            _ => continue,
        }
    }
    return token_vec;
}
