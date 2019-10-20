use syn::*;
use syn::parse::*;
use syn::export::Span;

use super::template::*;

fn ident_to_dashed_str(s: Ident) -> LitStr {
    let s: String = s.to_string().chars().map(|c| {
        match c {
            '_' => '-',
            _ => c,
        }
    }).collect();
    LitStr::new(s.as_str(), Span::call_site())
}

fn ty_to_string(s: &Ident) -> String {
    let s: String = s.to_string().chars().map(|c| {
        if c >= 'A' && c <= 'Z' {
            format!("-{}", c.to_lowercase())
        } else {
            c.to_string()
        }
    }).collect();
    s
}

fn parse_children(input: ParseStream) -> Result<(Vec<TemplateNode>, Vec<(Ident, TemplateValue)>)> {
    let content;
    braced!(content in input);
    let input = &content;
    let mut props = vec![];
    let mut children = vec![];
    while !input.is_empty() {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![if]) {
            input.parse::<Token![if]>()?;
            children.push(parse_template_if(input)?);
        } else if lookahead.peek(Token![for]) {
            input.parse::<Token![for]>()?;
            children.push(parse_template_for(input)?);
        } else if lookahead.peek(Ident) {
            let name: Ident = input.parse()?;
            if name.to_string() == "slot" {
                children.push(parse_template_slot(input)?);
            } else {
                let lookahead = input.lookahead1();
                if lookahead.peek(Token![=]) {
                    input.parse::<Token![=]>()?;
                    let expr: Expr = input.parse()?;
                    input.parse::<Token![;]>()?;
                    props.push((name, TemplateValue::from(expr)));
                } else if lookahead.peek(token::Brace) {
                    children.push(parse_template_element(name, input)?);
                } else {
                    return Err(lookahead.error());
                }
            }
        } else if lookahead.peek(token::Paren) || lookahead.peek(LitStr) {
            children.push(parse_template_text_node(&input)?);
        } else {
            return Err(lookahead.error());
        }
    }
    Ok((children, props))
}

fn parse_template_slot(input: ParseStream) -> Result<TemplateNode> {
    let lookahead = input.lookahead1();
    let name = if lookahead.peek(Token![;]) {
        None
    } else {
        Some(input.parse()?)
    };
    input.parse::<Token![;]>()?;
    Ok(TemplateNode::VirtualNode(TemplateVirtualNode::Slot { name }))
}

fn parse_template_shadow_root(input: ParseStream) -> Result<TemplateShadowRoot> {
    let (children, _) = parse_children(input)?;
    Ok(TemplateShadowRoot { children })
}

fn parse_template_if(input: ParseStream) -> Result<TemplateNode> {
    let mut branches = vec![];
    let cond: Expr = input.parse()?;
    let (children, _) = parse_children(input)?;
    branches.push((Some(cond), children));
    loop {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![else]) {
            input.parse::<Token![else]>()?;
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![if]) {
                input.parse::<Token![if]>()?;
                let cond: Expr = input.parse()?;
                let (children, _) = parse_children(input)?;
                branches.push((Some(cond), children));
            } else {
                let (children, _) = parse_children(input)?;
                branches.push((None, children));
                break
            }
        } else {
            break
        }
    }
    Ok(TemplateNode::VirtualNode(TemplateVirtualNode::If { branches }))
}

fn parse_template_for(input: ParseStream) -> Result<TemplateNode> {
    let lookahead = input.lookahead1();
    let (index, item) = if lookahead.peek(Ident) {
        (Ident::new("index", Span::call_site()), input.parse::<Ident>()?)
    } else if lookahead.peek(token::Paren) {
        let index = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;
        let item = input.parse::<Ident>()?;
        (index, item)
    } else {
        return Err(lookahead.error());
    };
    input.parse::<Token![in]>()?;
    let list: Expr = input.parse()?;
    let lookahead = input.lookahead1();
    let key = if lookahead.peek(Token![use]) {
        input.parse::<Token![use]>()?;
        let field: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: Path = input.parse()?;
        Some((field, ty))
    } else {
        None
    };
    let (children, _) = parse_children(input)?;
    Ok(TemplateNode::VirtualNode(TemplateVirtualNode::For { index, item, list, key, children }))
}

fn parse_template_element(name: Ident, input: ParseStream) -> Result<TemplateNode> {
    let name_s = name.to_string();
    let is_component = name_s.chars().next().unwrap().is_uppercase();
    let lookahead = input.lookahead1();
    let slot: LitStr = if lookahead.peek(Token![in]) {
        input.parse()?
    } else if lookahead.peek(token::Brace) {
        LitStr::new("", Span::call_site())
    } else {
        return Err(lookahead.error());
    };
    let (children, props) = parse_children(input)?;
    if is_component {
        Ok(TemplateNode::Component(TemplateComponent {
            tag_name: LitStr::new(format!("maomi{}", ty_to_string(&name)).as_str(), Span::call_site()),
            component: name,
            property_values: props,
            slot,
            children,
        }))
    } else {
        Ok(TemplateNode::NativeNode(TemplateNativeNode {
            tag_name: ident_to_dashed_str(name),
            attributes: props.into_iter().map(|(name, value)| {
                (ident_to_dashed_str(name), value)
            }).collect(),
            slot,
            children,
        }))
    }
}

fn parse_template_text_node(input: ParseStream) -> Result<TemplateNode> {
    let expr: Expr = input.parse()?;
    input.parse::<Token![;]>()?;
    let expr: Expr = if let Expr::Paren(x) = expr { *x.expr } else { expr };
    Ok(TemplateNode::TextNode(TemplateValue::from(expr)))
}

pub(crate) fn parse_template(input: ParseStream) -> Result<TemplateShadowRoot> {
    parse_template_shadow_root(input)
}
