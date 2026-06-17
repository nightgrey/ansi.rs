use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{
    Attribute, Error, Expr, Ident, LitInt, Path, Token, braced, bracketed, parse_macro_input,
};

pub fn transition_fn(input: TokenStream) -> TokenStream {
    let machine = parse_macro_input!(input as Machine);
    match expand(machine) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

pub struct Machine {
    action_attrs: Option<Vec<Attribute>>,
    state_attrs: Option<Vec<Attribute>>,
    blocks: Vec<StateBlock>,
}
pub struct StateBlock { attrs: Vec<Attribute>, key: StateKey, entries: Vec<Entry> }
#[derive(Clone)] pub enum StateKey { Anywhere, State(Path) }
pub enum Entry { OnEntry(Expr), OnExit(Expr), Transition { byte: BytePattern, effect: Effect } }

#[derive(Clone)]
pub enum BytePattern {
    Single(LitInt),
    /// `start..end` (exclusive) or `start..=end` (inclusive)
    Range { start: LitInt, end: LitInt, inclusive: bool },
}
pub enum Effect { ActionOnly(Expr), StateOnly(Expr), ActionAndState { action: Expr, state: Expr } }

impl Parse for Machine {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut action_attrs = None;
        let mut state_attrs = None;
        let mut blocks = Vec::new();
        while !input.is_empty() {
            let attrs = input.call(Attribute::parse_outer)?;
            if input.peek(Token![enum]) {
                input.parse::<Token![enum]>()?;
                let name: Ident = input.parse()?;
                input.parse::<Token![;]>()?;
                let destination = match name.to_string().as_str() {
                    "Action" => &mut action_attrs,
                    "State" => &mut state_attrs,
                    _ => return Err(Error::new_spanned(name, "expected enum Action; or enum State;")),
                };
                if destination.replace(attrs).is_some() {
                    return Err(Error::new_spanned(name, "duplicate enum declaration"));
                }
                continue;
            }
            blocks.push(StateBlock::parse_with_attrs(input, attrs)?);
            if input.peek(Token![,]) { input.parse::<Token![,]>()?; }
        }
        Ok(Self { action_attrs, state_attrs, blocks })
    }
}
impl Parse for StateBlock {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        Self::parse_with_attrs(input, attrs)
    }
}
impl StateBlock {
    fn parse_with_attrs(input: ParseStream<'_>, attrs: Vec<Attribute>) -> syn::Result<Self> {
        let key = input.parse()?;
        input.parse::<Token![=>]>()?;
        let body; braced!(body in input);
        let mut entries = Vec::new();
        while !body.is_empty() {
            entries.push(body.parse()?);
            if body.peek(Token![,]) { body.parse::<Token![,]>()?; }
        }
        Ok(Self { attrs, key, entries })
    }
}
impl Parse for StateKey {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(Token![_]) { input.parse::<Token![_]>()?; Ok(Self::Anywhere) }
        else { Ok(Self::State(input.parse()?)) }
    }
}
impl Parse for Entry {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(Ident) {
            let fork = input.fork();
            let ident: Ident = fork.parse()?;
            match ident.to_string().as_str() {
                "on_entry" => { input.parse::<Ident>()?; input.parse::<Token![=>]>()?; return Ok(Self::OnEntry(input.parse()?)); }
                "on_exit"  => { input.parse::<Ident>()?; input.parse::<Token![=>]>()?; return Ok(Self::OnExit(input.parse()?)); }
                _ => {}
            }
        }
        let byte = input.parse()?;
        input.parse::<Token![=>]>()?;
        let effect = input.parse()?;
        Ok(Self::Transition { byte, effect })
    }
}
impl Parse for BytePattern {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let start: LitInt = input.parse()?;
        // NB: `..=` must be checked before `..`, since `peek(Token![..])` is also
        // true at the start of a `..=` token.
        if input.peek(Token![..=]) {
            input.parse::<Token![..=]>()?;
            let end = input.parse()?;
            Ok(Self::Range { start, end, inclusive: true })
        } else if input.peek(Token![..]) {
            input.parse::<Token![..]>()?;
            let end = input.parse()?;
            Ok(Self::Range { start, end, inclusive: false })
        } else {
            Ok(Self::Single(start))
        }
    }
}
impl Parse for Effect {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(syn::token::Bracket) {
            let inner; bracketed!(inner in input);
            let action: Expr = inner.parse()?;
            inner.parse::<Token![,]>()?;
            let state: Expr = inner.parse()?;
            if !inner.is_empty() { return Err(inner.error("expected exactly two items: [Action::..., State::...]")); }
            return Ok(Self::ActionAndState { action, state });
        }
        let expr: Expr = input.parse()?;
        match expr_root_ident(&expr).as_deref() {
            Some("Action") => Ok(Self::ActionOnly(expr)),
            Some("State") => Ok(Self::StateOnly(expr)),
            _ => Err(Error::new_spanned(expr, "expected Action::..., State::..., or [Action::..., State::...]")),
        }
    }
}

#[derive(Default)] pub struct OrderedNames { names: Vec<Ident> }
impl OrderedNames {
    fn insert(&mut self, ident: Ident) {
        if self.names.iter().any(|x| x == &ident) { return; }
        self.names.push(ident);
    }
}

pub fn expand(machine: Machine) -> syn::Result<TokenStream2> {
    let mut anywhere = Vec::<(&BytePattern, &Effect)>::new();
    let mut state_blocks = Vec::<&StateBlock>::new();
    for block in &machine.blocks {
        match &block.key {
            StateKey::Anywhere => {
                for entry in &block.entries {
                    match entry {
                        Entry::Transition { byte, effect } => anywhere.push((byte, effect)),
                        Entry::OnEntry(action) | Entry::OnExit(action) =>
                            return Err(Error::new_spanned(action, "on_entry / on_exit are not valid in the _ anywhere block")),
                    }
                }
            }
            StateKey::State(_) => state_blocks.push(block),
        }
    }
    let generated_state_enum = generate_state_enum(&machine)?;
    let generated_action_enum = generate_action_enum(&machine)?;
    let state_arms = transition_state_arms(&state_blocks, &anywhere)?;
    let entry_arms = state_lifecycle_arms(&state_blocks, LifecycleKind::Entry)?;
    let exit_arms = state_lifecycle_arms(&state_blocks, LifecycleKind::Exit)?;
    Ok(quote! {
        #generated_state_enum
        #generated_action_enum
        impl State {
            #[inline]
            pub const fn transition(self, byte: u8) -> (Action, State) {
                match self {
                    #(#state_arms)*
                    // States with no block of their own (e.g. transition-only
                    // targets) stay put and emit no action.
                    _ => (Action::None, self),
                }
            }
            /// Returns the entry action for the given state.
            #[inline]
            pub const fn entry(self) -> Action { match self { #(#entry_arms,)* _ => Action::None, } }
            /// Returns the exit action for the given state.
            #[inline]
            pub const fn exit(self) -> Action { match self { #(#exit_arms,)* _ => Action::None, } }
        }
    })
}

fn transition_rhs(effect: &Effect) -> TokenStream2 {
    match effect {
        Effect::ActionOnly(action) => quote! { (#action, State::None) },
        Effect::StateOnly(next_state) => quote! { (Action::None, #next_state) },
        Effect::ActionAndState { action, state } => quote! { (#action, #state) },
    }
}

#[derive(Clone, Copy)] pub enum LifecycleKind { Entry, Exit }
fn state_lifecycle_arms(state_blocks: &[&StateBlock], kind: LifecycleKind) -> syn::Result<Vec<TokenStream2>> {
    let mut arms = Vec::new();
    for block in state_blocks {
        let StateKey::State(state_path) = &block.key else { unreachable!() };
        let mut found: Option<&Expr> = None;
        for entry in &block.entries {
            let candidate = match (kind, entry) {
                (LifecycleKind::Entry, Entry::OnEntry(action)) => Some(action),
                (LifecycleKind::Exit, Entry::OnExit(action)) => Some(action),
                _ => None,
            };
            if let Some(action) = candidate {
                if found.is_some() { return Err(Error::new_spanned(action, "duplicate lifecycle action for this state")); }
                found = Some(action);
            }
        }
        if let Some(action) = found { arms.push(quote! { #state_path => #action }); }
    }
    Ok(arms)
}

fn expr_root_ident(expr: &Expr) -> Option<String> {
    let Expr::Path(path) = expr else { return None };
    path.path.segments.first().map(|s| s.ident.to_string())
}

fn transition_state_arms(
    state_blocks: &[&StateBlock],
    anywhere: &[(&BytePattern, &Effect)],
) -> syn::Result<Vec<TokenStream2>> {
    let mut arms = Vec::new();
    for block in state_blocks {
        let StateKey::State(state_path) = &block.key else { unreachable!() };
        let local: Vec<_> = block.entries.iter().filter_map(|e| match e {
            Entry::Transition { byte, effect } => Some((byte, effect)),
            _ => None,
        }).collect();
        let merged_arms = build_merged_byte_arms(anywhere, &local)?;
        arms.push(quote! {
            #state_path => match byte {
                #(#merged_arms)*
                _ => (Action::None, #state_path),
            }
        });
    }
    Ok(arms)
}

/// Fill `map` with first-match-wins semantics (mirrors a real `match`: the
/// earliest arm in source order claims each byte and is never overwritten).
fn fill_first_wins<'a>(
    map: &mut [Option<&'a Effect>; 256],
    list: &[(&BytePattern, &'a Effect)],
) -> syn::Result<()> {
    for (byte, effect) in list {
        let (start, end) = byte_pattern_range(byte)?;
        for b in start..=end {
            let slot = &mut map[b as usize];
            if slot.is_none() { *slot = Some(*effect); }
        }
    }
    Ok(())
}

fn build_merged_byte_arms(
    anywhere: &[(&BytePattern, &Effect)],
    local: &[(&BytePattern, &Effect)],
) -> syn::Result<Vec<TokenStream2>> {
    // Local (state-specific) arms take precedence over anywhere arms; within each
    // tier, the first arm in source order wins.
    let mut local_map: [Option<&Effect>; 256] = [None; 256];
    fill_first_wins(&mut local_map, local)?;
    let mut anywhere_map: [Option<&Effect>; 256] = [None; 256];
    fill_first_wins(&mut anywhere_map, anywhere)?;

    let mut byte_map: [Option<&Effect>; 256] = [None; 256];
    for b in 0..256usize { byte_map[b] = local_map[b].or(anywhere_map[b]); }

    // Coalesce contiguous bytes that share an effect into a single range arm.
    let mut merged: Vec<(u8, u8, &Effect)> = Vec::new();
    let mut run: Option<(u8, &Effect)> = None;
    for b in 0..=255u8 {
        match (run, byte_map[b as usize]) {
            (None, Some(eff)) => run = Some((b, eff)),
            (Some((start, cur)), Some(eff)) if !effects_equal(cur, eff) => {
                merged.push((start, b - 1, cur));
                run = Some((b, eff));
            }
            (Some((start, cur)), None) => { merged.push((start, b - 1, cur)); run = None; }
            _ => {}
        }
    }
    if let Some((start, eff)) = run { merged.push((start, 255, eff)); }

    Ok(merged.into_iter().map(|(start, end, effect)| {
        let rhs = transition_rhs(effect);
        let pat = byte_range_tokens(start, end);
        quote! { #pat => #rhs, }
    }).collect())
}

/// Resolve a `BytePattern` to an inclusive `(lo, hi)` byte range, with spanned
/// errors for out-of-range or empty ranges (no silent clamping).
fn byte_pattern_range(byte: &BytePattern) -> syn::Result<(u8, u8)> {
    match byte {
        BytePattern::Single(lit) => { let v = lit.base10_parse::<u8>()?; Ok((v, v)) }
        BytePattern::Range { start, end, inclusive } => {
            let lo = start.base10_parse::<u16>()?;
            let end_val = end.base10_parse::<u16>()?;
            let hi = if *inclusive {
                end_val
            } else {
                if end_val == 0 { return Err(Error::new_spanned(end, "exclusive range end must be greater than 0")); }
                end_val - 1
            };
            if lo > 0xff { return Err(Error::new_spanned(start, "byte literal out of range (expected 0..=255)")); }
            if hi > 0xff { return Err(Error::new_spanned(end, "byte literal out of range (expected 0..=255)")); }
            if lo > hi { return Err(Error::new_spanned(end, "empty or descending byte range")); }
            Ok((lo as u8, hi as u8))
        }
    }
}

fn effects_equal(a: &Effect, b: &Effect) -> bool { effect_to_string(a) == effect_to_string(b) }
fn effect_to_string(effect: &Effect) -> String {
    match effect {
        Effect::ActionOnly(expr) => quote!(#expr).to_string(),
        Effect::StateOnly(expr) => quote!(#expr).to_string(),
        Effect::ActionAndState { action, state } => quote!(#action, #state).to_string(),
    }
}
fn byte_range_tokens(start: u8, end: u8) -> TokenStream2 {
    if start == end {
        let lit = LitInt::new(&format!("0x{:02x}", start), proc_macro2::Span::call_site());
        quote! { #lit }
    } else {
        let start_lit = LitInt::new(&format!("0x{:02x}", start), proc_macro2::Span::call_site());
        let end_lit = LitInt::new(&format!("0x{:02x}", end), proc_macro2::Span::call_site());
        quote! { #start_lit..=#end_lit }
    }
}
fn generate_state_enum(machine: &Machine) -> syn::Result<TokenStream2> {
    let mut states = OrderedNames::default();
    states.insert(syn::parse_quote! { None });
    for block in &machine.blocks {
        if let StateKey::State(path) = &block.key { states.insert(last_path_ident(path)?); }
        for entry in &block.entries {
            if let Entry::Transition { effect, .. } = entry { collect_state_from_effect(effect, &mut states)?; }
        }
    }
    let default = default_state_ident(machine)?.unwrap_or_else(|| syn::parse_quote! { None });
    let variants = states.names.into_iter().map(|name| {
        if name == default { quote! { #[default] #name, } } else { quote! { #name, } }
    });
    let attrs = machine.state_attrs.as_ref().map_or_else(
        || quote! { #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)] }, |attrs| quote! { #(#attrs)* });
    Ok(quote! { #attrs pub enum State { #(#variants)* } })
}
fn generate_action_enum(machine: &Machine) -> syn::Result<TokenStream2> {
    let mut actions = OrderedNames::default();
    actions.insert(syn::parse_quote! { None });
    for block in &machine.blocks {
        for entry in &block.entries {
            match entry {
                Entry::OnEntry(action) | Entry::OnExit(action) => actions.insert(action_ident(action)?),
                Entry::Transition { effect, .. } => collect_action_from_effect(effect, &mut actions)?,
            }
        }
    }
    let variants = actions.names.into_iter().map(|name| {
        if name == "None" { quote! { #[default] None, } } else { quote! { #name, } }
    });
    let attrs = machine.action_attrs.as_ref().map_or_else(
        || quote! { #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)] }, |attrs| quote! { #(#attrs)* });
    Ok(quote! { #attrs pub enum Action { #(#variants)* } })
}
fn collect_state_from_effect(effect: &Effect, states: &mut OrderedNames) -> syn::Result<()> {
    match effect {
        Effect::ActionOnly(_) => {}
        Effect::StateOnly(state) => states.insert(state_ident(state)?),
        Effect::ActionAndState { state, .. } => states.insert(state_ident(state)?),
    }
    Ok(())
}
fn collect_action_from_effect(effect: &Effect, actions: &mut OrderedNames) -> syn::Result<()> {
    match effect {
        Effect::ActionOnly(action) => actions.insert(action_ident(action)?),
        Effect::StateOnly(_) => {}
        Effect::ActionAndState { action, .. } => actions.insert(action_ident(action)?),
    }
    Ok(())
}
fn state_ident(expr: &Expr) -> syn::Result<Ident> { enum_variant_ident(expr, "State") }
fn action_ident(expr: &Expr) -> syn::Result<Ident> { enum_variant_ident(expr, "Action") }
fn enum_variant_ident(expr: &Expr, enum_name: &str) -> syn::Result<Ident> {
    let Expr::Path(path) = expr else { return Err(Error::new_spanned(expr, format!("expected {enum_name}::Variant"))); };
    let mut segments = path.path.segments.iter();
    let Some(root) = segments.next() else { return Err(Error::new_spanned(expr, format!("expected {enum_name}::Variant"))); };
    if root.ident != enum_name { return Err(Error::new_spanned(expr, format!("expected {enum_name}::Variant"))); }
    let Some(variant) = segments.next() else { return Err(Error::new_spanned(expr, format!("expected {enum_name}::Variant"))); };
    if segments.next().is_some() { return Err(Error::new_spanned(expr, format!("expected {enum_name}::Variant, not a nested path"))); }
    Ok(variant.ident.clone())
}
fn last_path_ident(path: &Path) -> syn::Result<Ident> {
    path.segments.last().map(|s| s.ident.clone()).ok_or_else(|| Error::new_spanned(path, "expected state path"))
}
fn is_default_attr(attr: &Attribute) -> bool { attr.path().is_ident("default") }
fn default_state_ident(machine: &Machine) -> syn::Result<Option<Ident>> {
    let mut found: Option<Ident> = None;
    for block in &machine.blocks {
        if !block.attrs.iter().any(is_default_attr) { continue; }
        let StateKey::State(path) = &block.key else {
            return Err(Error::new_spanned(block.attrs.first().unwrap(), "#[default] is only valid on a concrete State arm, not on _"));
        };
        let ident = last_path_ident(path)?;
        if let Some(prev) = &found { return Err(Error::new_spanned(path, format!("duplicate #[default] state; already marked State::{prev} as default"))); }
        found = Some(ident);
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    fn render(m: Machine) -> String { expand(m).unwrap().to_string() }

    #[test]
    fn enum_declaration_attributes_are_applied() {
        let machine: Machine = syn::parse2(quote! {
            #[derive(Clone, Copy, Debug, Default, Maybe)]
            #[derive_const(PartialEq, Eq)]
            enum Action;

            #[derive(Clone, Copy, Debug, Default, Maybe)]
            #[derive_const(PartialEq, Eq)]
            enum State;

            _ => { 0x1b => [Action::Execute, State::Ground], }

            #[default]
            State::Ground => { 0x20..=0x7e => Action::Print, }
        }).unwrap();

        let expanded = render(machine);
        assert_eq!(expanded.matches("derive (Clone , Copy , Debug , Default , Maybe)").count(), 2);
        assert!(expanded.contains("pub enum Action"));
        assert!(expanded.contains("pub enum State"));
    }

    #[test]
    fn enum_declarations_remain_optional() {
        let machine: Machine = syn::parse2(quote! {
            #[default]
            State::Ground => { 0x20..=0x7e => Action::Print, }
        }).unwrap();
        let expanded = render(machine);
        // default derive now includes Clone (Copy requires it) + Eq for comparisons
        assert_eq!(
            expanded.matches("derive (Clone , Copy , Debug , Default , PartialEq , Eq)").count(),
            2
        );
    }

    #[test]
    fn rejects_duplicate_enum_declarations() {
        let err = match syn::parse2::<Machine>(quote! { enum Action; enum Action; }) {
            Ok(_) => panic!("duplicate declaration should fail"),
            Err(e) => e,
        };
        assert!(err.to_string().contains("duplicate enum declaration"));
    }

    #[test]
    fn transition_returns_tuple() {
        let machine: Machine = syn::parse2(quote! {
            _ => { 0x1b => [Action::Execute, State::Ground], } // folds in (not shadowed)
            State::Ground => {
                0x20..=0x7e => Action::Print,   // ActionOnly  => (Print, None)
                0x00..=0x1a => State::Escape,   // StateOnly   => (None, Escape)
            }
        }).unwrap();
        let expanded = render(machine);
        assert!(expanded.contains("-> (Action , State)"));
        assert!(expanded.contains("(Action :: Execute , State :: Ground)")); // ActionAndState
        assert!(expanded.contains("(Action :: Print , State :: None)"));
        assert!(expanded.contains("(Action :: None , State :: Escape)"));
    }

    #[test]
    fn entry_and_exit_are_methods() {
        let machine: Machine = syn::parse2(quote! {
            State::Ground => {
                on_entry => Action::Clear,
                on_exit => Action::Reset,
                0x20..=0x7e => Action::Print,
            }
        }).unwrap();
        let expanded = render(machine);
        assert!(expanded.contains("pub const fn entry (self) -> Action"));
        assert!(expanded.contains("pub const fn exit (self) -> Action"));
    }

    /// Regression: a specific byte written *before* a broader overlapping range
    /// must survive (match semantics = first arm wins). Previously the broad
    /// range silently clobbered it (last-write-wins).
    #[test]
    fn first_match_wins_keeps_specific_byte() {
        let machine: Machine = syn::parse2(quote! {
            State::Ground => {
                0x07 => Action::Bel,
                0x00..=0x1f => Action::Execute,
            }
        }).unwrap();
        let expanded = render(machine);
        assert!(expanded.contains("0x07 => (Action :: Bel"), "0x07 was clobbered:\n{expanded}");
        assert!(expanded.contains("0x00 ..= 0x06 => (Action :: Execute"));
        assert!(expanded.contains("0x08 ..= 0x1f => (Action :: Execute"));
    }

    #[test]
    fn exclusive_and_inclusive_ranges() {
        let machine: Machine = syn::parse2(quote! {
            State::Ground => {
                0x20..0x30 => Action::A,   // exclusive => 0x20..=0x2f
                0x40..=0x50 => Action::B,  // inclusive => 0x40..=0x50
            }
        }).unwrap();
        let expanded = render(machine);
        assert!(expanded.contains("0x20 ..= 0x2f => (Action :: A"));
        assert!(expanded.contains("0x40 ..= 0x50 => (Action :: B"));
    }

    #[test]
    fn local_overrides_anywhere() {
        let machine: Machine = syn::parse2(quote! {
            _ => { 0x00..=0x7f => Action::Execute, }
            State::Ground => { 0x20..=0x7f => Action::Print, }
        }).unwrap();
        let expanded = render(machine);
        assert!(expanded.contains("0x00 ..= 0x1f => (Action :: Execute"));
        assert!(expanded.contains("0x20 ..= 0x7f => (Action :: Print"));
    }

    #[test]
    fn out_of_range_byte_is_rejected() {
        let res = syn::parse2::<Machine>(quote! { State::G => { 0x100 => Action::X, } }).and_then(expand);
        assert!(res.is_err());
    }

    #[test]
    fn descending_range_is_rejected() {
        let res = syn::parse2::<Machine>(quote! { State::G => { 0x40..=0x20 => Action::X, } }).and_then(expand);
        assert!(res.is_err());
    }
}