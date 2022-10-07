
pub struct StyleSheet<T: StyleSheetConstructor> {
    ssc: T,
    pub items: Vec<StyleSheetItem<T>>,
    vars: StyleSheetVars,
}

pub struct StyleSheetVars {
    macros: FxHashMap<String, MacroDefinition>,
    consts: FxHashMap<String, Vec<CssToken>>,
    keyframes: FxHashMap<String, CssIdent>,
}

impl<T: StyleSheetConstructor> Parse for StyleSheet<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut ssc = T::new();
        let mut items = vec![];
        let mut vars = StyleSheetVars {
            macros: FxHashMap::default(),
            consts: FxHashMap::default(),
            keyframes: FxHashMap::default(),
        };

        // parse items
        while !input.is_empty() {
            let vars = &mut vars;
            let la = input.lookahead1();
            if la.peek(token::At) {
                let at_keyword: CssAtKeyword = input.parse()?;
                match at_keyword.formal_name.as_str() {
                    "import" => {
                        // IDEA considering a proper cache to avoid parsing during every import
                        let item: StyleSheetImportItem = input.parse()?;
                        let content = get_import_content(&item.src)?;
                        let token_stream = proc_macro2::TokenStream::from_str(&content)?;
                        let mut ss = parse2::<StyleSheet<T>>(token_stream).map_err(|err| {
                            let original_span = err.span();
                            let start = original_span.start();
                            Error::new(
                                at_keyword.span(),
                                format_args!(
                                    "when parsing {}:{}:{}: {}",
                                    item.src.value(),
                                    start.line,
                                    start.column,
                                    err
                                ),
                            )
                        })?;
                        vars.macros.extend(ss.vars.macros);
                        vars.consts.extend(ss.vars.consts);
                        items.append(&mut ss.items);
                    }
                    "macro" => {
                        let item = StyleSheetMacroItem::parse_with_vars(
                            input,
                            vars,
                            &mut ScopeVars::new(),
                        )?;
                        let mut refs = vec![];
                        item.for_each_ref(&mut |x| refs.push(x.clone()));
                        if vars
                            .macros
                            .insert(item.name.formal_name.clone(), item.mac.block)
                            .is_some()
                        {
                            return Err(Error::new(
                                item.name.span,
                                format!(
                                    "macro named `{}` has already defined",
                                    item.name.formal_name
                                ),
                            ));
                        }
                        items.push(StyleSheetItem::MacroDefinition {
                            at_keyword,
                            name: item.name,
                            refs,
                        })
                    }
                    "const" => {
                        let item = StyleSheetConstItem::parse_with_vars(
                            &input,
                            vars,
                            &mut ScopeVars::new(),
                        )?;
                        let (tokens, refs) = item.content.get();
                        if vars
                            .consts
                            .insert(item.name.formal_name.clone(), tokens)
                            .is_some()
                        {
                            return Err(Error::new(
                                item.name.span,
                                format!(
                                    "const named `{}` has already defined",
                                    item.name.formal_name
                                ),
                            ));
                        }
                        items.push(StyleSheetItem::ConstDefinition {
                            at_keyword,
                            name: item.name,
                            refs,
                        })
                    }
                    "config" => {
                        let name: CssIdent = input.parse()?;
                        let _: CssColon = input.parse()?;
                        let (tokens, _refs) = ParseTokenUntilSemi::parse_with_vars(
                            input,
                            vars,
                            &mut ScopeVars::new(),
                        )?
                        .get();
                        let mut stream = CssTokenStream::new(input.span(), tokens);
                        ssc.set_config(&name, &mut stream)?;
                        stream.expect_ended()?;
                        let _: CssSemi = input.parse()?;
                    }
                    "keyframes" => {
                        let dollar_token = input.parse()?;
                        let name: CssIdent = input.parse()?;
                        let content;
                        let brace_token = braced!(content in input);
                        let input = content;
                        let mut content = vec![];
                        while !input.is_empty() {
                            let la = input.lookahead1();
                            let percentage = if la.peek(Ident) {
                                let s: CssIdent = input.parse()?;
                                match s.formal_name.as_str() {
                                    "from" => CssPercentage {
                                        span: s.span(),
                                        num: Number::Int(0),
                                    },
                                    "to" => CssPercentage {
                                        span: s.span(),
                                        num: Number::Int(100),
                                    },
                                    _ => return Err(Error::new(s.span(), "illegal ident")),
                                }
                            } else if la.peek(Lit) {
                                input.parse()?
                            } else {
                                return Err(la.error());
                            };
                            let props = ParseWithVars::parse_with_vars(
                                &input,
                                vars,
                                &mut ScopeVars::new(),
                            )?;
                            content.push((percentage, props));
                        }
                        let def = ssc.define_key_frames(&name, &content);
                        vars.keyframes.insert(name.formal_name.clone(), def.clone());
                        items.push(StyleSheetItem::KeyFramesDefinition {
                            at_keyword,
                            dollar_token,
                            name,
                            brace_token,
                            content,
                            def,
                        })
                    }
                    _ => {
                        return Err(Error::new(at_keyword.span(), "unknown at-keyword"));
                    }
                }
            } else if la.peek(token::Dot) {
                let dot_token = input.parse()?;
                let ident = input.parse()?;
                items.push(StyleSheetItem::Rule {
                    dot_token,
                    ident,
                    content: ParseWithVars::parse_with_vars(input, vars, &mut ScopeVars::new())?,
                })
            } else {
                return Err(la.error());
            };
        }

        Ok(Self { ssc, items, vars })
    }
}

impl<T: StyleSheetConstructor> ToTokens for StyleSheet<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.ssc.to_tokens(self, tokens)
    }
}
