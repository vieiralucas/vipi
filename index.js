const tty = require('tty')

const state = {
  line: 0,
  mode: 'normal',
  content: ['Hello world', 'Another line'].join('\n'),
}

const { stdin, stdout } = process

// stdout.clearScreenDown(() => {
//   stdout.cursorTo(0, stdout.columns, () => {
//     stdout.write('N')
//     stdout.cursorTo(0, 0)
//   })
// })

const cursorTo = (x, y) =>
  new Promise((resolve) => {
    stdout.cursorTo(x, y, resolve)
  })

const clearScreen = () => stdout.write('\033c')

const render = async (state) => {
  clearScreen()

  await cursorTo(0, stdout.columns)
  stdout.write('N')

  const linesForFile = stdout.columns - 1
  const lines = state.content.split('\n').slice(0, linesForFile)

  for (let i = 0; i < lines.length; i++) {
    await cursorTo(0, i)
    stdout.write(lines[i])
  }

  await cursorTo(0, 0)
}

stdin.setRawMode(true)

stdin.on('data', (chunk) => {
  let dx = 0
  let dy = 0
  const input = chunk.toString()
  switch (input) {
    case 'h':
      dx = -1
      break
    case 'j':
      dy = 1
      break
    case 'k':
      dy = -1
      break
    case 'l':
      dx = 1
  }

  stdout.moveCursor(dx, dy)
})

render(state)

//
// console.log(stdout.isTTY)
//
// let columns = stdout.columns
//
// stdout.on('resize', () => {
// })
//
// console.log(process.stdout)
//
setTimeout(() => {
  process.exit(0)
}, 10000)
