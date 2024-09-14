
import function putchar(c: char) from "env"

function main() {
  for (let i = ' '; i <'\7F'; i++) {
    putchar(i)
  }
}
