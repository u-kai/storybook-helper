use std::str::Chars;

use crate::token::{TSXToken, TSXTokenType};

pub(super) struct Lexer<'a> {
    input: Chars<'a>,
    focus: char,
}
impl Lexer<'_> {
    pub fn new(input: &str) -> Lexer {
        let input = input.chars();
        let focus = ' ';
        Lexer { input, focus }
    }
    fn char_to_token(ch: char) -> TSXToken {
        match ch {
            '+' => TSXToken::new(TSXTokenType::Plus, ch),
            '?' => TSXToken::new(TSXTokenType::Question, ch),
            '-' => TSXToken::new(TSXTokenType::Minus, ch),
            '!' => TSXToken::new(TSXTokenType::Bang, ch),
            '*' => TSXToken::new(TSXTokenType::Asterisk, ch),
            '/' => TSXToken::new(TSXTokenType::Slash, ch),
            '<' => TSXToken::new(TSXTokenType::LTag, ch),
            '>' => TSXToken::new(TSXTokenType::RTag, ch),
            ',' => TSXToken::new(TSXTokenType::Comma, ch),
            ';' => TSXToken::new(TSXTokenType::Semicolon, ch),
            '(' => TSXToken::new(TSXTokenType::LParentheses, ch),
            ')' => TSXToken::new(TSXTokenType::RParentheses, ch),
            '{' => TSXToken::new(TSXTokenType::LCurlyBracket, ch),
            '}' => TSXToken::new(TSXTokenType::RCurlyBracket, ch),
            '[' => TSXToken::new(TSXTokenType::LBracket, ch),
            ']' => TSXToken::new(TSXTokenType::RBracket, ch),
            '=' => TSXToken::new(TSXTokenType::Eq, ch),
            ':' => TSXToken::new(TSXTokenType::Colon, ch),
            '.' => TSXToken::new(TSXTokenType::Dot, ch),
            '"' => TSXToken::new(TSXTokenType::DoubleQuote, ch),
            '\'' => TSXToken::new(TSXTokenType::SingleQuote, ch),
            '|' => TSXToken::new(TSXTokenType::Pipe, ch),
            '&' => TSXToken::new(TSXTokenType::And, ch),
            c => TSXToken::new(TSXTokenType::Ident, c),
        }
    }
    pub fn next_token(&mut self) -> TSXToken {
        self.skip_whitespace();
        match self.focus {
            // effect only one char
            ',' | ';' | '(' | ')' | '{' | '}' | ':' | '#' | '.' | '&' => {
                let token = Self::char_to_token(self.focus);
                self.set_next_char();
                token
            }
            '|' => {
                if self.set_next_char() && self.focus == '|' {
                    self.set_next_char();
                    TSXToken::new(TSXTokenType::Or, "||")
                } else {
                    TSXToken::new(TSXTokenType::Pipe, "|")
                }
            }
            '!' => {
                if self.set_next_char() && self.focus == '=' {
                    self.set_next_char();
                    TSXToken::new(TSXTokenType::NotEq, "!=")
                } else {
                    TSXToken::new(TSXTokenType::Bang, "!")
                }
            }
            '?' => {
                self.set_next_char();
                match self.focus {
                    '?' => {
                        self.set_next_char();
                        TSXToken::new(TSXTokenType::NullishCoalescing, "??")
                    }
                    _ => TSXToken::new(TSXTokenType::Question, "?"),
                }
            }
            '=' => {
                self.set_next_char();
                match self.focus {
                    '=' => {
                        self.set_next_char();
                        TSXToken::new(TSXTokenType::Eq, "==")
                    }
                    '>' => {
                        self.set_next_char();
                        TSXToken::new(TSXTokenType::Arrow, "=>")
                    }
                    _ => TSXToken::new(TSXTokenType::Assign, "="),
                }
            }
            '<' => {
                self.set_next_char();
                match self.focus {
                    '=' => {
                        self.set_next_char();
                        TSXToken::new(TSXTokenType::LtEq, "<=")
                    }
                    '/' => {
                        self.set_next_char();
                        TSXToken::new(TSXTokenType::CloseLTag, "</")
                    }
                    _ => TSXToken::new(TSXTokenType::LTag, "<"),
                }
            }
            '>' => {
                self.set_next_char();
                match self.focus {
                    '=' => {
                        self.set_next_char();
                        TSXToken::new(TSXTokenType::GtEq, ">=")
                    }
                    _ => TSXToken::new(TSXTokenType::RTag, ">"),
                }
            }
            '/' => {
                self.set_next_char();
                match self.focus {
                    '/' => TSXToken::new(TSXTokenType::Comment, self.read_comment()),
                    '*' => {
                        self.set_next_char();
                        TSXToken::new(TSXTokenType::StartDocComment, "/*")
                    }
                    '=' => {
                        self.set_next_char();
                        TSXToken::new(TSXTokenType::SlashEqual, "/=")
                    }
                    '>' => {
                        self.set_next_char();
                        TSXToken::new(TSXTokenType::CloseRTag, "/>")
                    }
                    _ => TSXToken::new(TSXTokenType::Slash, "/"),
                }
            }
            '*' => {
                self.set_next_char();
                match self.focus {
                    '/' => {
                        self.set_next_char();
                        let comment = self.read_doc_comment();
                        TSXToken::new(TSXTokenType::EndDocComment, comment)
                    }
                    '=' => {
                        self.set_next_char();
                        TSXToken::new(TSXTokenType::CrossEqual, "*=")
                    }
                    _ => TSXToken::new(TSXTokenType::Asterisk, "*"),
                }
            }
            '+' => {
                self.set_next_char();
                match self.focus {
                    '+' => {
                        self.set_next_char();
                        TSXToken::new(TSXTokenType::Increment, "++")
                    }
                    '=' => {
                        self.set_next_char();
                        TSXToken::new(TSXTokenType::IncrementEqual, "+=")
                    }
                    _ => TSXToken::new(TSXTokenType::Add, "+"),
                }
            }
            '-' => {
                self.set_next_char();
                match self.focus {
                    '-' => {
                        self.set_next_char();
                        TSXToken::new(TSXTokenType::Decrement, "--")
                    }
                    '=' => {
                        self.set_next_char();
                        TSXToken::new(TSXTokenType::DecrementEqual, "-=")
                    }
                    _ => TSXToken::new(TSXTokenType::Sub, "-"),
                }
            }
            '"' => {
                let literal = self.read_string();
                TSXToken::new(TSXTokenType::StringLiteral, literal)
            }
            '\'' => {
                self.set_next_char();
                let literal = self.read_string();
                TSXToken::new(TSXTokenType::StringLiteral, literal)
            }
            c => {
                if Self::is_letter(c) {
                    let literal = self.read_word();
                    return if let Some(token) = tsx_keywords(literal.as_str()) {
                        token
                    } else {
                        TSXToken::new(TSXTokenType::Ident, literal)
                    };
                }
                if Self::is_number(c) {
                    let literal = self.read_number();
                    TSXToken::new(TSXTokenType::NumberLiteral, literal)
                } else {
                    self.set_next_char();
                    TSXToken::new(TSXTokenType::Eof, "")
                }
            }
        }
    }
    fn skip_whitespace(&mut self) {
        while self.focus.is_whitespace() && self.set_next_char() {}
    }

    // only use when focus is doc comment token
    fn read_doc_comment(&mut self) -> String {
        let mut comment = String::new();
        while self.focus != '*' && self.set_next_char() {
            comment.push(self.focus);
        }
        if self.set_next_char() && self.focus == '/' {
            self.set_next_char();
        }
        comment
    }
    // only use when focus is comment token
    fn read_comment(&mut self) -> String {
        let mut comment = String::new();
        while self.focus != '\n' && self.set_next_char() {
            comment.push(self.focus);
        }
        comment
    }
    fn read_string(&mut self) -> String {
        let mut string = String::new();
        while self.set_next_char() && self.focus != '"' {
            string.push(self.focus);
        }
        self.set_next_char();
        string
    }
    fn read_word(&mut self) -> String {
        let mut word = String::new();
        // first char is letter(not number or symbol)
        if Self::is_letter(self.focus) {
            word.push(self.focus);
            self.set_next_char();
        }
        // next char is letter or number
        while Self::is_letter(self.focus) || Self::is_number(self.focus) {
            word.push(self.focus);
            self.set_next_char();
        }
        word
    }
    fn read_number(&mut self) -> String {
        let mut number = String::new();
        // next char is number
        while Self::is_number(self.focus) {
            number.push(self.focus);
            self.set_next_char();
        }
        number
    }
    fn is_number(ch: char) -> bool {
        ch.is_numeric()
    }
    fn is_letter(ch: char) -> bool {
        ch.is_alphabetic() || ch == '_'
    }
    fn set_next_char(&mut self) -> bool {
        if let Some(c) = self.input.next() {
            self.focus = c;
            true
        } else {
            self.focus = ' ';
            false
        }
    }
}

fn tsx_keywords(s: &str) -> Option<TSXToken> {
    match s {
        "type" => Some(TSXToken::new(TSXTokenType::Type, s)),
        "export" => Some(TSXToken::new(TSXTokenType::Export, s)),
        "import" => Some(TSXToken::new(TSXTokenType::Import, s)),
        "default" => Some(TSXToken::new(TSXTokenType::Default, s)),
        "const" => Some(TSXToken::new(TSXTokenType::Const, s)),
        "let" => Some(TSXToken::new(TSXTokenType::Let, s)),
        "return" => Some(TSXToken::new(TSXTokenType::Return, s)),
        "if" => Some(TSXToken::new(TSXTokenType::If, s)),
        "else" => Some(TSXToken::new(TSXTokenType::Else, s)),
        "true" => Some(TSXToken::new(TSXTokenType::True, s)),
        "false" => Some(TSXToken::new(TSXTokenType::False, s)),
        "function" => Some(TSXToken::new(TSXTokenType::Fn, s)),
        "class" => Some(TSXToken::new(TSXTokenType::Class, s)),
        "var" => Some(TSXToken::new(TSXTokenType::Var, s)),
        "from" => Some(TSXToken::new(TSXTokenType::From, s)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_lexer() {
        let content = r#"
import { Alert, AlertTitle } from "@mui/material";

type Props = {
  timeOut: number;
  errorMessage?: string;
  size: number; 
};
export const ErrorAlert: FC<Props> = (props: Props) => {}
"#;
        let mut lexer = Lexer::new(content);
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Import, "import")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::LCurlyBracket, "{")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Ident, "Alert")
        );
        assert_eq!(lexer.next_token(), TSXToken::new(TSXTokenType::Comma, ","));
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Ident, "AlertTitle")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::RCurlyBracket, "}")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::From, "from")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::StringLiteral, "@mui/material")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Semicolon, ";")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Type, "type")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Ident, "Props")
        );
        assert_eq!(lexer.next_token(), TSXToken::new(TSXTokenType::Assign, "="));
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::LCurlyBracket, "{")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Ident, "timeOut")
        );
        assert_eq!(lexer.next_token(), TSXToken::new(TSXTokenType::Colon, ":"));
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Ident, "number")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Semicolon, ";")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Ident, "errorMessage")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Question, "?")
        );
        assert_eq!(lexer.next_token(), TSXToken::new(TSXTokenType::Colon, ":"));
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Ident, "string")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Semicolon, ";")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Ident, "size")
        );
        assert_eq!(lexer.next_token(), TSXToken::new(TSXTokenType::Colon, ":"));
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Ident, "number")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Semicolon, ";")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::RCurlyBracket, "}")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Semicolon, ";")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Export, "export")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Const, "const")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Ident, "ErrorAlert")
        );
        assert_eq!(lexer.next_token(), TSXToken::new(TSXTokenType::Colon, ":"));
        assert_eq!(lexer.next_token(), TSXToken::new(TSXTokenType::Ident, "FC"));
        assert_eq!(lexer.next_token(), TSXToken::new(TSXTokenType::LTag, "<"));
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Ident, "Props")
        );
        assert_eq!(lexer.next_token(), TSXToken::new(TSXTokenType::RTag, ">"));
        assert_eq!(lexer.next_token(), TSXToken::new(TSXTokenType::Assign, "="));
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::LParentheses, "(")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Ident, "props")
        );
        assert_eq!(lexer.next_token(), TSXToken::new(TSXTokenType::Colon, ":"));
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::Ident, "Props")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::RParentheses, ")")
        );
        assert_eq!(lexer.next_token(), TSXToken::new(TSXTokenType::Arrow, "=>"));
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::LCurlyBracket, "{")
        );
        assert_eq!(
            lexer.next_token(),
            TSXToken::new(TSXTokenType::RCurlyBracket, "}")
        );
        assert_eq!(lexer.next_token(), TSXToken::new(TSXTokenType::Eof, ""));
    }
}
