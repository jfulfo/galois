# Galois Design

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

```
Large language models (LLMs) have shown impressive results at a wide-range of tasks.
However, they have limitations, such as hallucinating facts and struggling with arithmetic.
Recent work has addressed these issues with sophisticated decoding techniques.
However, performant decoding, particularly for sophisticated techniques, relies crucially on parallelization and batching, which are difficult for developers.
We make two observations:
  1) existing approaches are high-level domain-specific languages for gluing expensive black-box calls, but are not general or compositional;
  2) LLM programs are essentially pure (all effects commute).
Guided by these observations, we develop a novel, general-purpose lambda calculus for automatically parallelizing a wide-range of LLM interactions, without user intervention.
The key difference versus standard lambda calculus is a novel "opportunistic" evaluation strategy, which steps independent parts of a program in parallel, dispatching black-box external calls as eagerly as possible, even while data-independent parts of the program are waiting for their own external calls to return.
To maintain the simplicity of the language and to ensure uniformity of opportunistic evaluation, control-flow and looping constructs are implemented in-language, via Church encodings.
```

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
