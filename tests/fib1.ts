function fib(n: int): int {
    var a = 0, b = 1
    if (n > 0) {
        while (--n > 0) {
            let t = a + b
            a = b
            b = t
        }
        return b
    }
    return a
}
