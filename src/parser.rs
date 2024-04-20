use std::{collections::HashMap, path::Path, str::Chars};

use crate::component::{Component, ExpandProps, Key, NamedProps, Props, Type};

/*

import { Alert, AlertTitle } from "@mui/material";
import React, { FC, useContext, useEffect } from "react";
import { AppErrorContext } from "../../contexts/error";

type Props = {
  timeOut: number;
  errorMessage?: string;
};

// propsは何かしらの明示された型か、リテラルなタイプ、もしくは空か
// 何かしらの明示された型であればその型のプロパティを再起的に拾ってくる必要がある
// そのファイルのコンポーネントはexport defaultかexport constのどちらかで、export constである場合は、関数である可能性もあるので、それがコンポーネントであるのかどうかの判定は必要

export const ErrorAlert: FC<Props> = (props: Props) => {
  const { appError, setAppError } = useContext(AppErrorContext);
  useEffect(() => {
    if (appError !== undefined) {
      setTimeout(() => {
        setAppError(undefined);
      }, props.timeOut);
    }
  }, [appError, setAppError]);

  return (
    <>
      {appError !== undefined || props.errorMessage ? (
        <Alert severity="error">
          <AlertTitle>Error</AlertTitle>
          {props.errorMessage ?? appError.message}
        </Alert>
      ) : null}
    </>
  );
};
*/

#[derive(Debug, PartialEq)]
struct TSXToken {
    token_type: TSXTokenType,
    literal: String,
}
impl TSXToken {
    fn new(token_type: TSXTokenType, literal: impl Into<String>) -> Self {
        Self {
            token_type,
            literal: literal.into(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum TSXTokenType {
    Comment,
    StartDocComment,
    EndDocComment,
    Increment,
    Add,
    Sub,
    Colon,
    Dot,
    DoubleQuote,
    SingleQuote,
    CrossEqual,
    From,
    IncrementEqual,
    DecrementEqual,
    SlashEqual,
    Decrement,
    Assign,
    Eof,
    NumberLiteral,
    Ident,
    Plus,
    Minus,
    Bang,
    Asterisk,
    Slash,
    LTag,
    RTag,
    CloseLTag,
    CloseRTag,
    LtEq,
    GtEq,
    Comma,
    Semicolon,
    LParentheses,
    RParentheses,
    LCurlyBracket,
    RCurlyBracket,
    LBracket,
    RBracket,
    Eq,
    NotEq,
    Fn,
    True,
    False,
    If,
    Else,
    Return,
    Let,
    Var,
    Const,
    Question,
    NullishCoalescing,
    Type,
    StringLiteral,
    Class,
    Export,
    Import,
    Default,
    Arrow,
    String,
    Number,
    Boolean,
    Undefined,
}

pub(crate) struct TSXContent(String);

impl TSXContent {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self(content))
    }
    pub fn to_component(&self) -> Option<Component> {
        let mut parser = ComponentPartsParser::new(self);
        parser.search_component()
    }
}

type TypeName = String;
type ExportName = String;

struct ComponentPartsParser<'a> {
    lexer: Lexer<'a>,
    // TSXContent内の型情報を全て確保しておくもの
    type_buffer: HashMap<TypeName, Type>,
    component_candidates: HashMap<ExportName, TypeName>,
}

// 1. typeを探す
// 2. componentを探す
// 3. componentのpropsを探す
// 4. propsの型を探す
// TODO :一旦propsなしで

impl ComponentPartsParser<'_> {
    fn new(content: &TSXContent) -> ComponentPartsParser {
        let lexer = Lexer::new(&content.0);
        let type_buffer = HashMap::new();
        ComponentPartsParser {
            lexer,
            type_buffer,
            component_candidates: HashMap::new(),
        }
    }
    // TODO: export されているcomponentの名前しか見つけてない
    fn search_component(&mut self) -> Option<Component> {
        loop {
            let token = self.lexer.next_token();
            match token.token_type {
                // type TypeName = { KEY:TYPE }
                TSXTokenType::Type => {
                    let type_name = self.lexer.next_token();
                    let _assign = self.lexer.next_token();
                    let _lcurl = self.lexer.next_token();
                    let mut type_value = HashMap::new();
                    let mut key = self.lexer.next_token();
                    loop {
                        let _colon = self.lexer.next_token();
                        let type_name = self.lexer.next_token();
                        type_value.insert(key.literal.clone(), type_name.literal);
                        let comma_or_semicolon_or_rcurl_or_key = self.lexer.next_token();
                        match comma_or_semicolon_or_rcurl_or_key.token_type {
                            TSXTokenType::Comma => {
                                key = self.lexer.next_token();
                                continue;
                            }
                            TSXTokenType::Semicolon => {
                                key = self.lexer.next_token();
                                continue;
                            }
                            TSXTokenType::RCurlyBracket => {
                                key = self.lexer.next_token();
                                break;
                            }
                            TSXTokenType::Ident => {
                                key = comma_or_semicolon_or_rcurl_or_key;
                                continue;
                            }
                            _ => {}
                        }
                    }
                    // TODO
                    //self.type_buffer.insert(type_name, type_value);
                }
                // export default function NAME(props:Props)
                // export default const NAME
                // export default NAME
                // export const NAME
                TSXTokenType::Export => {
                    let default_or_const_type = self.lexer.next_token();
                    if default_or_const_type.token_type == TSXTokenType::Type {
                        continue;
                    }
                    let function_or_const_or_name = self.lexer.next_token();
                    match function_or_const_or_name.token_type {
                        TSXTokenType::Fn => {
                            let name = self.lexer.next_token();
                            return Some(Component::new(
                                name.literal,
                                Props::Expand(ExpandProps::new()),
                            ));
                        }
                        TSXTokenType::Const => {
                            let name = self.lexer.next_token();
                            return Some(Component::new(
                                name.literal,
                                Props::Expand(ExpandProps::new()),
                            ));
                        }
                        TSXTokenType::Ident => {
                            let name = function_or_const_or_name;
                            return Some(Component::new(
                                name.literal,
                                Props::Expand(ExpandProps::new()),
                            ));
                        }
                        _ => {}
                    }
                }
                TSXTokenType::Eof => {
                    break;
                }
                _ => {}
            }
        }
        None
    }
}
struct Lexer<'a> {
    input: Chars<'a>,
    focus: char,
}
impl Lexer<'_> {
    fn new(input: &str) -> Lexer {
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
            c => TSXToken::new(TSXTokenType::Ident, c),
        }
    }
    pub fn next_token(&mut self) -> TSXToken {
        self.skip_whitespace();
        match self.focus {
            // effect only one char
            ',' | ';' | '(' | ')' | '{' | '}' | ':' | '#' => {
                let token = Self::char_to_token(self.focus);
                self.set_next_char();
                token
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
    use crate::component::{ExpandProps, Key, NamedProps, Props, Type};

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
    #[test]
    fn test_to_component() {
        let content = r#"
    type Props = {
      timeOut: number;
      errorMessage?: string;
      size: number; 
    };
    export const ErrorAlert: FC<Props> = (props: Props) => {
      const { appError, setAppError } = useContext(AppErrorContext);
      useEffect(() => {
        if (appError !== undefined) {
          setTimeout(() => {
            setAppError(undefined);
          }, props.timeOut);
        }
      }, [appError, setAppError]);
    
      return (
        <>
          {appError !== undefined || props.errorMessage ? (
            <Alert severity="error">
              <AlertTitle>Error</AlertTitle>
              {props.errorMessage ?? appError.message}
            </Alert>
          ) : null}
        </>
      );
    };
    "#;
        let content = TSXContent(content.to_string());
        let component = content.to_component();
        let mut props = ExpandProps::new();
        props.insert(Key("timeOut".to_string()), Type::Number);
        props.insert(Key("errorMessage".to_string()), Type::String);
        props.insert(Key("size".to_string()), Type::Number);
        let expect = Component::new("ErrorAlert", Props::Named(NamedProps::new("Props", props)));

        assert_eq!(component.unwrap(), expect);
    }
}
