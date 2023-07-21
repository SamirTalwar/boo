# Boo

A little programming language, designed to be embedded inside another one, and
extended.

Currently it does neither of these things, and is simply a toy.

## Getting up and running

The main executable is the interpreter. You can run it directly for a REPL
interface:

```
$ cargo run --quiet
〉2 + 3
5
〉
```

Or you can pipe in a program to be run. For example:

```
$ echo '2 + 3' | cargo run --quiet
5
```

## Functionality

Boo is a lazy, purely-functional programming language that looks somewhat like
Haskell.

### Integers

Boo supports arbitrary-precision integers, which are expressed in decimal, e.g.
`123`, `-9`, or `0`. You can use underscores for readability, e.g. `1_000_000`.

You can add numbers with `+`, subtract them with `-`, and multiply with `*`.
Multiplication takes precedence. For example, `9 + 5 * 3 - 4` will result in
`20`.

You can use parentheses (`(` and `)`) to change precedence. For example:

```
〉(9 + 5) * (3 - 4)
-14
```

Numbers can keep growing, bounded only by available memory. for example:

```
〉1_000_000_000_000_000_000 * 1_000_000_000_000_000_000
1000000000000000000000000000000000000
```

Other numeric types, such as floating-point values or rational numbers, are not
supported.

### Assignment

You can assign variables with the `let ... in` keyword pair. For example:

```
〉let width = 9 in width * width
81
```

To assign multiple variables, you need to nest `let` expressions.

```
〉let width = 9 in let height = 7 in width * height
63
```

Programs are currently a single expression. There is no way to assign a
variable in one line on the REPL, and then use it in a subsequent line.

### Functions

A function is defined with the `fn` keyword. A function accepts a single
argument and returns a value.

`fn x -> x * 2` is a function that doubles its argument.

Functions are applied by placing another expression (the argument) to the
right:

```
〉(fn x -> x * 2) 7
14
```

You can name functions for later use with `let`:

```
〉let double = fn x -> x * 2 in double 7
14
```

Functions can take one or more parameters:

```
let add = fn x y -> x + y in add 2 3
```
