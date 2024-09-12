# ToyScript Language Specification

## Basic Syntax

- TypeScript-like syntax

```TypeScript
export function add(a: int, b: int): int {
    return a + b
}
```

### for statement

```
'for' '(' expr1 ';' expr2 ';' expr3 ')' '{' expr4 '}'
```

- _Currently, not yet supported_
- After executing `expr1`, evaluate `expr2` and repeat `expr4` and `expr3` for true.
- _Braces cannot be omitted_

### function statement

```
['export'] 'function' identifier1 '('
[identifier2 ':' type1 (',' identifier2n ':' type1n)* [','] ]
')' [':' type2] '{' expr '}'
```

- Defines the function `identifier1`, with arguments of `identifier2n` and `type1n`, return type of `type2`, and contents of `expr`.

### if-else statement

```
'if' expr1 '{' expr2 '}' ['else' '{' expr3 '}']
```

- Evaluates `expr1` and executes `expr2` if true, otherwise `expr3`.
- _Braces cannot be omitted_]

### return statement

```
'return' [expr]
```

- Returns control to the caller with `expr` as the return value of the function.

### switch statement

```
'switch' expr '{' ??? '}'
```

- _Currently, not yet supported_
- _Braces cannot be omitted_

### type statement

```
'declare' 'type' identifer '=' type
```

### var statement

```
('const' | 'let') identifier [':' type] ['=' expr] (',' identifier [':' type] ['=' expr])*
```

- Defines a variable named `identifier`, which binds the type if `type` is specified, or evaluates and assigns an expression if `expr` is specified.
- Type inference is made only on the first assignment that defines a variable.

### while statement

```
'while' expr1 '{' expr2 '}'
```

- Evaluate `expr1` and repeat `expr2` while true.
- _Braces cannot be omitted_

### import statement (temp)

```
'import' 'function' identifier '('
[identifier ':' type (',' identifier ':' type)* [','] ]
')' [':' type] 'from' string
```

- Declares a function with the given identifier and argument list. The function entity is provided by an external module or environment.
- _This syntax is subject to change in the future._

## Main differences from TypeScript

### Union Type, typeof expression, ...

This is because TypeScript is a dynamic language while ToyScript is a static language.

### Type System

ToyScript requires a more strict numeric type.

### Conditional Expression Type

In ToyScript, conditional expressions must be strictly of type `boolean`.

### Classes, Enums, ...

Not yet supported

### Templates

unplanned
