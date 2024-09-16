# Type System

## Builtin Types

| Name      | Bits | Size Of | WASM binding | Similar in C99        | Description                             |
| --------- | ---- | ------- | ------------ | --------------------- | --------------------------------------- |
| `void`    | 0    | 0       | -            | `void`                | Empty Value                             |
| `never`   | 0    | 0       | -            | `_Noreturn`           | The Function never returns              |
| `int`     | Vary | Vary    | `i32`        | `int`                 | Most Useful Signed Integer Type         |
| `uint`    | Vary | Vary    | `i32`        | `unsigned int`        | Most Useful Unsigned Integer Type       |
| `isize`   | Vary | Vary    | `i32`        | `intptr_t`            | Pointer precision Signed Integer Type   |
| `usize`   | Vary | Vary    | `i32`        | `uintptr_t`, `size_t` | Pointer precision Unsigned Integer Type |
| `boolean` | 1?   | 1?      | `i32`        | `_Bool`               | Boolean Value                           |
| `i8`      | 8    | 1       | `i32`        | `int8_t`              | Signed Integer Type                     |
| `u8`      | 8    | 1       | `i32`        | `uint8_t`             | Unsigned Integer Type                   |
| `i16`     | 16   | 2       | `i32`        | `int16_t`             | Signed Integer Type                     |
| `u16`     | 16   | 2       | `i32`        | `uint16_t`            | Unsigned Integer Type                   |
| `i32`     | 32   | 4       | `i32`        | `int32_t`             | Signed Integer Type                     |
| `u32`     | 32   | 4       | `i32`        | `uint32_t`            | Unsigned Integer Type                   |
| `i64`     | 64   | 8       | `i64`        | `int64_t`             | Signed Integer Type                     |
| `u64`     | 64   | 8       | `i64`        | `uint64_t`            | Unsigned Integer Type                   |
| `f32`     | 32   | 4       | `f32`        | `float`               | Floating Point Number Type              |
| `f64`     | 64   | 8       | `f64`        | `double`              | Floating Point Number Type              |
| `number`  | 64   | 8       | `f64`        | `double`              | JavaScript Number Type                  |
| `char`    | 32   | 4       | `i32`        | `char`                | UTF-32 Character Type                   |
| `string`  | Vary | Vary    | -            | `char*`               | UTF-8 String Type                       |

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
