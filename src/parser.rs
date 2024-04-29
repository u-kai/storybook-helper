use std::collections::HashMap;

use crate::{
    component::{
        Component, Key, NamedProps, ObjectType, Props, TSType, TSXContent, Type, UnionType,
    },
    lexer::Lexer,
    token::{TSXToken, TSXTokenType},
};

type TypeName = String;
type ExportName = String;

pub(super) struct ComponentPartsParser<'a> {
    lexer: Lexer<'a>,
    // TSXContent内の型情報を全て確保しておくもの
    type_buffer: HashMap<TypeName, Props>,
    component_candidates: HashMap<ExportName, TypeName>,
    peek: Option<TSXToken>,
}

// 1. typeを探す
// 2. componentを探す
// 3. componentのpropsを探す
// 4. propsの型を探す
// TODO :一旦propsなしで

impl ComponentPartsParser<'_> {
    pub fn new(content: &TSXContent) -> ComponentPartsParser {
        let lexer = Lexer::new(&content.0);
        let type_buffer = HashMap::new();
        ComponentPartsParser {
            lexer,
            type_buffer,
            component_candidates: HashMap::new(),
            peek: None,
        }
    }
    // TODO: export されているcomponentの名前しか見つけてない
    pub fn search_component(&mut self) -> Option<Component> {
        loop {
            let token = self.lexer.next_token();
            match token.token_type {
                // type TypeName = { KEY:TYPE }
                TSXTokenType::Type => {
                    let type_name = self.lexer.next_token();
                    assert_eq!(type_name.token_type, TSXTokenType::Ident);
                    let assign = self.lexer.next_token();
                    assert_eq!(assign.token_type, TSXTokenType::Assign);
                    let next = self.lexer.next_token();
                    match next.token_type {
                        TSXTokenType::LCurlyBracket => {
                            self.after_type_lcurl(type_name.literal.as_str());
                        }
                        _ => {}
                    }
                }
                // export default function NAME(props:Props)
                // export default const NAME = (props:Props) => {}
                // export default const NAME:IDENT_TYPE<Type> = (props:Props) => {}
                // export default const NAME = {}
                // export default const NAME = "hoge"
                // export default const NAME = 1
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
                            return self.from_props_lparen(name.literal.as_str());
                        }
                        TSXTokenType::Const => {
                            let name = self.lexer.next_token();
                            if let Some(component) = self.after_const_name(name.literal.as_str()) {
                                return Some(component);
                            };
                            break;
                        }
                        TSXTokenType::Ident => {
                            let name = function_or_const_or_name;
                            if let Some(component) = self.after_const_name(name.literal.as_str()) {
                                return Some(component);
                            };
                            break;
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
    fn get_type_value(&mut self) -> Type {
        // key: type_value_token
        let mut type_value_token = self.lexer.next_token();
        match type_value_token.token_type {
            TSXTokenType::Ident => {
                // < or | or & or ; or } or , or ident(next key)
                let next = self.lexer.next_token();
                match next.token_type {
                    TSXTokenType::Ident => {
                        self.peek = Some(next);
                        Type::Named(type_value_token.literal)
                    }
                    TSXTokenType::Semicolon | TSXTokenType::RCurlyBracket | TSXTokenType::Comma => {
                        self.peek = Some(next);
                        Type::Named(type_value_token.literal)
                    }
                    TSXTokenType::LTag => {
                        type_value_token.literal.push_str("<");
                        let mut next = self.lexer.next_token();
                        let mut rest_ltag_count = 1;
                        while next.token_type != TSXTokenType::RTag && rest_ltag_count != 0 {
                            type_value_token.literal.push_str(&next.literal);
                            next = self.lexer.next_token();
                            if next.token_type == TSXTokenType::LTag {
                                rest_ltag_count += 1;
                            }
                            if next.token_type == TSXTokenType::RTag {
                                rest_ltag_count -= 1;
                            }
                        }
                        type_value_token.literal.push_str(">");
                        Type::Named(type_value_token.literal)
                    }
                    TSXTokenType::Pipe => {
                        type_value_token.literal.push_str("|");
                        let mut next = self.lexer.next_token();
                        while next.token_type != TSXTokenType::Semicolon
                            && next.token_type != TSXTokenType::RCurlyBracket
                            && next.token_type != TSXTokenType::Comma
                        {
                            type_value_token.literal.push_str(&next.literal);
                            next = self.lexer.next_token();
                        }
                        Type::Named(type_value_token.literal)
                    }
                    TSXTokenType::And => {
                        type_value_token.literal.push_str("&");
                        let mut next = self.lexer.next_token();
                        while next.token_type != TSXTokenType::Semicolon
                            && next.token_type != TSXTokenType::RCurlyBracket
                            && next.token_type != TSXTokenType::Comma
                        {
                            type_value_token.literal.push_str(&next.literal);
                            next = self.lexer.next_token();
                        }
                        Type::Named(type_value_token.literal)
                    }
                    _ => {
                        panic!("unexpected token {:?}", next)
                    }
                }
            }
            // case func
            // (props:Props)=>Type;
            // ()=>Type;
            TSXTokenType::LParentheses => {
                let mut next = self.lexer.next_token();
                let mut type_value = String::from("(");
                while next.token_type != TSXTokenType::RParentheses {
                    type_value.push_str(&next.literal);
                    next = self.lexer.next_token();
                }
                type_value.push_str(")");
                let arrow = self.lexer.next_token();
                assert_eq!(arrow.token_type, TSXTokenType::Arrow);
                type_value.push_str(" => ");
                let return_type = self.lexer.next_token();
                type_value.push_str(&return_type.literal);
                Type::Named(type_value)
            }
            _ => panic!("unexpected token {:?}", type_value_token),
        }
    }
    fn after_type_lcurl(&mut self, type_name: &str) {
        let mut type_value = ObjectType::new();
        let key_or_rcurl = self.lexer.next_token();
        if key_or_rcurl.token_type == TSXTokenType::RCurlyBracket {
            self.type_buffer.insert(
                type_name.to_string(),
                Props::Named(NamedProps::new(type_name, type_value)),
            );
            return;
        }
        let mut key = key_or_rcurl;
        loop {
            println!("key {:?}", key);
            let colon_or_question = self.lexer.next_token();
            if colon_or_question.token_type == TSXTokenType::Question {
                let colon = self.lexer.next_token();
                key = TSXToken::new(TSXTokenType::Ident, format!("{}?", key.literal));
                assert_eq!(colon.token_type, TSXTokenType::Colon);
            }
            println!("colon_or_question {:?}", colon_or_question);
            let type_literal = self.get_type_value();
            println!("type_literal {:?}", type_literal);
            type_value.insert(Key(key.literal.clone()), type_literal);

            let comma_or_semicolon_or_rcurl_or_key = if self.peek.is_some() {
                let token = self.peek.take().unwrap();
                token
            } else {
                self.lexer.next_token()
            };
            match comma_or_semicolon_or_rcurl_or_key.token_type {
                TSXTokenType::Comma => {
                    let key_or_rcurl = self.lexer.next_token();
                    if key_or_rcurl.token_type == TSXTokenType::RCurlyBracket {
                        self.type_buffer.insert(
                            type_name.to_string(),
                            Props::Named(NamedProps::new(type_name, type_value)),
                        );
                        break;
                    }
                    key = key_or_rcurl;
                    continue;
                }
                TSXTokenType::Semicolon => {
                    let key_or_rcurl = self.lexer.next_token();
                    if key_or_rcurl.token_type == TSXTokenType::RCurlyBracket {
                        self.type_buffer.insert(
                            type_name.to_string(),
                            Props::Named(NamedProps::new(type_name, type_value)),
                        );
                        break;
                    }
                    key = key_or_rcurl;
                    continue;
                }
                TSXTokenType::Ident => {
                    key = comma_or_semicolon_or_rcurl_or_key;
                    continue;
                }
                TSXTokenType::RCurlyBracket => {
                    self.type_buffer.insert(
                        type_name.to_string(),
                        Props::Named(NamedProps::new(type_name, type_value)),
                    );
                    break;
                }
                // not implement yet
                TSXTokenType::LTag
                | TSXTokenType::RTag
                | TSXTokenType::Or
                | TSXTokenType::Pipe
                | TSXTokenType::And => {}
                _ => {
                    panic!("unexpected token {:?}", comma_or_semicolon_or_rcurl_or_key)
                }
            }
        }
    }
    // export const NAME:FC<Type> = (props:Props) => {}
    // export const NAME:VFC<Type> = (props:Props) => {}
    // export const NAME = (props:Props) => {}
    fn after_const_name(&mut self, component_name: &str) -> Option<Component> {
        let colon_or_eq = self.lexer.next_token();
        match colon_or_eq.token_type {
            TSXTokenType::Colon => {
                let type_name = self.lexer.next_token();
                // Not React Component
                if type_name.literal != "FC" {
                    return None;
                }
                let _lt = self.lexer.next_token();
                let type_name = self.lexer.next_token();
                let props = self.type_buffer.remove(&type_name.literal);
                if let Some(props) = props {
                    return Some(Component::new(component_name, props));
                }
                Some(Component::new(
                    component_name,
                    Props::Named(NamedProps::new(type_name.literal, ObjectType::new())),
                ))
            }
            TSXTokenType::Assign => {
                let lp = self.lexer.next_token();
                assert_eq!(lp.token_type, TSXTokenType::LParentheses);
                self.from_props_lparen(component_name)
            }
            _ => None,
        }
    }
    fn from_props_lparen(&mut self, component_name: &str) -> Option<Component> {
        let props_or_rpar = self.lexer.next_token();
        // props なし
        if props_or_rpar.token_type == TSXTokenType::RParentheses {
            return Some(Component::new(
                component_name,
                Props::Expand(ObjectType::new()),
            ));
        }
        let colon = self.lexer.next_token();
        assert_eq!(colon.token_type, TSXTokenType::Colon);
        let props_name_or_lcurl = self.lexer.next_token();
        // case props is named
        if props_name_or_lcurl.token_type == TSXTokenType::Ident {
            let props_name = props_name_or_lcurl;
            assert_eq!(props_name.token_type, TSXTokenType::Ident);
            let props = self.type_buffer.remove(&props_name.literal);
            if let Some(props) = props {
                return Some(Component::new(component_name, props));
            }
            return Some(Component::new(
                component_name,
                Props::Named(NamedProps::new(props_name.literal, ObjectType::new())),
            ));
        }
        // case props is expand
        let key_or_rcurl = self.lexer.next_token();
        // case {}
        if key_or_rcurl.token_type == TSXTokenType::RCurlyBracket {
            return Some(Component::new(
                component_name,
                Props::Expand(ObjectType::new()),
            ));
        }
        let mut type_value = ObjectType::new();
        let mut key = key_or_rcurl;
        loop {
            let colon_or_question = self.lexer.next_token();
            if colon_or_question.token_type == TSXTokenType::Question {
                let colon = self.lexer.next_token();
                key = TSXToken::new(TSXTokenType::Ident, format!("{}?", key.literal));
                assert_eq!(colon.token_type, TSXTokenType::Colon);
            }
            let type_literal = self.lexer.next_token();
            assert_eq!(type_literal.token_type, TSXTokenType::Ident);
            let mut comma_or_semicolon_or_rcurl_or_key_or_dot_or_ltag = self.lexer.next_token();
            while comma_or_semicolon_or_rcurl_or_key_or_dot_or_ltag.token_type
                != TSXTokenType::Comma
                || comma_or_semicolon_or_rcurl_or_key_or_dot_or_ltag.token_type
                    != TSXTokenType::Semicolon
                || comma_or_semicolon_or_rcurl_or_key_or_dot_or_ltag.token_type
                    != TSXTokenType::RCurlyBracket
            {
                if comma_or_semicolon_or_rcurl_or_key_or_dot_or_ltag.token_type == TSXTokenType::Dot
                {
                    let key = self.lexer.next_token();
                    assert_eq!(key.token_type, TSXTokenType::Ident);
                    let colon = self.lexer.next_token();
                    assert_eq!(colon.token_type, TSXTokenType::Colon);
                    let type_literal = self.lexer.next_token();
                    assert_eq!(type_literal.token_type, TSXTokenType::Ident);
                    comma_or_semicolon_or_rcurl_or_key_or_dot_or_ltag = self.lexer.next_token();
                    continue;
                }
                if comma_or_semicolon_or_rcurl_or_key_or_dot_or_ltag.token_type
                    == TSXTokenType::LTag
                {
                    let key = self.lexer.next_token();
                    assert_eq!(key.token_type, TSXTokenType::Ident);
                    let colon = self.lexer.next_token();
                    assert_eq!(colon.token_type, TSXTokenType::Colon);
                    let type_literal = self.lexer.next_token();
                    assert_eq!(type_literal.token_type, TSXTokenType::Ident);
                    let _rtag = self.lexer.next_token();
                    comma_or_semicolon_or_rcurl_or_key_or_dot_or_ltag = self.lexer.next_token();
                    continue;
                }
            }

            type_value.insert(
                Key(key.literal.clone()),
                Type::Named(type_literal.literal.clone()),
            );
            // match comma_or_semicolon_or_rcurl_or_key.token_type {
            //     TSXTokenType::Comma => {
            //         let key_or_rcurl = self.lexer.next_token();
            //         if key_or_rcurl.token_type == TSXTokenType::RCurlyBracket {
            //             return Some(Component::new(component_name, Props::Expand(type_value)));
            //         }
            //         key = key_or_rcurl;
            //         continue;
            //     }
            //     TSXTokenType::Semicolon => {
            //         let key_or_rcurl = self.lexer.next_token();
            //         if key_or_rcurl.token_type == TSXTokenType::RCurlyBracket {
            //             return Some(Component::new(component_name, Props::Expand(type_value)));
            //         }
            //         key = key_or_rcurl;
            //         continue;
            //     }
            //     TSXTokenType::Ident => {
            //         key = comma_or_semicolon_or_rcurl_or_key;
            //         continue;
            //     }
            //     TSXTokenType::RCurlyBracket => {
            //         return Some(Component::new(component_name, Props::Expand(type_value)));
            //     }
            //     _ => {
            //         panic!("unexpected token {:?}", comma_or_semicolon_or_rcurl_or_key)
            //     }
            // }
        }
        //fn after_colon_type_parser(&mut self) -> Option<Props> {}
    }
}

// lexer state must after read type keyword and type name and assign
struct TSTypeParser<'a> {
    // パラメータ時の無名な時も解析可能にしたいので、型名は含めない方がいい気がしてきた
    //type_name: String,
    lexer: &'a mut Lexer<'a>,
    peek: Option<TSXToken>,
}
//impl<'a> TSTypeParser<'a> {
//    fn new(lexer: &'a mut Lexer<'a>) -> Self {
//        Self {
//            lexer,
//            peek: None,
//        }
//    }
//
//    // TypeName = <TSType>
//    fn parse(&mut self) -> Option<TSType> {
//        // parse type
//        let start_type_token = self.lexer.next_token();
//        match start_type_token.token_type {
//            // case string literal
//            TSXTokenType::DoubleQuote => {}
//            TSXTokenType::SingleQuote => {} // case
//        }
//    }
//    fn case_string_literal(&mut self, delimiter: TSXTokenType) -> (TSType, Option<TSXToken>) {
//        let type_literal = self.lexer.next_token();
//        assert_eq!(type_literal.token_type, TSXTokenType::Ident);
//
//        let double_quote = self.lexer.next_token();
//        assert_eq!(double_quote.token_type, delimiter);
//
//        let literal = TSType::Literal(format!(
//            "{}{}{}",
//            delimiter.to_str(),
//            type_literal.literal,
//            delimiter.to_str()
//        ));
//        let next = self.lexer.next_token();
//        match next.token_type {
//            TSXTokenType::Comma => (literal, None),
//            TSXTokenType::Semicolon => (literal, None),
//            TSXTokenType::RCurlyBracket => (literal, None),
//            TSXTokenType::Ident => (literal, Some(next)),
//            // case union
//            TSXTokenType::Pipe => {
//                let mut union = UnionType::new();
//                union.push(literal);
//                let
//            }
//            _ => panic!("unexpected token {:?}", next),
//        }
//    }
//}

#[cfg(test)]
mod tests {
    use crate::{
        component::{
            Identifier, Key, NamedProps, ObjectType, ObjectTypes, PrimitiveType, Property, Props,
            Type,
        },
        lexer::Lexer,
    };

    use super::*;

    #[test]
    fn test_to_component4() {
        let content = r#"
import * as React from "react";
import AddIcon from "@mui/icons-material/Add";
import { Fab } from "@mui/material";

type ButtonProps = {
  generics: React<Hoge>
  noGenerics: React
};

export const RegisterButtons = (props: ButtonProps) => {
  return (
    <Fab color="primary" aria-label="add" size="small">
      <AddIcon sx={{}} onClick={props.handler}></AddIcon>
    </Fab>
  );
};
"#;
        let content = TSXContent(content.to_string());
        let component = content.to_component();
        let mut props = ObjectType::new();
        props.insert(
            Key("generics".to_string()),
            Type::Named("React<Hoge>".to_string()),
        );
        props.insert(
            Key("noGenerics".to_string()),
            Type::Named("React".to_string()),
        );
        let expect = Component::new(
            "RegisterButtons",
            Props::Named(NamedProps::new("ButtonProps", props)),
        );
        assert_eq!(component.unwrap(), expect);
    }

    //    #[test]
    //    fn test_parse_type_define() {
    //        let content = r#"
    //type Props = {
    //    timeOut: number;
    //    errorMessage?: string;
    //    generic: React<>;
    //    union: string | number;
    //};
    //"#;
    //        let mut lexer = Lexer::new(content);
    //        let _type_ident = lexer.next_token();
    //        let _type_name = lexer.next_token();
    //        let _assign = lexer.next_token();
    //
    //        let mut type_parser = TSTypeParser::new(&mut lexer);
    //        let type_define = type_parser.parse();
    //        let mut expect_type = ObjectTypes::new();
    //        expect_type.push(Property::new(
    //            Identifier::new("timeOut"),
    //            TSType::Primitive(PrimitiveType::Number),
    //        ));
    //        expect_type.push(Property::new(
    //            Identifier::new("errorMessage?"),
    //            TSType::Primitive(PrimitiveType::String),
    //        ));
    //        expect_type.push(Property::new(
    //            Identifier::new("genericFn"),
    //            TSType::Function(FunctionType::new(
    //                vec![Type::Named("T".to_string())],
    //                Type::Named("T".to_string()),
    //            )),
    //        ));
    //        let expect_type = TSType::Object(expect_type);
    //        assert_eq!(type_define, expect_type);
    //    }
    #[test]
    fn test_to_component3() {
        let content = r#"
import * as React from "react";
import AddIcon from "@mui/icons-material/Add";
import { Fab } from "@mui/material";

type ButtonProps = {
  handler: () => void;
};

export const RegisterButtons = (props: ButtonProps) => {
  return (
    <Fab color="primary" aria-label="add" size="small">
      <AddIcon sx={{}} onClick={props.handler}></AddIcon>
    </Fab>
  );
};
"#;
        let content = TSXContent(content.to_string());
        let component = content.to_component();
        let mut props = ObjectType::new();
        props.insert(
            Key("handler".to_string()),
            Type::Named("() => void".to_string()),
        );
        let expect = Component::new(
            "RegisterButtons",
            Props::Named(NamedProps::new("ButtonProps", props)),
        );
        assert_eq!(component.unwrap(), expect);
    }

    #[test]
    fn test_to_component2() {
        let content = r#"
import React from "react";
import { AppFooter } from "./elements/Footer";

export const Footer = () => {
  return <AppFooter></AppFooter>;
};
"#;
        let content = TSXContent(content.to_string());
        let component = content.to_component();
        let expect = Component::new("Footer", Props::Expand(ObjectType::new()));
        assert_eq!(component.unwrap(), expect);
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
        let mut props = ObjectType::new();
        props.insert(
            Key("timeOut".to_string()),
            Type::Named("number".to_string()),
        );
        props.insert(
            Key("errorMessage?".to_string()),
            Type::Named("string".to_string()),
        );
        props.insert(Key("size".to_string()), Type::Named("number".to_string()));
        let expect = Component::new("ErrorAlert", Props::Named(NamedProps::new("Props", props)));

        assert_eq!(component.unwrap(), expect);
    }
}
