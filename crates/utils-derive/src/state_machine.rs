use quote::quote;
use syn::{
    braced, bracketed, Attribute, Error, Expr, Ident, LitInt, Path, Token, Visibility,
};
use syn::__private::TokenStream2;
use syn::parse::{Parse, ParseStream};

pub struct Machine {
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
    Transition {
        byte: BytePattern,
        effect: Effect,
    },
}

#[derive(Clone)]
pub enum BytePattern {
    Single(LitInt),
    RangeInclusiveSyntax {
        start: LitInt,
        end: LitInt,
    },
}

pub enum Effect {
    ActionOnly(Expr),
    StateOnly(Expr),
    ActionAndState {
        action: Expr,
        state: Expr,
    },
}

impl Parse for Machine {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut blocks = Vec::new();

        while !input.is_empty() {
            blocks.push(input.parse()?);

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self { blocks })
    }
}

impl Parse for StateBlock {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

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

        Ok(Self { attrs, key, entries })
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
                    let action: Expr = input.parse()?;
                    return Ok(Self::OnEntry(action));
                }
                "on_exit" => {
                    input.parse::<Ident>()?;
                    input.parse::<Token![=>]>()?;
                    let action: Expr = input.parse()?;
                    return Ok(Self::OnExit(action));
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
        let start: LitInt = input.parse()?;

        if input.peek(Token![..]) {
            input.parse::<Token![..]>()?;
            let end: LitInt = input.parse()?;

            Ok(Self::RangeInclusiveSyntax { start, end })
        } else {
            Ok(Self::Single(start))
        }
    }
}

impl Parse for Effect {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(syn::token::Bracket) {
            let inner;
            bracketed!(inner in input);

            let action: Expr = inner.parse()?;
            inner.parse::<Token![,]>()?;
            let state: Expr = inner.parse()?;

            if !inner.is_empty() {
                return Err(inner.error("expected exactly two items: [Action::..., State::...]"));
            }

            return Ok(Self::ActionAndState { action, state });
        }

        let expr: Expr = input.parse()?;

        match expr_root_ident(&expr).as_deref() {
            Some("Action") => Ok(Self::ActionOnly(expr)),
            Some("State") => Ok(Self::StateOnly(expr)),
            _ => Err(Error::new_spanned(
                expr,
                "expected Action::..., State::..., or [Action::..., State::...]",
            )),
        }
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
                        Entry::Transition { byte, effect } => {
                            anywhere.push((byte, effect));
                        }
                        Entry::OnEntry(action) | Entry::OnExit(action) => {
                            return Err(Error::new_spanned(
                                action,
                                "on_entry / on_exit are not valid in the _ anywhere block",
                            ));
                        }
                    }
                }
            }
            StateKey::State(_) => {
                state_blocks.push(block);
            }
        }
    }

    let generated_state_enum = generate_state_enum(&machine)?;
    let generated_action_enum = generate_action_enum(&machine)?;

    let state_arms = transition_state_arms(&state_blocks, &anywhere);
    let anywhere_byte_arms = transition_byte_arms(&anywhere);

    let entry_arms = state_lifecycle_arms(&state_blocks, LifecycleKind::Entry)?;
    let exit_arms = state_lifecycle_arms(&state_blocks, LifecycleKind::Exit)?;

    Ok(quote! {
        #generated_state_enum

        #generated_action_enum

        #[inline]
        pub const fn transition(state: State, byte: u8) -> (Action, State) {
            match state {
                #(#state_arms)*
                state => match byte {
                    #(#anywhere_byte_arms)*
                    _ => (Action::None, state),
                },
            }
        }

        #[inline]
        pub const fn entry_action(state: State) -> Action {
            match state {
                #(#entry_arms)*
                _ => Action::None,
            }
        }

        #[inline]
        pub const fn exit_action(state: State) -> Action {
            match state {
                #(#exit_arms)*
                _ => Action::None,
            }
        }
    })
}
fn transition_rhs(effect: &Effect) -> TokenStream2 {
    match effect {
        Effect::ActionOnly(action) => {
            quote! { (#action, State::None) }
        }
        Effect::StateOnly(next_state) => {
            quote! { (Action::None, #next_state) }
        }
        Effect::ActionAndState { action, state } => {
            quote! { (#action, #state) }
        }
    }
}

fn byte_pattern_tokens(byte: &BytePattern) -> TokenStream2 {
    match byte {
        BytePattern::Single(lit) => {
            quote! { #lit }
        }
        BytePattern::RangeInclusiveSyntax { start, end } => {
            quote! { #start..=#end }
        }
    }
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
            unreachable!();
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
            arms.push(quote! {
                #state_path => #action,
            });
        }
    }

    Ok(arms)
}

fn expr_root_ident(expr: &Expr) -> Option<String> {
    let Expr::Path(path) = expr else {
        return None;
    };

    path.path
        .segments
        .first()
        .map(|segment| segment.ident.to_string())
}

fn transition_state_arms(
    state_blocks: &[&StateBlock],
    anywhere: &[(&BytePattern, &Effect)],
) -> Vec<TokenStream2> {
    let mut arms = Vec::new();

    for block in state_blocks {
        let StateKey::State(state_path) = &block.key else {
            unreachable!();
        };

        let anywhere_arms = transition_byte_arms(anywhere);

        let local: Vec<_> = block
            .entries
            .iter()
            .filter_map(|entry| match entry {
                Entry::Transition { byte, effect } => Some((byte, effect)),
                Entry::OnEntry(_) | Entry::OnExit(_) => None,
            })
            .collect();

        let local_arms = transition_byte_arms(&local);

        arms.push(quote! {
            #state_path => match byte {
                #(#anywhere_arms)*
                #(#local_arms)*
                _ => (Action::None, #state_path),
            },
        });
    }

    arms
}

fn transition_byte_arms(
    transitions: &[(&BytePattern, &Effect)],
) -> Vec<TokenStream2> {
    transitions
        .iter()
        .map(|(byte, effect)| {
            let byte_pat = byte_pattern_tokens(byte);
            let rhs = transition_rhs(effect);

            quote! {
                #byte_pat => #rhs,
            }
        })
        .collect()
}

fn generate_state_enum(machine: &Machine) -> syn::Result<TokenStream2> {
    let mut states = OrderedNames::default();

    states.insert(syn::parse_quote! { None });

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

    let default = default_state_ident(machine)?
        .unwrap_or_else(|| syn::parse_quote! { None });

    let variants = states.names.into_iter().map(|name| {
        if name == default {
            quote! {
                #[default]
                #name,
            }
        } else {
            quote! {
                #name,
            }
        }
    });

    Ok(quote! {
        #[derive(Copy, Debug, Default)]
        #[derive_const(Clone, PartialEq, Eq)]
        pub enum State {
            #(#variants)*
        }
    })
}
fn generate_action_enum(machine: &Machine) -> syn::Result<TokenStream2> {
    let mut actions = OrderedNames::default();

    actions.insert(syn::parse_quote! { None });

    for block in &machine.blocks {
        for entry in &block.entries {
            match entry {
                Entry::OnEntry(action) | Entry::OnExit(action) => {
                    actions.insert(action_ident(action)?);
                }
                Entry::Transition { effect, .. } => {
                    collect_action_from_effect(effect, &mut actions)?;
                }
            }
        }
    }

    let variants = actions.names.into_iter().map(|name| {
        if name == "None" {
            quote! {
                #[default]
                None,
            }
        } else {
            quote! {
                #name,
            }
        }
    });

    Ok(quote! {
        #[derive(Copy, Debug, Default)]
        #[derive_const(Clone, PartialEq, Eq)]
        pub enum Action {
            #(#variants)*
        }
    })
}
fn collect_state_from_effect(effect: &Effect, states: &mut OrderedNames) -> syn::Result<()> {
    match effect {
        Effect::ActionOnly(_) => {}
        Effect::StateOnly(state) => {
            states.insert(state_ident(state)?);
        }
        Effect::ActionAndState { state, .. } => {
            states.insert(state_ident(state)?);
        }
    }

    Ok(())
}

fn collect_action_from_effect(effect: &Effect, actions: &mut OrderedNames) -> syn::Result<()> {
    match effect {
        Effect::ActionOnly(action) => {
            actions.insert(action_ident(action)?);
        }
        Effect::StateOnly(_) => {}
        Effect::ActionAndState { action, .. } => {
            actions.insert(action_ident(action)?);
        }
    }

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
        .map(|segment| segment.ident.clone())
        .ok_or_else(|| Error::new_spanned(path, "expected state path"))
}fn is_default_attr(attr: &Attribute) -> bool {
    attr.path().is_ident("default")
}

fn default_state_ident(machine: &Machine) -> syn::Result<Option<Ident>> {
    let mut found: Option<Ident> = None;

    for block in &machine.blocks {
        let has_default = block.attrs.iter().any(is_default_attr);

        if !has_default {
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