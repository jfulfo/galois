# galois design

this file will initially be an amalgam of thoughts but eventually will be more organized.

## the ideal scripting language:

bash scripting is in general not the best interface for humans to communicate with computers.
other scripting languages have inheritted these flaws, namely:

- **unnatural**: it is well known that using ffmpeg without the docs is basically impossible
  alternative: we define a kind of universal inteface between
  programs/programming languages and our runtime.
- **uninternet**: existing languages struggle with the asynchronicity of the internet.
  this does not have to be the case. fix: automatic parallelization
- **interpreted**: this may be unavoidable, but ideally we want our scripts to be as close to
  the operating system as possible (i.e. in native)
- **paradigms**: we want our language to be declarative, procedural, affine dynamically typed or untyped,

we can further the automatic parallelization idea by using the notion of "foreign function calls".
we can have "holes" in our program where a foreign function call will return some value representable in
our language. this is where our universal interface idea come in. we want our language to be
[opportunistically parallel](https://arxiv.org/abs/2405.11361) (abstract from that paper):

> Large language models (LLMs) have shown impressive results at a wide-range of tasks.
> However, they have limitations, such as hallucinating facts and struggling with arithmetic.
> Recent work has addressed these issues with sophisticated decoding techniques.
> However, performant decoding, particularly for sophisticated techniques, relies crucially on parallelization and batching, which are difficult for developers.
> We make two observations:
>
> - Existing approaches are high-level domain-specific languages for gluing expensive black-box calls, but are not general or compositional;
> - LLM programs are essentially pure (all effects commute).
>
> Guided by these observations, we develop a novel, general-purpose lambda calculus for automatically parallelizing a wide-range of LLM interactions, without user intervention.
> The key difference versus standard lambda calculus is a novel "opportunistic" evaluation strategy, which steps independent parts of a program in parallel, dispatching black-box external calls as eagerly as possible, even while data-independent parts of the program are waiting for their own external calls to return.
> To maintain the simplicity of the language and to ensure uniformity of opportunistic evaluation, control-flow and looping constructs are implemented in-language, via Church encodings.

we conjecture that using lambda calculus and church encodings is unnecessary/PL gobbledygook.
we will first target the LLM scripting scene. necessarily this requires us to have python interoperability, or at least we
need to be able to make foreign function calls to python. we do have to start off generally enough for our foreign function call idea though.
after we get that, then we can focus on the bigger ideas of building a universal scripting language.
necessarily we need a language to remain general-purpose in order to avoid the pitfall of other DSLs.

## denotation

we can think of our language as a way of defining the "glue" morphisms in a certain category of programs.
there is a similar graph denotation for this.
the morphisms themselves (our pure language without foreign functions) is also a kind of category.
we need to define a notion of parallelizability for the pure language. it will need to be linear.
the graph denotation is the killer: _if_ we can formulate our language with graph combinators _then_
we can run our language with CUDA--combine infinite parallelizability with universal interoperability.
make galois like a tactic language for composing underlying functional terms? then the use of tacticals gives a clean tree like structure
we can then construct the graph isomorphism in the toolchain

## the works

a few notes on what we really want in the language:

- the language should be pythonic. the python interoperability should be very tight, like zig and c
- lots of sugar. we need a custom notation feature like in coq.
  this necessitates everything being a function in some way.
  the language should come with nothing but the compiler like coq.
  the stdlib will provide all the features needed to actually program.
  all the stdlib should actually do is provide the external foreign function call interfaces.
- the automatic paralellization is the kingmaker of our language.
  eventually making a compiler for cuda is very much desired, we want our language to be
  as parallel as possible
- we use `_` for spaces in name generally

# formalization

the following sections will focus on a more concrete view of the language.
note that our language is untyped and imperative-style.

## syntax

an expression in galois can be one of the following:

- variable: variables can be "assigned to" a function or act as a wrapper around a primitive.
  variables that are just `x = y` can be "cut" as in `let x = y in`. this is non obvious initially,
  but we will have to have some "inlining" step. variables are identified by a string
- function definitions: functions are identified by a string, take an arbitrary list of arguments,
  and also contain the function body
- function applications: generally inlined in variable assignments or in other applications. straightforward.
- return statements: `return x`, may be omitted to just `x` if the last expression of a function body.
- notation declarations: identifies a pattern with a function, we will use this to build the sugar of our language.
  e.g. `z = x + y` will sugar to `z = add(x,y)`. note that we will also have to use this for `for` and `while` loops,
  so it will have to be quite powerful technology.
- foreign function interface declarations: declares the set of linkable foreign functions that can be used in the program.
  we will use these to control the effects of our otherwise purely functional language. ideally we want this to be a
  single `includes` call, but we want a better keyword, probably `use` (rust like). we will first target python as a foreign
  functions. we will probably have to make our own protocol for this if we eventually want to include other languages,
  even making it eventually language agnostic.
- foreign function calls: we can either make these explicit with a keyword like `call` or `dispatch`, or we can use the existing
  syntax (latter probably preferred). for debugging purposes, we will only have the declarations (with no linking) and represent these
  just with a `CALL` print. we will work on the hard part later.

## semantics

it is **very** important to note that _variables and functions do not have to be defined before they are used_.
this follows immediately from our goal of automatic parallelization:
if we want arbitrary parts of the program to be parallelizable, then we always want to step as much as we can
(partial applications if needed).
if we want to step as much as we can, then we need to step blocks of code that may contain "holes" in the program
where we don't have a defined assignments for a variable.

## various notes on ffi

there are many different ways we can view the denotation of our ffi backend that we want to implement.
here is the idea i am going for so far.

- variable assignments as a functor: for the purposes of our ffi, it will be infeasible to make an "object to primitive"
  and vice versa for everything. semantically we will want the variable to be "pointing" to some object from another program in
  galois or another language. this will allow us to make an "ffi" for galois itself.
- universal glue: we want galois to be a kind of universal glue between programming languages, but also completely general-purpose
  alternatively, galois is the "super" or "meta" graph connecting different computation graphs (i.e. the ffi's).
  from this we will have the ability to paralleize very easy by being resource minded. eventually we will make the language
  affine.
- similarly to the infeasibility of variable assignments, we will need a way to use actual outside code in a good way.
  this is best seen as an example. in the current implmentation in [ffi_numpy.py](../std/ffi/python/ffi_numpy.py),
  we have to predefine all the functions in numpy and wrap them. this should be easily automatable, at least for python.
  if we combine this with the functor view then it should give us the tooling? we will see.
  make sure we follow the separation of concerns here. the interpreter shouldn't have to know what the ffis are actually doing or
  which one is being used -- so it has to be general.
