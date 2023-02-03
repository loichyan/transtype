use crate::{
    builtin, kw, state, transform::StateHook, ForkCommand, ListOf, NamedArg, PipeCommand,
    TransformState,
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote_spanned, ToTokens};
use std::marker::PhantomData;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    DeriveInput, Path, Result,
};

pub trait Transformer: Sized {
    type Args: Parse;

    fn transform(
        data: DeriveInput,
        args: Self::Args,
        rest: &mut TransformRest,
    ) -> Result<TransformState>;
}

pub trait Executor: Sized {
    fn execute(
        cmd: PipeCommand,
        data: DeriveInput,
        rest: &mut TransformRest,
    ) -> Result<ExecuteState>;
}

pub enum ExecuteState {
    Executed { state: TransformState },
    Unsupported { cmd: PipeCommand, data: DeriveInput },
}

impl From<TransformState> for ExecuteState {
    fn from(state: TransformState) -> Self {
        Self::Executed { state }
    }
}

pub struct TransformInput<T: Transformer> {
    data: NamedArg<kw::data, DeriveInput>,
    args: NamedArg<kw::args, T::Args>,
    rest: NamedArg<kw::rest, TransformRest>,
}

impl<T: Transformer> TransformInput<T> {
    pub fn transform(self) -> Result<TokenStream> {
        self.transform_with::<NoopExecutor>()
    }

    pub fn transform_with<Exe: Executor>(self) -> Result<TokenStream> {
        let mut rest = self.rest.content;
        T::transform(self.data.content, self.args.content, &mut rest)?.transform_with::<Exe>(rest)
    }
}

impl<T: Transformer> Parse for TransformInput<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            data: input.parse()?,
            args: input.parse()?,
            rest: input.parse()?,
        })
    }
}

pub struct TransformRest {
    this: NamedArg<kw::this, Path>,
    pipe: NamedArg<kw::pipe, ListOf<PipeCommand>>,
    extra: NamedArg<kw::extra, TokenStream>,
    marker: NamedArg<kw::marker, TokenStream>,
}

impl TransformRest {
    /// Get this span of current command.
    pub fn span(&self) -> Span {
        let path = &self.this.content;
        path.segments
            .last()
            .map(|t| t.ident.span())
            .unwrap_or_else(|| path.span())
    }

    /// Appends tokens which will always be expanded to the final stream.
    pub fn append_mark(&mut self, marker: TokenStream) {
        self.marker.content.extend(marker);
    }

    pub(crate) fn empty(path: Path) -> Self {
        Self {
            this: NamedArg::new(path),
            pipe: Default::default(),
            extra: Default::default(),
            marker: Default::default(),
        }
    }

    pub(crate) fn track_builtin(&mut self) {
        let span = self.span();
        let path = &self.this.content;
        self.append_mark(quote_spanned!(span=> ::transtype::#path!{}));
    }

    fn is_empty(&self) -> bool {
        self.pipe.content.is_empty()
    }

    fn set_this(&mut self, this: Path) {
        self.this.content = this;
    }

    fn next_pipe(&mut self) -> Option<PipeCommand> {
        self.pipe.content.pop()
    }

    fn hook(&mut self, hook: StateHook) {
        let StateHook {
            pipe,
            extra,
            marker,
        } = hook;
        if let Some(pipe) = pipe {
            self.prepend_pipe(pipe);
        }
        if let Some(extra) = extra {
            self.append_extra(extra);
        }
        if let Some(marker) = marker {
            self.append_mark(marker);
        }
    }

    fn prepend_pipe(&mut self, pipe: ListOf<PipeCommand>) {
        self.pipe
            .content
            .extend(pipe.into_inner().into_iter().rev());
    }

    fn append_extra(&mut self, extra: TokenStream) {
        self.extra.content.extend(extra);
    }

    fn fork(&self, mut pipe: ListOf<PipeCommand>) -> Self {
        pipe.reverse();
        Self {
            this: self.this.clone(),
            pipe: self.pipe.clone_with(pipe),
            extra: self.extra.clone(),
            marker: self.marker.clone(),
        }
    }

    fn take_extra(&mut self) -> TokenStream {
        std::mem::take(&mut self.extra.content)
    }

    fn take_mark(&mut self) -> TokenStream {
        std::mem::take(&mut self.marker.content)
    }
}

impl Parse for TransformRest {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            this: input.parse()?,
            pipe: input.parse()?,
            extra: input.parse()?,
            marker: input.parse()?,
        })
    }
}

impl ToTokens for TransformRest {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.this.to_tokens(tokens);
        self.pipe.to_tokens(tokens);
        self.extra.to_tokens(tokens);
        self.marker.to_tokens(tokens);
    }
}

impl TransformState {
    pub(crate) fn transform(self, rest: TransformRest) -> Result<TokenStream> {
        self.transform_with::<NoopExecutor>(rest)
    }

    pub(crate) fn transform_with<T: Executor>(self, rest: TransformRest) -> Result<TokenStream> {
        transform_impl(self, rest, WithBuiltinExecutor::<T>::execute)
    }
}

// TODO: put `marker` to the output when error occurs
fn transform_impl(
    mut state: TransformState,
    mut rest: TransformRest,
    execute: fn(PipeCommand, DeriveInput, &mut TransformRest) -> Result<ExecuteState>,
) -> Result<TokenStream> {
    type State = TransformState;
    type Ty = crate::state::Type;

    Ok(loop {
        match state.0 {
            Ty::Consume(state::Consume { mut data, hook }) => {
                rest.hook(hook);
                if !rest.is_empty() {
                    return Err(syn::Error::new(
                        rest.span(),
                        "a consume command should not be followed by other commands",
                    ));
                }
                data.extend(rest.take_extra());
                data.extend(rest.take_mark());
                break data;
            }
            Ty::Debug(state::Debug { data, args, hook }) => {
                rest.hook(hook);
                let span = rest.span();
                let name = format_ident!("DEBUG_{}", data.ident, span = span);
                let rest = TransformRest {
                    this: rest.this.clone(),
                    pipe: rest.pipe.take(),
                    extra: rest.extra.take(),
                    marker: rest.marker.clone(),
                };
                let data = quote_spanned!(span=>
                    data={#data}
                    args={#args}
                    rest={#rest}
                );
                let s = data.to_string();
                state = State::consume(quote_spanned!(span=>
                    macro_rules! #name {
                        () => {{ #s }};
                        (@$visit:path) => {
                            $visit! { #data }
                        };
                    }
                ))
                .build();
            }
            Ty::Fork(state::Fork { data, fork, hook }) => {
                rest.hook(hook);
                if let Some(fork) = fork {
                    let mut tokens = TokenStream::default();
                    for ForkCommand(fork) in fork {
                        let mut data = data.clone();
                        data.ident = fork.name;
                        let rest = rest.fork(fork.content);
                        tokens.extend(transform_impl(State::pipe(data).build(), rest, execute)?);
                    }
                    state = State::consume(tokens).build();
                } else {
                    state = State::consume(data.into_token_stream()).build();
                }
            }
            Ty::Pipe(state::Pipe { data, hook }) => {
                rest.hook(hook);
                match rest.next_pipe() {
                    Some(cmd) => {
                        rest.set_this(cmd.path().clone());
                        let output = match execute(cmd, data, &mut rest) {
                            Ok(t) => t,
                            Err(mut e) => {
                                e.combine(syn::Error::new(
                                    rest.span(),
                                    "an error occurs in this command",
                                ));
                                return Err(e);
                            }
                        };
                        match output {
                            ExecuteState::Executed { state: s } => state = s,
                            ExecuteState::Unsupported { cmd, data } => {
                                break cmd.execute(data, rest);
                            }
                        }
                    }
                    None => {
                        return Err(syn::Error::new(
                            rest.span(),
                            "a pipe command should be consumed",
                        ));
                    }
                }
            }
            Ty::Resume(state::Resume { path, hook }) => {
                rest.hook(hook);
                rest.set_this(path.clone());
                let span = rest.span();
                break quote_spanned!(span=>
                    #path! { rest={#rest} }
                );
            }
            Ty::Save(state::Save { data, hook }) => {
                rest.hook(hook);
                let span = rest.span();
                let name = &data.ident;
                let extra = rest.take_extra();
                state = State::consume(quote_spanned!(span=>
                    macro_rules! #name {
                        ($($args:tt)*) => {
                            ::transtype::__predefined! {
                                args={$($args)*}
                                data={#data}
                                extra={#extra}
                            }
                        };
                    }
                ))
                .build();
            }
        };
    })
}

struct NoopExecutor;

impl Executor for NoopExecutor {
    fn execute(cmd: PipeCommand, data: DeriveInput, _: &mut TransformRest) -> Result<ExecuteState> {
        Ok(ExecuteState::Unsupported { cmd, data })
    }
}

struct WithBuiltinExecutor<T>(PhantomData<T>);

impl<T: Executor> Executor for WithBuiltinExecutor<T> {
    fn execute(
        cmd: PipeCommand,
        data: DeriveInput,
        rest: &mut TransformRest,
    ) -> Result<ExecuteState> {
        Ok(match builtin::Executor::execute(cmd, data, rest)? {
            ExecuteState::Executed { state } => ExecuteState::Executed { state },
            ExecuteState::Unsupported { cmd, data } => T::execute(cmd, data, rest)?,
        })
    }
}
