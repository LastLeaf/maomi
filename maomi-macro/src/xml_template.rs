use syn::export::Span;
use syn::parse::*;
use syn::*;

use super::template::Attribute as TemplateAttribute;
use super::template::*;

const SYSTEM_ATTRIBUTES: [&'static str; 2] = ["id", "style"];
const SYSTEM_EVENTS: [&'static str; 20] = [
    "click",
    "mouse_down",
    "mouse_move",
    "mouse_up",
    "touch_start",
    "touch_move",
    "touch_end",
    "touch_cancel",
    "tap",
    "long_tap",
    "cancel_tap",
    "key_down",
    "key_press",
    "key_up",
    "change",
    "submit",
    "animation_start",
    "animation_iteration",
    "animation_end",
    "transition_end",
];

fn ident_to_dashed_str(s: &Ident) -> LitStr {
    let s: String = s
        .to_string()
        .chars()
        .map(|c| match c {
            '_' => '-',
            _ => c,
        })
        .collect();
    LitStr::new(s.as_str(), Span::call_site())
}

fn ty_to_string(s: &Ident) -> String {
    let s: String = s
        .to_string()
        .chars()
        .map(|c| {
            if c >= 'A' && c <= 'Z' {
                format!("-{}", c.to_lowercase())
            } else {
                c.to_string()
            }
        })
        .collect();
    s
}

fn parse_children(input: ParseStream) -> Result<Vec<TemplateNode>> {
    let mut children = vec![];
    while !input.is_empty() {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![<]) {
            input.parse::<Token![<]>()?;
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![/]) {
                input.parse::<Token![/]>()?;
                break;
            } else if lookahead.peek(Token![if]) {
                input.parse::<Token![if]>()?;
                children.push(parse_template_if(input)?);
            } else if lookahead.peek(Token![for]) {
                input.parse::<Token![for]>()?;
                children.push(parse_template_for(input)?);
            } else if lookahead.peek(Token![in]) {
                input.parse::<Token![in]>()?;
                children.push(parse_template_in(input)?);
            } else {
                children.push(parse_template_element_or_slot(input)?);
            }
        } else if lookahead.peek(token::Brace) || lookahead.peek(LitStr) {
            children.push(TemplateNode::TextNode(parse_template_value(&input)?));
        } else {
            return Err(lookahead.error());
        }
    }
    Ok(children)
}

fn parse_template_value(input: ParseStream) -> Result<TemplateValue> {
    let lookahead = input.lookahead1();
    if lookahead.peek(token::Brace) {
        let content;
        braced!(content in input);
        Ok(TemplateValue::from(content.parse::<Expr>()?))
    } else if lookahead.peek(LitStr) {
        let s = input.parse::<LitStr>()?;
        let expr: Expr = Expr::Lit(ExprLit {
            attrs: vec![],
            lit: Lit::Str(s),
        });
        Ok(TemplateValue::from(expr))
    } else {
        return Err(lookahead.error());
    }
}

fn parse_template_slot(input: ParseStream) -> Result<TemplateNode> {
    let lookahead = input.lookahead1();
    let name = if lookahead.peek(Token![/]) {
        None
    } else {
        Some(input.parse()?)
    };
    input.parse::<Token![/]>()?;
    input.parse::<Token![>]>()?;
    Ok(TemplateNode::VirtualNode(TemplateVirtualNode::Slot {
        name,
    }))
}

fn parse_template_shadow_root(input: ParseStream) -> Result<TemplateShadowRoot> {
    let children = parse_children(input)?;
    Ok(TemplateShadowRoot { children })
}

fn parse_template_if(input: ParseStream) -> Result<TemplateNode> {
    let mut branches = vec![];
    let mut is_end = false;
    loop {
        let cond = if is_end {
            None
        } else {
            let content;
            braced!(content in input);
            let cond = content.parse()?;
            Some(cond)
        };
        let lookahead = input.lookahead1();
        let children = if lookahead.peek(Token![/]) {
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;
            vec![]
        } else if lookahead.peek(Token![>]) {
            input.parse::<Token![>]>()?;
            let children = parse_children(input)?;
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![if]) {
                input.parse::<Token![if]>()?;
            } else if lookahead.peek(Token![else]) {
                input.parse::<Token![else]>()?;
            } else {
                return Err(lookahead.error());
            }
            input.parse::<Token![>]>()?;
            children
        } else {
            return Err(lookahead.error());
        };
        branches.push((cond, children));
        if is_end {
            break;
        }
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![<]) {
            if input.peek2(Token![else]) {
                input.parse::<Token![<]>()?;
                input.parse::<Token![else]>()?;
                let lookahead = input.lookahead1();
                if lookahead.peek(Token![if]) {
                    input.parse::<Token![if]>()?;
                } else {
                    is_end = true;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }
    if !is_end {
        branches.push((None, vec![]));
    }
    Ok(TemplateNode::VirtualNode(TemplateVirtualNode::If {
        branches,
    }))
}

fn parse_template_for(input: ParseStream) -> Result<TemplateNode> {
    let lookahead = input.lookahead1();
    let (index, item) = if lookahead.peek(Ident) {
        (
            Ident::new("index", Span::call_site()),
            input.parse::<Ident>()?,
        )
    } else if lookahead.peek(token::Paren) {
        let content;
        parenthesized!(content in input);
        let input = content;
        let index = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;
        let item = input.parse::<Ident>()?;
        (index, item)
    } else {
        return Err(lookahead.error());
    };
    input.parse::<Token![in]>()?;
    let content;
    braced!(content in input);
    let list: Expr = content.parse()?;
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
    let lookahead = input.lookahead1();
    let children = if lookahead.peek(Token![/]) {
        input.parse::<Token![/]>()?;
        input.parse::<Token![>]>()?;
        vec![]
    } else if lookahead.peek(Token![>]) {
        input.parse::<Token![>]>()?;
        let children = parse_children(input)?;
        input.parse::<Token![for]>()?;
        input.parse::<Token![>]>()?;
        children
    } else {
        return Err(lookahead.error());
    };
    Ok(TemplateNode::VirtualNode(TemplateVirtualNode::For {
        index,
        item,
        list,
        key,
        children,
    }))
}

fn parse_template_in(input: ParseStream) -> Result<TemplateNode> {
    let name = input.parse()?;
    let children = parse_children(input)?;
    input.parse::<Token![in]>()?;
    input.parse::<Token![>]>()?;
    Ok(TemplateNode::VirtualNode(TemplateVirtualNode::InSlot {
        name,
        children,
    }))
}

fn ident_to_string(ident: &Ident) -> String {
    let ret = ident.to_string();
    if ret.starts_with("r#") {
        ret[2..].into()
    } else {
        ret
    }
}

fn parse_dashed_name(input: ParseStream) -> Result<Ident> {
    let name: Ident = input.parse()?;
    let span = name.span();
    let mut s = ident_to_string(&name);
    loop {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![-]) {
            input.parse::<Token![-]>()?;
            let name: Ident = input.parse()?;
            s += "_";
            s += &ident_to_string(&name);
        } else {
            break;
        }
    }
    Ok(Ident::new(&s, span))
}

fn parse_attributes(
    input: ParseStream,
    is_component: bool,
) -> Result<(Vec<TemplateAttribute>, bool)> {
    let mut props = vec![];
    let mut self_closed = false;
    loop {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![/]) {
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;
            self_closed = true;
            break;
        } else if lookahead.peek(Token![>]) {
            input.parse::<Token![>]>()?;
            break;
        } else if lookahead.peek(Token![@]) {
            input.parse::<Token![@]>()?;
            let lookahead = input.lookahead1();
            let mut is_custom = false;
            if lookahead.peek(Token![#]) {
                input.parse::<Token![#]>()?;
                is_custom = true;
            } else if lookahead.peek(Ident) {
                // empty
            } else {
                return Err(lookahead.error());
            }
            let name: Ident = parse_dashed_name(input)?;
            let name_str = name.to_string();
            if !is_custom && SYSTEM_EVENTS.iter().position(|s| s == &name_str).is_none() {
                is_custom = true;
            }
            input.parse::<Token![=]>()?;
            let content;
            braced!(content in input);
            let value: Expr = content.parse()?;
            if is_custom {
                if !is_component {
                    return Err(Error::new(
                        name.span(),
                        "custom events are not supported on this node",
                    ));
                }
                props.push(TemplateAttribute::Ev {
                    name,
                    value: TemplateValue::from(value),
                });
            } else {
                props.push(TemplateAttribute::SystemEv {
                    name,
                    value: TemplateValue::from(value),
                });
            }
        } else if lookahead.peek(Token![#]) {
            input.parse::<Token![#]>()?;
            let name: Ident = parse_dashed_name(input)?;
            input.parse::<Token![=]>()?;
            let value = parse_template_value(input)?;
            if !is_component {
                return Err(Error::new(
                    name.span(),
                    "component properties are not supported on this node",
                ));
            }
            props.push(TemplateAttribute::Prop { name, value });
        } else if lookahead.peek(Ident) {
            let name: Ident = parse_dashed_name(input)?;
            let name_str = name.to_string();
            input.parse::<Token![=]>()?;
            let value = parse_template_value(input)?;
            if name_str == "mark" {
                props.push(TemplateAttribute::Mark { value });
            } else if name_str == "class" {
                props.push(TemplateAttribute::ClassProp { value });
            } else if is_component
                && SYSTEM_ATTRIBUTES
                    .iter()
                    .position(|s| s == &name_str)
                    .is_none()
            {
                props.push(TemplateAttribute::Prop { name, value });
            } else {
                props.push(TemplateAttribute::Common {
                    name: ident_to_dashed_str(&name),
                    value,
                });
            }
        } else {
            return Err(lookahead.error());
        }
    }
    Ok((props, self_closed))
}

fn parse_template_element_or_slot(input: ParseStream) -> Result<TemplateNode> {
    let component: Path = input.parse()?;
    let name = &component.segments[0].ident;
    let name_s = name.to_string();
    if name_s == "slot" {
        return parse_template_slot(input);
    }
    let is_component =
        component.leading_colon.is_some() || name_s.chars().next().unwrap().is_uppercase();
    let (props, self_closed) = parse_attributes(input, is_component)?;
    let children = if !self_closed {
        let children = parse_children(input)?;
        let component_end: Ident = input.parse()?;
        if component.segments[0].ident != component_end {
            return Err(Error::new(
                component_end.span(),
                "end tag does not match the start tag",
            ));
        }
        input.parse::<Token![>]>()?;
        children
    } else {
        vec![]
    };
    if is_component {
        Ok(TemplateNode::Component(TemplateComponent {
            tag_name: LitStr::new(
                format!("maomi{}", ty_to_string(&name)).as_str(),
                Span::call_site(),
            ),
            component,
            property_values: props,
            children,
        }))
    } else {
        Ok(TemplateNode::NativeNode(TemplateNativeNode {
            tag_name: ident_to_dashed_str(name),
            attributes: props,
            children,
        }))
    }
}

pub(crate) fn parse_template(input: ParseStream) -> Result<TemplateShadowRoot> {
    let content;
    braced!(content in input);
    parse_template_shadow_root(&content)
}
