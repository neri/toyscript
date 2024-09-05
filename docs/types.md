# Type System

## Builtin Types

| Name     | Bits | Size Of | Align Of | Primitive? | Similar in C99        | Description                             |
| -------- | ---- | ------- | -------- | ---------- | --------------------- | --------------------------------------- |
| `void`   | 0    | 0       | 0        | Yes        | `void`                | Empty Value                             |
| `never`  | 0    | 0       | 0        | No         | `_Noreturn`           | The Function never returns              |
| `int`    | Vary | Vary    | Vary     | No         | `int`                 | Most Useful Signed Integer Type         |
| `uint`   | Vary | Vary    | Vary     | No         | `unsigned int`        | Most Useful Unsigned Integer Type       |
| `isize`  | Vary | Vary    | Vary     | No         | `intptr_t`            | Pointer precision Signed Integer Type   |
| `usize`  | Vary | Vary    | Vary     | No         | `uintptr_t`, `size_t` | Pointer precision Unsigned Integer Type |
| `bool`   | 1?   | 1?      | 1?       | ?          | `_Bool`               | Boolean Value                           |
| `i8`     | 8    | 1       | 1        | ?          | `int8_t`              | Signed Integer Type                     |
| `u8`     | 8    | 1       | 1        | ?          | `uint8_t`             | Unsigned Integer Type                   |
| `i16`    | 16   | 2       | 2        | ?          | `int16_t`             | Signed Integer Type                     |
| `u16`    | 16   | 2       | 2        | ?          | `uint16_t`            | Unsigned Integer Type                   |
| `i32`    | 32   | 4       | 4        | Yes        | `int32_t`             | Signed Integer Type                     |
| `u32`    | 32   | 4       | 4        | ?          | `uint32_t`            | Unsigned Integer Type                   |
| `i64`    | 64   | 8       | 8        | Yes        | `int64_t`             | Signed Integer Type                     |
| `u64`    | 64   | 8       | 8        | ?          | `uint64_t`            | Unsigned Integer Type                   |
| `f32`    | 32   | 4       | 4?       | Yes        | `float`               | Floating Point Number Type              |
| `f64`    | 64   | 8       | 8?       | Yes        | `double`              | Floating Point Number Type              |
| `number` | 64   | 8       | 8        | No         | `double`              | JavaScript Number Type                  |
| `char`   | 32   | 4       | 4        | No         | `char`                | UTF-32 Character Type                   |
| `string` | Vary | Vary    | 1        | No         | `char*`               | UTF-8 String Type                       |

* The actual types of `int`, `uint`, `isize` and `usize` are determined at build time. In practice, they are treated as `i32` or `u32` in the WASM environment.

## Basic Syntax

```typescript
  declare type MyInt = int
```

### Allowed

```typescript
  const i: MyInt = 123
```

### Not Allowed

```typescript
  const i: MyInt = 123

  const j: int = i
//               ^
// Type mismatch 'MyInt', Expected 'int'

  const k: int = 123
  k + i
//    ^
// Type mismatch 'MyInt', Expected 'int'
```

## Type Inference

* Type inference is made only on the first assignment that defines a variable.
* Numeric literals are inferred as `int` types if type inference cannot be made from the context.
