# ToyScript Language Specification

## Basic Syntax

- TypeScript-like syntax

```TypeScript
export function add(a: int, b: int): int {
    return a + b
}
```

### break statement

```
'break'
```

- Interrupts execution of the innermost loop and exits outside the loop.
- Cannot be used outside of a loop.

### continue statement

```
'continue'
```

- Interrupts execution of the innermost loop and executes the next lap of the loop.
- Cannot be used outside of a loop.

### declare-function statement

```
['export'] 'declare' 'function' identifier1 '('
[identifier2 ':' type2 (',' identifier2n ':' type2n)* [','] ]
')' [':' type3]
```

- Declares a function with the given identifier and argument list. The function entity is provided by an external module or environment.
- If the `export` modifier is present, even functions not referenced from within the module will be output to binary.

### declare-type statement

```
'declare' 'type' identifer '=' type
```

- Defines an alias for a type. The new type is interconvertible with the original type, but is treated as a different type unless explicitly converted.

### function statement

```
['export'] 'function' identifier1 '('
[identifier2 ':' type2 (',' identifier2n ':' type2n)* [','] ]
')' [':' type3] '{' expr '}'
```

- Defines the function `identifier1`, with arguments of `identifier2n` and `type2n`, return type of `type3`, and contents of `expr`.
- If the `export` modifier is present, even functions not referenced from within the module will be output to binary.

### for statement

```
'for' '(' expr1 ';' expr2 ';' expr3 ')' '{' expr4 '}'
```

- After executing `expr1`, evaluate `expr2` and repeat `expr4` and `expr3` for true.
- `expr1` also permits assignment expressions with `let` statements.
- `expr2` in this form is evaluated each time for each loop.
- _Braces cannot be omitted_

### if-else statement

```
'if' expr1 '{' expr2 '}' ['else' '{' expr3 '}']
```

- Evaluates `expr1` and executes `expr2` if true, otherwise `expr3`.
- _Braces cannot be omitted_

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

### var statement (let, const)

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
