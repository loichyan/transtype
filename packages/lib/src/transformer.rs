use crate::{builtin, kw, state, ForkCommand, ListOf, NamedArg, PipeCommand, TransformState};
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

    pub fn with_pipe(&mut self, pipe: ListOf<PipeCommand>) -> &mut Self {
        self.pipe
            .content
            .extend(pipe.into_inner().into_iter().rev());
        self
    }

    pub fn with_extra(&mut self, extra: TokenStream) -> &mut Self {
        self.extra.content.extend(extra);
        self
    }

    pub fn with_marker(&mut self, marker: TokenStream) -> &mut Self {
        self.marker.content.extend(marker);
        self
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
        self.with_marker(quote_spanned!(span=> ::transtype::#path!{}));
    }

    fn set_this(&mut self, this: Path) {
        self.this.content = this;
    }

    fn next_pipe(&mut self) -> Option<PipeCommand> {
        self.pipe.content.pop()
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

    fn take(&mut self) -> Self {
        Self {
            this: self.this.clone(),
            pipe: self.pipe.take(),
            extra: self.extra.take(),
            marker: self.marker.clone(),
        }
    }

    fn take_extra(&mut self) -> TokenStream {
        std::mem::take(&mut self.extra.content)
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

    pub(crate) fn transform_with<T: Executor>(
        self,
        mut rest: TransformRest,
    ) -> Result<TokenStream> {
        let mut tokens = match transform_impl(self, &mut rest, WithBuiltinExecutor::<T>::execute) {
            Ok(tokens) => {
                if cfg!(debug_assertions) {
                    assert!(rest.pipe.content.is_empty());
                    assert!(rest.extra.content.is_empty());
                }
                tokens
            }
            Err(err) => err.into_compile_error(),
        };
        tokens.extend(std::mem::take(&mut rest.marker.content));
        Ok(tokens)
    }
}

fn transform_impl(
    mut state: TransformState,
    rest: &mut TransformRest,
    execute: fn(PipeCommand, DeriveInput, &mut TransformRest) -> Result<ExecuteState>,
) -> Result<TokenStream> {
    type State = TransformState;
    type Ty = crate::state::Type;

    Ok(loop {
        match state.0 {
            Ty::Consume(state::Consume { mut data }) => {
                if !rest.pipe.content.is_empty() {
                    return Err(syn::Error::new(
                        rest.span(),
                        "a consume command should not be followed by other commands",
                    ));
                }
                data.extend(rest.take_extra());
                break data;
            }
            Ty::Debug(state::Debug { data, args }) => {
                let span = rest.span();
                let name = format_ident!("DEBUG_{}", data.ident, span = span);
                let rest = rest.take();
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
            Ty::Fork(state::Fork { data, fork }) => {
                if let Some(fork) = fork {
                    let mut tokens = TokenStream::default();
                    for ForkCommand(fork) in fork {
                        let mut data = data.clone();
                        data.ident = fork.name;
                        let mut rest = rest.fork(fork.content);
                        tokens.extend(transform_impl(
                            State::pipe(data).build(),
                            &mut rest,
                            execute,
                        )?);
                    }
                    state = State::consume(tokens).build();
                } else {
                    state = State::consume(data.into_token_stream()).build();
                }
            }
            Ty::Pipe(state::Pipe { data }) => match rest.next_pipe() {
                Some(cmd) => {
                    rest.set_this(cmd.path().clone());
                    let output = match execute(cmd, data, rest) {
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
                            let span = rest.span();
                            let PipeCommand { path, args, .. } = cmd;
                            let rest = rest.take();
                            state = State::consume(quote_spanned!(span=>
                                #path! {
                                    data={#data}
                                    args={#args}
                                    rest={#rest}
                                }
                            ))
                            .build();
                        }
                    }
                }
                None => {
                    return Err(syn::Error::new(
                        rest.span(),
                        "a pipe command should be consumed",
                    ));
                }
            },
            Ty::Resume(state::Resume { path }) => {
                rest.set_this(path.clone());
                let span = rest.span();
                let rest = rest.take();
                state = State::consume(quote_spanned!(span=> #path! { rest={#rest} })).build();
            }
            Ty::Save(state::Save { data }) => {
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
