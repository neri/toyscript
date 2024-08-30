;; hello world
(module
  (type (;0;) (func (param i32 i32)))
  (type (;1;) (func))

  (import "env" "println" (func $println (param i32) (param i32)))

  (memory 1)

  (data (i32.const 16) "hello world!")

  (func $main (export "main")
    (local $a i32)
    i32.const 12
    i32.const 16
    call $println
  )
)
