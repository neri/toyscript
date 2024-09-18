
declare function putchar(c: char)

function main() {
  for (let i = ' '; i < 127; i++) {
    putchar(i)
  }
}
