const fs = require('fs')
const readline = require('readline')

const { stdin, stdout } = process

const state = {
  line: 0,
  mode: 'normal',
  lines: [''],
  cursor: {
    x: 0,
    y: 0,
  },
}

const logFile = fs.openSync('/tmp/lucas-logs.txt', 'w')
const log = (str) => {
  fs.writeSync(logFile, str + '\n')
}

const cursorTo = (x, y) =>
  new Promise((resolve) => {
    stdout.cursorTo(x, y, resolve)
  })

const clearScreen = () => stdout.write('\033c')

const render = async () => {
  clearScreen()

  await cursorTo(0, stdout.rows - 1)
  switch (state.mode) {
    case 'insert':
      stdout.write('I')
      break
  }

  await cursorTo(0, stdout.rows - 2)
  stdout.write(`[No Name] - ${state.cursor.x}, ${state.cursor.y}`)

  const linesForFile = stdout.rows - 2
  const lines = state.lines.slice(0, linesForFile)

  for (let i = 0; i < lines.length; i++) {
    await cursorTo(0, i)
    stdout.write(lines[i])
  }

  await cursorTo(state.cursor.x, state.cursor.y)
}

const moveCursor = (dx, dy) => {
  state.cursor.x = Math.min(stdout.columns, Math.max(0, state.cursor.x + dx))
  state.cursor.y = Math.min(
    state.lines.length - 1,
    Math.max(0, state.cursor.y + dy)
  )

  if (state.lines[state.cursor.y] !== undefined) {
    state.cursor.x = Math.min(
      state.lines[state.cursor.y].length,
      state.cursor.x
    )
  }
}

const insertLine = (y) => {
  state.lines = [...state.lines.slice(0, y), '', ...state.lines.slice(y)]
}

const onKeyPressNormal = (chunk, key) => {
  let dx = 0
  let dy = 0

  switch (key.name) {
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
      break
    case 'i':
      state.mode = 'insert'
      if (key.shift) {
        dx = -Infinity
      }
      break
    case 'o':
      dy = key.shift ? 0 : 1
      insertLine(state.cursor.y + dy)
      state.mode = 'insert'
      break
    case 'x':
      const chars = state.lines[state.cursor.y].split('')
      state.lines[state.cursor.y] = [
        ...chars.slice(0, state.cursor.x),
        ...chars.slice(state.cursor.x + 1),
      ].join('')
      break
  }

  moveCursor(dx, dy)
}

const INSERT_MODE_IGNORED_KEYS = new Set(['backspace'])
const onKeyPressInsert = (chunk, key) => {
  let dx = 0
  let dy = 0

  if (INSERT_MODE_IGNORED_KEYS.has(key.name)) {
    return
  }

  if (key.name === 'escape') {
    state.mode = 'normal'
    return
  }

  if (key.name === 'return') {
    insertLine(state.cursor.y + 1)
    dy = 1
  } else {
    const input = chunk?.toString() ?? ''
    if (input.length > 0) {
      const chars = state.lines[state.cursor.y].split('')
      state.lines[state.cursor.y] = [
        ...chars.slice(0, state.cursor.x),
        input,
        ...chars.slice(state.cursor.x),
      ].join('')
      dx = 1
    }
  }

  moveCursor(dx, dy)
}

const onKeyPress = (chunk, key) => {
  log(JSON.stringify({ key, chunk }))
  if (key.ctrl && key.name === 'c') {
    process.exit(0)
  }

  switch (state.mode) {
    case 'normal':
      onKeyPressNormal(chunk, key)
      break
    case 'insert':
      onKeyPressInsert(chunk, key)
      break
  }

  render()
}

const main = () => {
  readline.emitKeypressEvents(process.stdin, {
    escapeCodeTimeout: 0,
  })

  // keypress(stdin)
  stdin.setRawMode(true)

  stdin.on('keypress', onKeyPress)
  stdin.setRawMode(true)
  render()
}

main()

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
// setTimeout(() => {
//   process.exit(0)
// }, 10000)
