use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{
    Attribute, Error, Expr, Ident, LitInt, Path, Token, braced, parenthesized, parse_macro_input,
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
pub struct StateBlock {
    attrs: Vec<Attribute>,
    key: StateKey,
    entries: Vec<Entry>,
}
#[derive(Clone)]
pub enum StateKey {
    Anywhere,
    State(Path),
}
pub enum Entry {
    OnEntry(Expr),
    OnExit(Expr),
    Transition { byte: BytePattern, effect: Effect },
}

#[derive(Clone)]
pub enum BytePattern {
    Single(LitInt),
    /// `start..end` (exclusive) or `start..=end` (inclusive)
    Range {
        start: LitInt,
        end: LitInt,
        inclusive: bool,
    },
    /// `_` – matches any byte
    CatchAll,
}

pub struct Effect {
    pub action: Expr,
    pub state: Expr,
}

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
                    _ => {
                        return Err(Error::new_spanned(
                            name,
                            "expected enum Action; or enum State;",
                        ));
                    }
                };
                if destination.replace(attrs).is_some() {
                    return Err(Error::new_spanned(name, "duplicate enum declaration"));
                }
                continue;
            }
            blocks.push(StateBlock::parse_with_attrs(input, attrs)?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Self {
            action_attrs,
            state_attrs,
            blocks,
        })
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
        let body;
        braced!(body in input);
        let mut entries = Vec::new();
        while !body.is_empty() {
            entries.push(body.parse()?);
            if body.peek(Token![,]) {
                body.parse::<Token![,]>()?;
            }
        }
        Ok(Self {
            attrs,
            key,
            entries,
        })
    }
}
impl Parse for StateKey {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(Token![_]) {
            input.parse::<Token![_]>()?;
            Ok(Self::Anywhere)
        } else {
            Ok(Self::State(input.parse()?))
        }
    }
}
impl Parse for Entry {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(Ident) {
            let fork = input.fork();
            let ident: Ident = fork.parse()?;
            match ident.to_string().as_str() {
                "on_entry" => {
                    input.parse::<Ident>()?;
                    input.parse::<Token![=>]>()?;
                    return Ok(Self::OnEntry(input.parse()?));
                }
                "on_exit" => {
                    input.parse::<Ident>()?;
                    input.parse::<Token![=>]>()?;
                    return Ok(Self::OnExit(input.parse()?));
                }
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
        if input.peek(Token![_]) {
            input.parse::<Token![_]>()?;
            return Ok(Self::CatchAll);
        }
        let start: LitInt = input.parse()?;
        // NB: `..=` must be checked before `..`, since `peek(Token![..])` is also
        // true at the start of a `..=` token.
        if input.peek(Token![..=]) {
            input.parse::<Token![..=]>()?;
            let end = input.parse()?;
            Ok(Self::Range {
                start,
                end,
                inclusive: true,
            })
        } else if input.peek(Token![..]) {
            input.parse::<Token![..]>()?;
            let end = input.parse()?;
            Ok(Self::Range {
                start,
                end,
                inclusive: false,
            })
        } else {
            Ok(Self::Single(start))
        }
    }
}
impl Parse for Effect {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let inner;
        parenthesized!(inner in input);
        let state: Expr = inner.parse()?;
        inner.parse::<Token![,]>()?;
        let action: Expr = inner.parse()?;
        if !inner.is_empty() {
            return Err(inner.error("expected exactly two items: (State::..., Action::...)"));
        }
        Ok(Self { action, state })
    }
}

#[derive(Default)]
pub struct OrderedNames {
    names: Vec<Ident>,
}
impl OrderedNames {
    fn insert(&mut self, ident: Ident) {
        if self.names.iter().any(|x| x == &ident) {
            return;
        }
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
                        Entry::OnEntry(action) | Entry::OnExit(action) => {
                            return Err(Error::new_spanned(
                                action,
                                "on_entry / on_exit are not valid in the _ anywhere block",
                            ));
                        }
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
            pub  fn transition(self, byte: u8) -> (State, Action) {
                match self {
                    #(#state_arms)*
                    _ => panic!("Undefined transition {self:?} + 0x{byte:2x}"),
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
    let Effect { action, state } = effect;
    quote! { (#state, #action) }
}

#[derive(Clone, Copy)]
pub enum LifecycleKind {
    Entry,
    Exit,
}
fn state_lifecycle_arms(
    state_blocks: &[&StateBlock],
    kind: LifecycleKind,
) -> syn::Result<Vec<TokenStream2>> {
    let mut arms = Vec::new();
    for block in state_blocks {
        let StateKey::State(state_path) = &block.key else {
            unreachable!()
        };
        let mut found: Option<&Expr> = None;
        for entry in &block.entries {
            let candidate = match (kind, entry) {
                (LifecycleKind::Entry, Entry::OnEntry(action)) => Some(action),
                (LifecycleKind::Exit, Entry::OnExit(action)) => Some(action),
                _ => None,
            };
            if let Some(action) = candidate {
                if found.is_some() {
                    return Err(Error::new_spanned(
                        action,
                        "duplicate lifecycle action for this state",
                    ));
                }
                found = Some(action);
            }
        }
        if let Some(action) = found {
            arms.push(quote! { #state_path => #action });
        }
    }
    Ok(arms)
}

fn transition_state_arms(
    state_blocks: &[&StateBlock],
    anywhere: &[(&BytePattern, &Effect)],
) -> syn::Result<Vec<TokenStream2>> {
    let mut arms = Vec::new();
    for block in state_blocks {
        let StateKey::State(state_path) = &block.key else {
            unreachable!()
        };
        let local: Vec<_> = block
            .entries
            .iter()
            .filter_map(|e| match e {
                Entry::Transition { byte, effect } => Some((byte, effect)),
                _ => None,
            })
            .collect();
        let merged_arms = build_merged_byte_arms(anywhere, &local)?;
        arms.push(quote! {
            #state_path => match byte {
                #(#merged_arms)*
                _ => (#state_path, Action::None),
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
            if slot.is_none() {
                *slot = Some(*effect);
            }
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
    for b in 0..256usize {
        byte_map[b] = local_map[b].or(anywhere_map[b]);
    }

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
            (Some((start, cur)), None) => {
                merged.push((start, b - 1, cur));
                run = None;
            }
            _ => {}
        }
    }
    if let Some((start, eff)) = run {
        merged.push((start, 255, eff));
    }

    Ok(merged
        .into_iter()
        .map(|(start, end, effect)| {
            let rhs = transition_rhs(effect);
            let pat = byte_range_tokens(start, end);
            quote! { #pat => #rhs, }
        })
        .collect())
}

/// Resolve a `BytePattern` to an inclusive `(lo, hi)` byte range, with spanned
/// errors for out-of-range or empty ranges (no silent clamping).
fn byte_pattern_range(byte: &BytePattern) -> syn::Result<(u8, u8)> {
    match byte {
        BytePattern::CatchAll => Ok((0, 255)),
        BytePattern::Single(lit) => {
            let v = lit.base10_parse::<u8>()?;
            Ok((v, v))
        }
        BytePattern::Range {
            start,
            end,
            inclusive,
        } => {
            let lo = start.base10_parse::<u16>()?;
            let end_val = end.base10_parse::<u16>()?;
            let hi = if *inclusive {
                end_val
            } else {
                if end_val == 0 {
                    return Err(Error::new_spanned(
                        end,
                        "exclusive range end must be greater than 0",
                    ));
                }
                end_val - 1
            };
            if lo > 0xff {
                return Err(Error::new_spanned(
                    start,
                    "byte literal out of range (expected 0..=255)",
                ));
            }
            if hi > 0xff {
                return Err(Error::new_spanned(
                    end,
                    "byte literal out of range (expected 0..=255)",
                ));
            }
            if lo > hi {
                return Err(Error::new_spanned(end, "empty or descending byte range"));
            }
            Ok((lo as u8, hi as u8))
        }
    }
}

fn effects_equal(a: &Effect, b: &Effect) -> bool {
    effect_to_string(a) == effect_to_string(b)
}
fn effect_to_string(effect: &Effect) -> String {
    let Effect { action, state } = effect;
    quote!(#state, #action).to_string()
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
    for block in &machine.blocks {
        if let StateKey::State(path) = &block.key {
            states.insert(last_path_ident(path)?);
        }
        for entry in &block.entries {
            if let Entry::Transition { effect, .. } = entry {
                collect_state_from_effect(effect, &mut states)?;
            }
        }
    }
    let default = default_state_ident(machine)?.unwrap_or_else(|| syn::parse_quote! { None });
    let variants = states.names.into_iter().map(|name| {
        if name == default {
            quote! { #[default] #name, }
        } else {
            quote! { #name, }
        }
    });
    let attrs = machine.state_attrs.as_ref().map_or_else(
        || quote! { #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)] },
        |attrs| quote! { #(#attrs)* },
    );
    Ok(quote! { #attrs pub enum State { #(#variants)* } })
}
fn generate_action_enum(machine: &Machine) -> syn::Result<TokenStream2> {
    let mut actions = OrderedNames::default();
    for block in &machine.blocks {
        for entry in &block.entries {
            match entry {
                Entry::OnEntry(action) | Entry::OnExit(action) => {
                    actions.insert(action_ident(action)?)
                }
                Entry::Transition { effect, .. } => {
                    collect_action_from_effect(effect, &mut actions)?
                }
            }
        }
    }
    let variants = actions.names.into_iter().map(|name| {
        if name == "None" {
            quote! { #[default] None, }
        } else {
            quote! { #name, }
        }
    });
    let attrs = machine.action_attrs.as_ref().map_or_else(
        || quote! { #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)] },
        |attrs| quote! { #(#attrs)* },
    );
    Ok(quote! { #attrs pub enum Action { #(#variants)* } })
}
fn collect_state_from_effect(effect: &Effect, states: &mut OrderedNames) -> syn::Result<()> {
    states.insert(state_ident(&effect.state)?);
    Ok(())
}
fn collect_action_from_effect(effect: &Effect, actions: &mut OrderedNames) -> syn::Result<()> {
    actions.insert(action_ident(&effect.action)?);
    Ok(())
}
fn state_ident(expr: &Expr) -> syn::Result<Ident> {
    enum_variant_ident(expr, "State")
}
fn action_ident(expr: &Expr) -> syn::Result<Ident> {
    enum_variant_ident(expr, "Action")
}
fn enum_variant_ident(expr: &Expr, enum_name: &str) -> syn::Result<Ident> {
    let Expr::Path(path) = expr else {
        return Err(Error::new_spanned(
            expr,
            format!("expected {enum_name}::Variant"),
        ));
    };
    let mut segments = path.path.segments.iter();
    let Some(root) = segments.next() else {
        return Err(Error::new_spanned(
            expr,
            format!("expected {enum_name}::Variant"),
        ));
    };
    if root.ident != enum_name {
        return Err(Error::new_spanned(
            expr,
            format!("expected {enum_name}::Variant"),
        ));
    }
    let Some(variant) = segments.next() else {
        return Err(Error::new_spanned(
            expr,
            format!("expected {enum_name}::Variant"),
        ));
    };
    if segments.next().is_some() {
        return Err(Error::new_spanned(
            expr,
            format!("expected {enum_name}::Variant, not a nested path"),
        ));
    }
    Ok(variant.ident.clone())
}
fn last_path_ident(path: &Path) -> syn::Result<Ident> {
    path.segments
        .last()
        .map(|s| s.ident.clone())
        .ok_or_else(|| Error::new_spanned(path, "expected state path"))
}
fn is_default_attr(attr: &Attribute) -> bool {
    attr.path().is_ident("default")
}
fn default_state_ident(machine: &Machine) -> syn::Result<Option<Ident>> {
    let mut found: Option<Ident> = None;
    for block in &machine.blocks {
        if !block.attrs.iter().any(is_default_attr) {
            continue;
        }
        let StateKey::State(path) = &block.key else {
            return Err(Error::new_spanned(
                block.attrs.first().unwrap(),
                "#[default] is only valid on a concrete State arm, not on _",
            ));
        };
        let ident = last_path_ident(path)?;
        if let Some(prev) = &found {
            return Err(Error::new_spanned(
                path,
                format!("duplicate #[default] state; already marked State::{prev} as default"),
            ));
        }
        found = Some(ident);
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    fn render(m: Machine) -> String {
        expand(m).unwrap().to_string()
    }

    #[test]
    fn enum_declaration_attributes_are_applied() {
        let machine: Machine = syn::parse2(quote! {
            #[derive(Clone, Copy, Debug, Default, Maybe)]
            #[derive_const(PartialEq, Eq)]
            enum Action;

            #[derive(Clone, Copy, Debug, Default, Maybe)]
            #[derive_const(PartialEq, Eq)]
            enum State;

            _ => { 0x1b => (State::Ground, Action::Execute), }

            #[default]
            State::Ground => { 0x20..=0x7e => (State::Ground, Action::Print), }
        })
        .unwrap();

        let expanded = render(machine);
        assert_eq!(
            expanded
                .matches("derive (Clone , Copy , Debug , Default , Maybe)")
                .count(),
            2
        );
        assert!(expanded.contains("pub enum Action"));
        assert!(expanded.contains("pub enum State"));
    }

    #[test]
    fn enum_declarations_remain_optional() {
        let machine: Machine = syn::parse2(quote! {
            #[default]
            State::Ground => { 0x20..=0x7e => (State::Ground, Action::Print), }
        })
        .unwrap();
        let expanded = render(machine);
        assert_eq!(
            expanded
                .matches("derive (Clone , Copy , Debug , Default , PartialEq , Eq)")
                .count(),
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
            _ => { 0x1b => (State::Ground, Action::Execute), }
            State::Ground => {
                0x20..=0x7e => (State::None, Action::Print),
                0x00..=0x1a => (State::Escape, Action::None),
            }
        })
        .unwrap();
        let expanded = render(machine);
        assert!(expanded.contains("-> (State , Action)"));
        assert!(expanded.contains("(State :: Ground , Action :: Execute)"));
        assert!(expanded.contains("(State :: None , Action :: Print)"));
        assert!(expanded.contains("(State :: Escape , Action :: None)"));
    }

    #[test]
    fn entry_and_exit_are_methods() {
        let machine: Machine = syn::parse2(quote! {
            State::Ground => {
                on_entry => Action::Clear,
                on_exit => Action::Reset,
                0x20..=0x7e => (State::Ground, Action::Print),
            }
        })
        .unwrap();
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
                0x07 => (State::Ground, Action::Bel),
                0x00..=0x1f => (State::Ground, Action::Execute),
            }
        })
        .unwrap();
        let expanded = render(machine);
        assert!(
            expanded.contains("0x07 => (State :: Ground , Action :: Bel"),
            "0x07 was clobbered:\n{expanded}"
        );
        assert!(expanded.contains("0x00 ..= 0x06 => (State :: Ground , Action :: Execute"));
        assert!(expanded.contains("0x08 ..= 0x1f => (State :: Ground , Action :: Execute"));
    }

    #[test]
    fn exclusive_and_inclusive_ranges() {
        let machine: Machine = syn::parse2(quote! {
            State::Ground => {
                0x20..0x30 => (State::None, Action::A),
                0x40..=0x50 => (State::None, Action::B),
            }
        })
        .unwrap();
        let expanded = render(machine);
        assert!(expanded.contains("0x20 ..= 0x2f => (State :: None , Action :: A"));
        assert!(expanded.contains("0x40 ..= 0x50 => (State :: None , Action :: B"));
    }

    #[test]
    fn local_overrides_anywhere() {
        let machine: Machine = syn::parse2(quote! {
            _ => { 0x00..=0x7f => (State::Ground, Action::Execute), }
            State::Ground => { 0x20..=0x7f => (State::Ground, Action::Print), }
        })
        .unwrap();
        let expanded = render(machine);
        assert!(expanded.contains("0x00 ..= 0x1f => (State :: Ground , Action :: Execute"));
        assert!(expanded.contains("0x20 ..= 0x7f => (State :: Ground , Action :: Print"));
    }

    #[test]
    fn out_of_range_byte_is_rejected() {
        let res =
            syn::parse2::<Machine>(quote! { State::G => { 0x100 => (State::None, Action::X), } })
                .and_then(expand);
        assert!(res.is_err());
    }

    #[test]
    fn descending_range_is_rejected() {
        let res = syn::parse2::<Machine>(
            quote! { State::G => { 0x40..=0x20 => (State::None, Action::X), } },
        )
        .and_then(expand);
        assert!(res.is_err());
    }

    #[test]
    fn catch_all_in_state_block() {
        let machine: Machine = syn::parse2(quote! {
            State::Ground => {
                0x20..=0x7e => (State::Ground, Action::Print),
                _ => (State::Ground, Action::Execute),
            }
        })
        .unwrap();
        let expanded = render(machine);
        // Specific pattern survives
        assert!(expanded.contains("0x20 ..= 0x7e => (State :: Ground , Action :: Print)"));
        // Catch-all fills the gaps
        assert!(expanded.contains("0x00 ..= 0x1f => (State :: Ground , Action :: Execute)"));
        assert!(expanded.contains("0x7f ..= 0xff => (State :: Ground , Action :: Execute)"));
    }

    #[test]
    fn catch_all_overrides_anywhere() {
        let machine: Machine = syn::parse2(quote! {
            _ => { 0x00..=0xff => (State::Ground, Action::Execute), }
            State::Ground => {
                0x20..=0x7e => (State::Ground, Action::Print),
                _ => (State::Ground, Action::Ignore),
            }
        })
        .unwrap();
        let expanded = render(machine);
        // Local specific pattern wins
        assert!(expanded.contains("(State :: Ground , Action :: Print)"));
        // Local catch-all wins over anywhere
        assert!(expanded.contains("(State :: Ground , Action :: Ignore)"));
        // Anywhere Execute should NOT appear for this state
        assert!(!expanded.contains("(State :: Ground , Action :: Execute)"));
    }

    #[test]
    fn catch_all_in_anywhere_block() {
        let machine: Machine = syn::parse2(quote! {
            _ => {
                0x1b => (State::Escape, Action::None),
                _ => (State::Ground, Action::Execute),
            }
            #[default]
            State::Ground => {
                0x20..=0x7e => (State::Ground, Action::Print),
            }
        })
        .unwrap();
        let expanded = render(machine);
        // Local specific still wins
        assert!(expanded.contains("(State :: Ground , Action :: Print)"));
        // Anywhere catch-all fills the rest
        assert!(expanded.contains("(State :: Ground , Action :: Execute)"));
        // Anywhere specific also works
        assert!(expanded.contains("(State :: Escape , Action :: None)"));
    }
}
