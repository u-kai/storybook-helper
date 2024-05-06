use std::collections::HashMap;

use crate::{
    component::{Component, Key, NamedProps, ObjectType, Props, TSXContent, Type},
    lexer::Lexer,
    token::{TSXToken, TSXTokenType},
};

type TypeName = String;

pub(super) struct ComponentPartsParser<'a> {
    lexer: Lexer<'a>,
    // TSXContent内の型情報を全て確保しておくもの
    type_buffer: HashMap<TypeName, Props>,
    peek: Option<TSXToken>,
}

// 1. typeを探す
// 2. componentを探す
// 3. componentのpropsを探す
// 4. propsの型を探す

impl ComponentPartsParser<'_> {
    pub fn new(content: &TSXContent) -> ComponentPartsParser {
        let lexer = Lexer::new(&content.0);
        let type_buffer = HashMap::new();
        ComponentPartsParser {
            lexer,
            type_buffer,
            peek: None,
        }
    }
    // TODO: export されているcomponentの名前しか見つけてない
    pub fn search_component(&mut self) -> Option<Component> {
        loop {
            let token = self.peek.take().unwrap_or_else(|| self.lexer.next_token());
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
                // export const NAME = (props:{key:value,...}) => {}
                // export const NAME = Value
                // export type
                TSXTokenType::Export => {
                    let default_or_const_type = self.lexer.next_token();
                    if default_or_const_type.token_type == TSXTokenType::Type {
                        self.peek = Some(default_or_const_type);
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
    // :を取得したタイミングで利用する
    fn get_type_value(&mut self, ltag_num: usize) -> Type {
        fn add_after_type_literal(this: &mut ComponentPartsParser, token: &mut TSXToken) {
            let mut next = this.lexer.next_token();
            while next.token_type != TSXTokenType::Semicolon
                && next.token_type != TSXTokenType::RCurlyBracket
                && next.token_type != TSXTokenType::Comma
            {
                token.literal.push_str(&next.literal);
                next = this.lexer.next_token();
            }
        }

        // key: type_value_token
        let mut type_value_token = self.lexer.next_token();
        match type_value_token.token_type {
            TSXTokenType::Ident => {
                // < or | or & or ; or } or , or ident(next key) or .
                let next = self.lexer.next_token();
                match next.token_type {
                    TSXTokenType::Ident | TSXTokenType::RCurlyBracket => {
                        self.peek = Some(next);
                        Type::Alias(type_value_token.literal)
                    }
                    TSXTokenType::Semicolon | TSXTokenType::Comma => {
                        Type::Alias(type_value_token.literal)
                    }
                    TSXTokenType::Dot => {
                        let after_type_value = self.get_type_value(ltag_num);
                        Type::Alias(format!(
                            "{}.{}",
                            type_value_token.literal,
                            after_type_value.to_str()
                        ))
                    }
                    TSXTokenType::RTag => {
                        type_value_token.literal.push_str(">");
                        let next = self.lexer.next_token();
                        match next.token_type {
                            TSXTokenType::Semicolon | TSXTokenType::Comma => {}
                            TSXTokenType::Ident | TSXTokenType::RCurlyBracket => {
                                self.peek = Some(next);
                            }
                            TSXTokenType::RTag => {
                                return if ltag_num == 2 {
                                    type_value_token.literal.push_str(">");
                                    let next = self.lexer.next_token();
                                    match next.token_type {
                                        TSXTokenType::Semicolon | TSXTokenType::Comma => {}
                                        TSXTokenType::Ident | TSXTokenType::RCurlyBracket => {
                                            self.peek = Some(next);
                                        }
                                        TSXTokenType::Pipe => {
                                            type_value_token.literal.push_str("|");
                                            add_after_type_literal(self, &mut type_value_token);
                                        }
                                        _ => {
                                            panic!("unexpected token {:?}", next)
                                        }
                                    }
                                    Type::Alias(type_value_token.literal)
                                } else {
                                    self.get_type_value(ltag_num - 2)
                                };
                            }
                            _ => {
                                panic!("unexpected token {:?}", next)
                            }
                        };
                        if ltag_num == 1 {
                            Type::Alias(type_value_token.literal)
                        } else {
                            self.get_type_value(ltag_num - 1)
                        }
                    }
                    TSXTokenType::LTag => {
                        type_value_token.literal.push_str("<");
                        let after_type_value_token = self.get_type_value(ltag_num + 1);
                        type_value_token
                            .literal
                            .push_str(&after_type_value_token.to_str());
                        Type::Alias(type_value_token.literal)
                    }
                    TSXTokenType::Pipe => {
                        type_value_token.literal.push_str("|");
                        add_after_type_literal(self, &mut type_value_token);
                        Type::Alias(type_value_token.literal)
                    }
                    TSXTokenType::And => {
                        type_value_token.literal.push_str("&");
                        add_after_type_literal(self, &mut type_value_token);
                        Type::Alias(type_value_token.literal)
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
                let mut count = 1;
                while next.token_type != TSXTokenType::RParentheses && count != 0 {
                    type_value.push_str(&next.literal);
                    next = self.lexer.next_token();
                    match next.token_type {
                        TSXTokenType::LParentheses => {
                            count += 1;
                        }
                        TSXTokenType::RParentheses => {
                            count -= 1;
                        }
                        _ => {}
                    }
                }
                type_value.push_str(")");
                let arrow = self.lexer.next_token();
                assert_eq!(arrow.token_type, TSXTokenType::Arrow);
                type_value.push_str(" => ");
                let return_type_value = self.get_type_value(0);
                type_value.push_str(&return_type_value.to_str());
                Type::Alias(type_value)
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
            let colon_or_question = self.lexer.next_token();
            if colon_or_question.token_type == TSXTokenType::Question {
                let colon = self.lexer.next_token();
                key = TSXToken::new(TSXTokenType::Ident, format!("{}?", key.literal));
                assert_eq!(colon.token_type, TSXTokenType::Colon);
            }
            let type_literal = self.get_type_value(0);
            type_value.insert(Key(key.literal.clone()), type_literal);

            let key_or_rcurl = self.peek.take().unwrap_or_else(|| self.lexer.next_token());
            match key_or_rcurl.token_type {
                TSXTokenType::Ident => {
                    key = key_or_rcurl;
                    continue;
                }
                TSXTokenType::RCurlyBracket => {
                    self.peek = Some(self.lexer.next_token());
                    while Some(TSXTokenType::Semicolon)
                        == self.peek.as_ref().map(|t| t.token_type.clone())
                    {
                        self.peek = Some(self.lexer.next_token());
                    }
                    match self.peek.as_ref().map(|t| t.token_type.clone()) {
                        Some(TSXTokenType::Pipe) => {
                            self.lexer.next_token();
                        }
                        Some(TSXTokenType::And) => {
                            let indent = self.lexer.next_token();
                            assert_eq!(indent.token_type, TSXTokenType::Ident);
                            // TODO:全然できてない.一つだけの&であればOK
                            // それ以外はpanic
                            self.type_buffer.insert(
                                type_name.to_string(),
                                Props::Named(NamedProps::new_intersection_type(
                                    type_name,
                                    vec![
                                        Type::Object(type_value),
                                        Type::Alias(indent.literal.clone()),
                                    ],
                                )),
                            );
                            break;
                        }
                        _ => {}
                    }
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
                    panic!("unexpected token {:?}", key_or_rcurl)
                }
            }
        }
    }
    // export const NAME:React.FC<Type> = (props:Props) => {}
    // export const NAME:FC<Type> = (props:Props) => {}
    // export const NAME:VFC<Type> = (props:Props) => {}
    // export const NAME = (props:Props) => {}
    // export const NAME = (props:{key:value....}) => {}
    fn after_const_name(&mut self, component_name: &str) -> Option<Component> {
        let colon_or_eq = self.lexer.next_token();
        match colon_or_eq.token_type {
            TSXTokenType::Colon => {
                fn case_fc_or_vfc(
                    this: &mut ComponentPartsParser,
                    focus_token: &TSXToken,
                    component_name: &str,
                ) -> Option<Component> {
                    if focus_token.literal == "FC" || focus_token.literal == "VFC" {
                        let _lt = this.lexer.next_token();
                        let type_name = this.lexer.next_token();
                        let props = this.type_buffer.remove(&type_name.literal);
                        if let Some(props) = props {
                            return Some(Component::new(component_name, props));
                        }
                        return Some(Component::new(
                            component_name,
                            Props::Named(NamedProps::new(type_name.literal, ObjectType::new())),
                        ));
                    };
                    None
                }
                let type_name = self.lexer.next_token();
                if let Some(component) = case_fc_or_vfc(self, &type_name, component_name) {
                    return Some(component);
                }
                if type_name.literal == "React" {
                    let _dot = self.lexer.next_token();
                    let _fc_or_vfc = self.lexer.next_token();
                    if let Some(component) = case_fc_or_vfc(self, &_fc_or_vfc, component_name) {
                        return Some(component);
                    }
                }
                None
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

        let mut key = key_or_rcurl;
        let mut obj = ObjectType::new();
        while key.token_type != TSXTokenType::RCurlyBracket {
            let colon_or_question = self.lexer.next_token();
            let obj_key = if colon_or_question.token_type == TSXTokenType::Question {
                let colon = self.lexer.next_token();
                assert_eq!(colon.token_type, TSXTokenType::Colon);
                Key(format!("{}?", key.literal))
            } else {
                Key(key.literal.clone())
            };
            let type_value = self.get_type_value(0);
            obj.insert(obj_key, type_value);
            key = self.lexer.next_token();
        }

        Some(Component::new(component_name, Props::Expand(obj)))
    }
}

#[cfg(test)]
mod tests {
    use crate::component::{Key, NamedProps, ObjectType, Props, Type};
    #[test]
    fn test_to_and_type() {
        let content = r#"
import React from "react";
import { InputField, InputFieldProps } from "./InputField";
import { styled } from "styled-components";

export type InputFieldWithButtonProps = {
  button: React.ReactNode;
} & InputFieldProps;

export const InputFieldWithButton = (props: InputFieldWithButtonProps) => {
  return (
    <Container>
      <InputField {...props} />
      {props.button}
    </Container>
  );
};

const Container = styled.div`
  display: flex;
  flex-direction: row;
  position: relative;
`;
"#;
        let content = TSXContent(content.to_string());
        let component = content.to_component();
        let mut props = ObjectType::new();
        props.insert(
            Key("button".to_string()),
            Type::Alias("React.ReactNode".to_string()),
        );
        let expect = Component::new(
            "InputFieldWithButton",
            Props::Named(NamedProps::new_intersection_type(
                "InputFieldWithButtonProps",
                vec![
                    Type::Object(props),
                    Type::Alias("InputFieldProps".to_string()),
                ],
            )),
        );
        assert_eq!(component.unwrap(), expect);
    }

    use super::*;
    #[test]
    fn test_to_union_generic() {
        let content = r#"
import * as React from "react";
import AddIcon from "@mui/icons-material/Add";
import { Fab } from "@mui/material";

export const DeleteConfirmModal = (props: {
  deleteHandler: () => Promise<void>;
  setOpen: React.Dispatch<React.SetStateAction<boolean>> | union;
  open: boolean;
}) => {
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
            Key("deleteHandler".to_string()),
            Type::Alias("() => Promise<void>".to_string()),
        );
        props.insert(Key("open".to_string()), Type::Alias("boolean".to_string()));
        props.insert(
            Key("setOpen".to_string()),
            Type::Alias("React.Dispatch<React.SetStateAction<boolean>>|union".to_string()),
        );
        let expect = Component::new("DeleteConfirmModal", Props::Expand(props));
        assert_eq!(component.unwrap(), expect);
    }
    #[test]
    fn test_to_func_generic() {
        let content = r#"
import * as React from "react";
import AddIcon from "@mui/icons-material/Add";
import { Fab } from "@mui/material";

export const DeleteConfirmModal = (props: {
  deleteHandler: () => Promise<void>;
  open: boolean;
  setOpen: React.Dispatch<React.SetStateAction<boolean>>;
}) => {
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
            Key("deleteHandler".to_string()),
            Type::Alias("() => Promise<void>".to_string()),
        );
        props.insert(Key("open".to_string()), Type::Alias("boolean".to_string()));
        props.insert(
            Key("setOpen".to_string()),
            Type::Alias("React.Dispatch<React.SetStateAction<boolean>>".to_string()),
        );
        let expect = Component::new("DeleteConfirmModal", Props::Expand(props));
        assert_eq!(component.unwrap(), expect);
    }

    #[test]
    fn test_to_component4() {
        let content = r#"
import * as React from "react";
import AddIcon from "@mui/icons-material/Add";
import { Fab } from "@mui/material";

export type ButtonProps = {
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
            Type::Alias("React<Hoge>".to_string()),
        );
        props.insert(
            Key("noGenerics".to_string()),
            Type::Alias("React".to_string()),
        );
        let expect = Component::new(
            "RegisterButtons",
            Props::Named(NamedProps::new_object_type("ButtonProps", props)),
        );
        assert_eq!(component.unwrap(), expect);
    }

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
            Type::Alias("() => void".to_string()),
        );
        let expect = Component::new(
            "RegisterButtons",
            Props::Named(NamedProps::new_object_type("ButtonProps", props)),
        );
        assert_eq!(component.unwrap(), expect);
    }

    #[test]
    fn test_to_react_dot_fc() {
        let content = r#"
import React from "react";
import { AppFooter } from "./elements/Footer";

export type Props = {
  timeOut: number;
};

export const Footer:React.FC<Props> = (props) => {
  return <AppFooter></AppFooter>;
};
"#;
        let content = TSXContent(content.to_string());
        let component = content.to_component();
        let mut obj = ObjectType::new();
        obj.insert(
            Key("timeOut".to_string()),
            Type::Alias("number".to_string()),
        );
        let expect = Component::new(
            "Footer",
            Props::Named(NamedProps::new_object_type("Props", obj)),
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
            Type::Alias("number".to_string()),
        );
        props.insert(
            Key("errorMessage?".to_string()),
            Type::Alias("string".to_string()),
        );
        props.insert(Key("size".to_string()), Type::Alias("number".to_string()));
        let expect = Component::new(
            "ErrorAlert",
            Props::Named(NamedProps::new_object_type("Props", props)),
        );

        assert_eq!(component.unwrap(), expect);
    }
}
