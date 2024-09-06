# ToyScript Language Specification

## Basic Syntax

- TypeScript-like syntax

```TypeScript
export function add(a: int, b: int): int {
    return a + b
}
```

### for statement

`for` ??? `{` expr `}`

- _Currently, not yet supported_
- _Braces cannot be omitted_

### function statement

[`export`] `function` identifier `(` [identifier `:` type (`,` identifier `:` type )* [`,`] ] `)` [`:` type] `{` expr `}`

### if-else statement

`if` expr `{` expr `}` [`else` `{` expr `}`]

- _Braces cannot be omitted_

### return statement

`return` expr

### switch statement

`switch` expr `{` ??? `}`

- _Currently, not yet supported_
- _Braces cannot be omitted_

### type statement

`declare` `type` identifer `=` type

### var statement

(`const` | `let` | `var`) identifier [`:` type] [`=` expr]

- Currently there is no difference between `let` and `var`.
- Type inference is made only on the first assignment that defines a variable.

### while statement

`while` expr `{` expr `}`

- _Braces cannot be omitted_

## Main differences from TypeScript

### Union Type, typeof expression, ...

This is because TypeScript is a dynamic language while ToyScript is a static language.

### Type System

ToyScript requires a more strict numeric type.

### Conditional Expression Type

In ToyScript, conditional expressions must be strictly of type `bool`.

### Classes, Enums, ...

Not yet supported

### Templates

unplanned
