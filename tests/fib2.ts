export function fib(n: int): int {
    let a = 0, b = 1
    for (let i = 0; i < n; i++) {
        let t = a + b
        a = b
        b = t
    }
    return b
}
