// chars

import function putchar(c: int) from "env"

function main() {
  for (let i = 0x20; i < 0x7F; i++) {
    putchar(i)
  }
}

