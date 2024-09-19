# ToyIR

* ToyIR is an intermediate language designed for internal processing of ToyScript. NOT FOR EXTERNAL EXCHANGE.
* Most opcodes have corresponding WebAssembly instructions, but they differ significantly from raw WebAssembly in that they make stack inputs and outputs explicit.
* Variable length using an array of 32-bit words, with the first word in the same format in all opcodes.

### First word format

```
+31-16+15-----0+
| len | opcode |
+-----+--------+
```

* The value of the upper 16 bits of the first word indicates the word length of the entire instruction, and the value of the lower 16 bits indicates the opcode of the instruction.
* The specific value of the opcode is currently undetermined. They are subject to change with each build.
* Some instructions may be of indeterminate length or may have trailing padding.

### Blocks

```
[(len|opcode), label]
-> %label = block

[(len|opcode), label]
-> %label = loop

[(len|opcode), label]
-> end %label
```

* Define the start or end of a block with `%label`. 
* The defined label can be the target of a branch only within the block.
* Conform to the WebASsembly specification for block and branching behavior.

### Branches

```
[(len|opcode), label]
-> br %label

[(len|opcode), label, cond]
-> br_if %label, %cond
```

### Call

```
[(len|opcode) result, label, params, ...]
-> %result = call $label, %params, ...

[(len|opcode) dummy, label, params, ...]
-> call_v $label, %params, ...
```

* The `call_v` instruction has the same format as the `call` instruction, but the return value does not actually exist.
* Refer to the definition of the function to be called to determine which instruction should be used.

### Return

```
[(len|opcode)]
-> return

[(len|opcode) operand]
-> return %operand
```

### Constants

```
[(len|opcode), result, i32]
-> %result = i32.const $i32

[(len|opcode), result, a, b]
-> %result = i64.const (a + b)

[(len|opcode), result, f32]
-> %result = f32.const $f32

[(len|opcode), result, a, b]
-> %result = f64.const (a + b)
```

* Only these instructions, which require 64-bit values, are divided by little-endian.

### Binary Operators

```
[(len|opcode), result, lhs, rhs]
-> %result = binop %lhs, %rhs
```

### Unary Operators

```
[(len|opcode), result, operand]
-> %result = unop %operand
```

### Pseudo Unary Operators

```
[(len|opcode), result, operand]
-> %result = not %operand
-> %result = xor %operand, $-1

[(len|opcode), result, operand]
-> %result = inc %operand
-> %result = add %operand, $1

[(len|opcode), result, operand]
-> %result = dec %operand
-> %result = sub %operand, $1
```

* They are treated as unary operators on the IR, but are replaced by appropriate instructions during final code generation.

### Pseudo Opcodes

```
[(len|opcode), result, operand]
-> %result = unary_nop %operand

[(len|opcode), result, lhs, rhs]
-> %result = drop_right %lhs, %rhs

[(len|opcode), dummy, lhs, rhs]
-> drop2 %lhs, %rhs
```

* Special instructions for optimization
* The `unary_nop` instruction is a do-nothing unary operator that returns the value as is and disappears at final code generation.
* The `drop_right` instruction is like a combination of `unary_nop` and `drop`, dropping `rhs` and returning `lhs` as is.
* The `drop2` instruction drops both `lhs` and `rhs`.

### Cast

```
[(len|opcode), result, operand, new_type_id, old_type_id]
-> %result = cast %operand, $old_type as $new_type
```

* Typecasts the value of “operand”.
