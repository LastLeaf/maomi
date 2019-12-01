use syn::*;
use syn::parse::*;
use syn::export::Span;

use super::template::*;
use super::template::Attribute as TemplateAttribute;

const SYSTEM_ATTRIBUTES: [&'static str; 3] = [
    "id",
    "class",
    "style",
];
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

fn parse_children(input: ParseStream, is_component: bool, is_virtual: bool) -> Result<(Vec<TemplateNode>, Vec<TemplateAttribute>)> {
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
        } else if lookahead.peek(Token![in]) {
            input.parse::<Token![in]>()?;
            children.push(parse_template_in(input)?);
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
            let name: Ident = input.parse()?;
            if is_virtual {
                return Err(Error::new(name.span(), "cannot add event listeners here"));
            }
            let name_str = name.to_string();
            if !is_custom && SYSTEM_EVENTS.iter().position(|s| s == &name_str).is_none() {
                is_custom = true;
            }
            input.parse::<Token![=]>()?;
            let expr: Expr = input.parse()?;
            input.parse::<Token![;]>()?;
            if is_custom {
                if !is_component {
                    return Err(Error::new(name.span(), "custom events are not supported on this node"));
                }
                props.push(TemplateAttribute::Ev { name, value: TemplateValue::from(expr) });
            } else {
                props.push(TemplateAttribute::SystemEv { name, value: TemplateValue::from(expr) });
            }
        } else if lookahead.peek(Token![#]) {
            input.parse::<Token![#]>()?;
            let name: Ident = input.parse()?;
            if is_virtual {
                return Err(Error::new(name.span(), "cannot add attributes here"));
            }
            input.parse::<Token![=]>()?;
            let expr: Expr = input.parse()?;
            input.parse::<Token![;]>()?;
            if !is_component {
                return Err(Error::new(name.span(), "component properties are not supported on this node"));
            }
            props.push(TemplateAttribute::Prop { name, value: TemplateValue::from(expr) });
        } else if lookahead.peek(Ident) {
            if input.peek2(Token![=]) || input.peek2(Token![;]) {
                let name: Ident = input.parse()?;
                if name.to_string() == "slot" {
                    children.push(parse_template_slot(input)?);
                } else {
                    if is_virtual {
                        return Err(Error::new(name.span(), "cannot add attributes here"));
                    }
                    let name_str = name.to_string();
                    input.parse::<Token![=]>()?;
                    let expr: Expr = input.parse()?;
                    input.parse::<Token![;]>()?;
                    if name_str == "mark" {
                        props.push(TemplateAttribute::Mark { value: TemplateValue::from(expr) });
                    } else if is_component && SYSTEM_ATTRIBUTES.iter().position(|s| s == &name_str).is_none() {
                        props.push(TemplateAttribute::Prop { name, value: TemplateValue::from(expr) });
                    } else {
                        props.push(TemplateAttribute::Common { name: ident_to_dashed_str(&name), value: TemplateValue::from(expr) });
                    }
                }
            } else {
                children.push(parse_template_element(input)?);
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
        input.parse::<Token![=]>()?;
        Some(input.parse()?)
    };
    input.parse::<Token![;]>()?;
    Ok(TemplateNode::VirtualNode(TemplateVirtualNode::Slot { name }))
}

fn parse_template_shadow_root(input: ParseStream) -> Result<TemplateShadowRoot> {
    let (children, _) = parse_children(input, false, true)?;
    Ok(TemplateShadowRoot { children })
}

fn parse_template_if(input: ParseStream) -> Result<TemplateNode> {
    let mut branches = vec![];
    let cond: Expr = input.parse()?;
    let (children, _) = parse_children(input, false, true)?;
    branches.push((Some(cond), children));
    loop {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![else]) {
            input.parse::<Token![else]>()?;
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![if]) {
                input.parse::<Token![if]>()?;
                let cond: Expr = input.parse()?;
                let (children, _) = parse_children(input, false, true)?;
                branches.push((Some(cond), children));
            } else {
                let (children, _) = parse_children(input, false, true)?;
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
    let (children, _) = parse_children(input, false, true)?;
    Ok(TemplateNode::VirtualNode(TemplateVirtualNode::For { index, item, list, key, children }))
}

fn parse_template_in(input: ParseStream) -> Result<TemplateNode> {
    let name = input.parse()?;
    let (children, _) = parse_children(input, false, true)?;
    Ok(TemplateNode::VirtualNode(TemplateVirtualNode::InSlot { name, children }))
}

fn parse_template_element(input: ParseStream) -> Result<TemplateNode> {
    let component: Path = input.parse()?;
    let name = &component.segments[0].ident;
    let name_s = name.to_string();
    let is_component = component.leading_colon.is_some() || name_s.chars().next().unwrap().is_uppercase();
    let (children, props) = parse_children(input, is_component, false)?;
    if is_component {
        Ok(TemplateNode::Component(TemplateComponent {
            tag_name: LitStr::new(format!("maomi{}", ty_to_string(&name)).as_str(), Span::call_site()),
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

fn parse_template_text_node(input: ParseStream) -> Result<TemplateNode> {
    let expr: Expr = input.parse()?;
    input.parse::<Token![;]>()?;
    let expr: Expr = if let Expr::Paren(x) = expr { *x.expr } else { expr };
    Ok(TemplateNode::TextNode(TemplateValue::from(expr)))
}

pub(crate) fn parse_template(input: ParseStream) -> Result<TemplateShadowRoot> {
    parse_template_shadow_root(input)
}
