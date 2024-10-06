
for (let i = 0; i <= 10; i++) {
    putchar('f')
    putchar('i')
    putchar('b')
    putchar('(')
    print_num(i)
    putchar(')')
    putchar(' ')
    putchar('=')
    putchar(' ')
    print_num(fib(i))
    putchar('\n')
}

function fib(n: int): int {
    let a = 0, b = 1
    for (let i = 0; i < n; i++) {
        let t = a + b
        a = b
        b = t
    }
    return a
}

function print_num(i: int) {
    if i < 10 {
        putchar('0' + (i as char))
    } else {
        print_num(i / 10)
        putchar('0' + (i % 10) as char)
    }
}

declare function putchar(c: char)
