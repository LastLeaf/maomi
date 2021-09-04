use cssparser::{
    ParseError, ParseErrorKind, Parser, ParserInput, ToCss, Token, TokenSerializationType,
};
use std::collections::HashMap;
use std::fmt::Write;
use std::path::Path;
use std::rc::Rc;

pub(crate) struct CustomError {
    message: String,
}
impl std::fmt::Debug for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.message)
    }
}

#[derive(Clone)]
struct VarValue {
    value: Rc<String>,
    tst_before: TokenSerializationType,
    tst_after: TokenSerializationType,
}
impl VarValue {
    fn write(&self, output: &mut String, tst: TokenSerializationType) -> TokenSerializationType {
        let next_tst = self.tst_before.clone();
        if tst.needs_separator_when_before(next_tst) {
            output.write_str(" ").unwrap();
        }
        output.write_str(&self.value).unwrap();
        self.tst_after.clone()
    }
}

struct ParseState<'a> {
    namespace: Option<&'a str>,
    prefix: &'a str,
    vars: &'a mut HashMap<String, VarValue>,
    output: &'a mut String,
}
impl<'a> ParseState<'a> {
    fn new(
        namespace: Option<&'a str>,
        vars: &'a mut HashMap<String, VarValue>,
        output: &'a mut String,
    ) -> Self {
        Self {
            namespace,
            prefix: "",
            vars: vars,
            output: output,
        }
    }
    fn sub<'b>(&'b mut self) -> ParseState<'b>
    where
        'a: 'b,
    {
        ParseState {
            namespace: self.namespace,
            prefix: self.prefix,
            vars: self.vars,
            output: self.output,
        }
    }
    fn sub_output<'b>(&'b mut self, output: &'b mut String) -> ParseState<'b>
    where
        'a: 'b,
    {
        ParseState {
            namespace: self.namespace,
            prefix: self.prefix,
            vars: self.vars,
            output: output,
        }
    }
    fn sub_scope<'b>(
        &'b mut self,
        prefix: &'b str,
        vars: &'b mut HashMap<String, VarValue>,
    ) -> ParseState<'b>
    where
        'a: 'b,
    {
        ParseState {
            namespace: self.namespace,
            prefix,
            vars,
            output: self.output,
        }
    }
    fn get_var(&self, name: &str) -> Option<&VarValue> {
        self.vars.get(name)
    }
    fn set_var(&mut self, name: &str, value: VarValue) {
        self.vars.insert(name.into(), value);
    }
}

fn write_token(
    token: &Token,
    output: &mut String,
    tst: TokenSerializationType,
) -> TokenSerializationType {
    let next_tst = token.serialization_type();
    if tst.needs_separator_when_before(next_tst) {
        output.write_str(" ").unwrap();
    }
    token.to_css(output).unwrap();
    token.serialization_type()
}

pub(crate) fn compile<'a>(
    namespace: Option<&'a str>,
    style: &'a str,
) -> Result<String, ParseError<'a, CustomError>> {
    let mut parser_input = ParserInput::new(&style);
    let mut parser = Parser::new(&mut parser_input);
    let mut output = String::new();
    let mut vars = HashMap::new();
    parse_segment(
        &mut parser,
        ParseState::new(namespace, &mut vars, &mut output),
    )?;
    Ok(output)
}

fn import_file<'a, 't: 'a, 'i: 't>(
    prev_parser: &'a mut Parser<'i, 't>,
    path: &'a Path,
    st: ParseState<'a>,
) -> Result<(), ParseError<'i, CustomError>> {
    let s = std::fs::read_to_string(path).map_err(|_| {
        prev_parser.new_custom_error(CustomError {
            message: format!("Failed to read file {:?}", path),
        })
    })?;
    let mut parser_input = ParserInput::new(&s);
    let mut parser = Parser::new(&mut parser_input);
    parse_segment(&mut parser, st).map_err(|err| {
        prev_parser.new_custom_error(CustomError {
            message: format!("Failed to load file {:?}: {:?}", path, err),
        })
    })?;
    Ok(())
}

fn parse_any_until_end<'a, 't: 'a, 'i: 't>(
    parser: &'a mut Parser<'i, 't>,
    mut st: ParseState<'a>,
    tst: TokenSerializationType,
) -> Result<TokenSerializationType, ParseError<'i, CustomError>> {
    let mut tst = tst;
    while !parser.is_exhausted() {
        let (_, next_tst) = parse_any(parser, st.sub(), tst)?;
        tst = next_tst;
    }
    Ok(tst)
}

fn parse_any_until_rule_end<'a, 't: 'a, 'i: 't>(
    parser: &'a mut Parser<'i, 't>,
    mut st: ParseState<'a>,
    tst: TokenSerializationType,
) -> Result<TokenSerializationType, ParseError<'i, CustomError>> {
    let mut tst = tst;
    while !parser.is_exhausted() {
        let (token, next_tst) = parse_any(parser, st.sub(), tst)?;
        tst = next_tst;
        match token {
            Token::CurlyBracketBlock => break,
            Token::Semicolon => break,
            _ => {}
        }
    }
    Ok(tst)
}

fn parse_any_end_with_semicolon<'a, 't: 'a, 'i: 't>(
    parser: &'a mut Parser<'i, 't>,
    mut st: ParseState<'a>,
    tst: TokenSerializationType,
) -> Result<TokenSerializationType, ParseError<'i, CustomError>> {
    let mut tst = tst;
    while !parser.is_exhausted() {
        if parser.try_parse(|parser| parser.expect_semicolon()).is_ok() {
            break;
        }
        let (_, next_tst) = parse_any(parser, st.sub(), tst)?;
        tst = next_tst;
    }
    Ok(tst)
}

fn parse_any<'a, 't: 'a, 'i: 't>(
    parser: &'a mut Parser<'i, 't>,
    mut st: ParseState<'a>,
    tst: TokenSerializationType,
) -> Result<(Token<'i>, TokenSerializationType), ParseError<'i, CustomError>> {
    let next = parser.next()?.clone();
    match next {
        Token::Function(_) => {
            write_token(&next, st.output, tst);
            parser.parse_nested_block(|parser| {
                parse_any_until_end(parser, st.sub(), TokenSerializationType::nothing())
            })?;
            st.output.write_str(")").unwrap();
            Ok((next, TokenSerializationType::nothing()))
        }
        Token::ParenthesisBlock => {
            st.output.write_str("(").unwrap();
            parser.parse_nested_block(|parser| {
                parse_any_until_end(parser, st.sub(), TokenSerializationType::nothing())
            })?;
            st.output.write_str(")").unwrap();
            Ok((next, TokenSerializationType::nothing()))
        }
        Token::SquareBracketBlock => {
            st.output.write_str("[").unwrap();
            parser.parse_nested_block(|parser| {
                parse_any_until_end(parser, st.sub(), TokenSerializationType::nothing())
            })?;
            st.output.write_str("]").unwrap();
            Ok((next, TokenSerializationType::nothing()))
        }
        Token::CurlyBracketBlock => {
            st.output.write_str("{").unwrap();
            parser.parse_nested_block(|parser| parse_segment(parser, st.sub()))?;
            st.output.write_str("}").unwrap();
            Ok((next, TokenSerializationType::nothing()))
        }
        Token::Semicolon => {
            let next_tst = write_token(&next, st.output, tst);
            Ok((next, next_tst))
        }
        Token::Ident(ref s) => {
            let next_tst = match st.get_var(s).cloned() {
                Some(x) => x.write(st.output, tst),
                None => write_token(&next, st.output, tst),
            };
            Ok((next, next_tst))
        }
        _ => {
            let next_tst = write_token(&next, st.output, tst);
            Ok((next, next_tst))
        }
    }
}

fn parse_segment<'a, 't: 'a, 'i: 't>(
    parser: &'a mut Parser<'i, 't>,
    mut st: ParseState<'a>,
) -> Result<(), ParseError<'i, CustomError>> {
    while !parser.is_exhausted() {
        parse_block(parser, st.sub())?;
    }
    Ok(())
}

fn parse_block<'a, 't: 'a, 'i: 't>(
    parser: &'a mut Parser<'i, 't>,
    mut st: ParseState<'a>,
) -> Result<(), ParseError<'i, CustomError>> {
    parser
        .try_parse(|parser| parse_at_keyword(parser, st.sub()))
        .or_else(|e| {
            if let ParseErrorKind::Custom(_) = e.kind {
                Err(e)
            } else {
                parser
                    .try_parse(|parser| parse_style_item_list(parser, st.sub()))
                    .or_else(|_| parse_common_block(parser, st.sub()))
            }
        })
}

fn parse_at_keyword<'a, 't: 'a, 'i: 't>(
    parser: &'a mut Parser<'i, 't>,
    mut st: ParseState<'a>,
) -> Result<(), ParseError<'i, CustomError>> {
    let next = parser.next()?.clone();
    match &next {
        Token::AtKeyword(k) => {
            let k: &str = k;
            match k {
                "import" => {
                    let import_path = parser.expect_string()?.clone();
                    parser.expect_semicolon()?;
                    if import_path.len() < 1 || &import_path[0..1] != "/" {
                        return Err(parser.new_custom_error(CustomError {
                            message: format!(r#"Imported path must start with "/" (relative to crate src), found {:?}"#, import_path)
                        }));
                    }
                    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
                    let source_path = std::path::Path::new(&root)
                        .join("src/")
                        .join(Path::new(&import_path[1..]));
                    import_file(parser, &source_path, st.sub())?;
                    Ok(())
                }
                "set" => {
                    let name = parser.expect_ident()?.clone();
                    let value: VarValue = {
                        let next = parser.next()?.clone();
                        match next {
                            Token::CurlyBracketBlock => {
                                let mut value = String::new();
                                parser.parse_nested_block(|parser| {
                                    parse_style_item_list_content(parser, st.sub_output(&mut value))
                                })?;
                                VarValue {
                                    value: Rc::new(value),
                                    tst_before: next.serialization_type(),
                                    tst_after: TokenSerializationType::nothing(),
                                }
                            }
                            Token::Colon => {
                                let mut value = String::new();
                                let tst = parse_any_end_with_semicolon(
                                    parser,
                                    st.sub_output(&mut value),
                                    TokenSerializationType::nothing(),
                                )?;
                                VarValue {
                                    value: Rc::new(value),
                                    tst_before: next.serialization_type(),
                                    tst_after: tst,
                                }
                            }
                            _ => {
                                return Err(parser.new_unexpected_token_error(next));
                            }
                        }
                    };
                    st.set_var(&name, value);
                    Ok(())
                }
                "calc" => {
                    // TODO impl @calc
                    unimplemented!()
                }
                _ => {
                    let next_tst = write_token(&next, st.output, TokenSerializationType::nothing());
                    parse_any_until_rule_end(parser, st.sub(), next_tst)?;
                    Ok(())
                }
            }
        }
        _ => Err(parser.new_unexpected_token_error(next)),
    }
}

fn parse_common_block<'a, 't: 'a, 'i: 't>(
    parser: &'a mut Parser<'i, 't>,
    mut st: ParseState<'a>,
) -> Result<(), ParseError<'i, CustomError>> {
    let mut cur_prefix: String = st.prefix.into();
    let mut need_namespace = false;
    let mut tst = TokenSerializationType::nothing();
    while !parser.is_exhausted() {
        let next = parser.next()?.clone();
        let mut next_need_namespace = false;
        match next {
            Token::Ident(ref s) => {
                if need_namespace {
                    cur_prefix += st.namespace.unwrap_or("");
                }
                tst = match st.get_var(s).cloned() {
                    Some(x) => x.write(&mut cur_prefix, tst),
                    None => write_token(&next, &mut cur_prefix, tst),
                };
            }
            Token::Hash(_) => {
                tst = write_token(&next, &mut cur_prefix, tst);
            }
            Token::IDHash(_) => {
                tst = write_token(&next, &mut cur_prefix, tst);
            }
            Token::Delim(c) => {
                if c == '.' {
                    next_need_namespace = true;
                }
                tst = write_token(&next, &mut cur_prefix, tst);
            }
            Token::Colon => {
                tst = write_token(&next, &mut cur_prefix, tst);
            }
            Token::Comma => {
                tst = write_token(&next, &mut cur_prefix, tst);
            }
            Token::CDO => {}
            Token::CDC => {}
            Token::Function(_) => {
                tst = write_token(&next, &mut cur_prefix, tst);
                parser.parse_nested_block(|parser| {
                    parse_segment(parser, st.sub_output(&mut cur_prefix))
                })?;
                cur_prefix.write_str(")").unwrap();
            }
            Token::ParenthesisBlock => {
                cur_prefix.write_str("(").unwrap();
                parser.parse_nested_block(|parser| {
                    parse_segment(parser, st.sub_output(&mut cur_prefix))
                })?;
                cur_prefix.write_str(")").unwrap();
            }
            Token::SquareBracketBlock => {
                cur_prefix.write_str("[").unwrap();
                parser.parse_nested_block(|parser| {
                    parse_any_until_end(
                        parser,
                        st.sub_output(&mut cur_prefix),
                        TokenSerializationType::nothing(),
                    )
                })?;
                cur_prefix.write_str("]").unwrap();
            }
            Token::CurlyBracketBlock => {
                parser.parse_nested_block(|parser| {
                    parse_segment(parser, st.sub_scope(&mut cur_prefix, &mut st.vars.clone()))
                })?;
                break;
            }
            _ => {
                return Err(parser.new_unexpected_token_error(next));
            }
        }
        need_namespace = next_need_namespace;
    }
    Ok(())
}

fn parse_style_item_list<'a, 't: 'a, 'i: 't>(
    parser: &'a mut Parser<'i, 't>,
    mut st: ParseState<'a>,
) -> Result<(), ParseError<'i, CustomError>> {
    let mut list_output = String::new();
    parse_style_item_list_content(parser, st.sub_output(&mut list_output))?;
    st.output
        .write_fmt(format_args!("{}{{{}", st.prefix, list_output))
        .unwrap();
    st.output.write_str("}").unwrap();
    Ok(())
}

fn parse_style_item_list_content<'a, 't: 'a, 'i: 't>(
    parser: &'a mut Parser<'i, 't>,
    mut st: ParseState<'a>,
) -> Result<(), ParseError<'i, CustomError>> {
    let mut item_output = String::new();
    parse_style_item(parser, st.sub_output(&mut item_output))?;
    st.output
        .write_fmt(format_args!("{}", item_output))
        .unwrap();
    loop {
        match parser.try_parse::<_, _, ParseError<'i, CustomError>>(|parser| {
            let mut item_output = String::new();
            parse_style_item(parser, st.sub_output(&mut item_output))?;
            st.output
                .write_fmt(format_args!(";{}", item_output))
                .unwrap();
            Ok(())
        }) {
            Ok(_) => {}
            Err(_) => break,
        }
    }
    Ok(())
}

fn parse_style_item<'a, 't: 'a, 'i: 't>(
    parser: &'a mut Parser<'i, 't>,
    mut st: ParseState<'a>,
) -> Result<(), ParseError<'i, CustomError>> {
    let ident = parser.expect_ident()?.clone();
    match st.get_var(&ident).cloned() {
        Some(x) => {
            x.write(st.output, TokenSerializationType::nothing());
            let next = parser.next()?.clone();
            match next {
                Token::Colon => {}
                Token::Semicolon => return Ok(()),
                _ => return Err(parser.new_unexpected_token_error(next)),
            }
        }
        None => {
            parser.expect_colon()?;
            st.output.write_fmt(format_args!("{}:", ident)).unwrap();
        }
    }
    parse_any_end_with_semicolon(parser, st.sub(), TokenSerializationType::nothing()).map(|_| ())
}
