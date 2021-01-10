const fs = require('fs')
const readline = require('readline')
const redux = require('redux')
const { default: PQueue } = require('p-queue')

const { stdin, stdout } = process

const logFile = fs.openSync('/tmp/lucas-logs.txt', 'w')
const log = (str) => {
  fs.writeSync(logFile, str + '\n')
}

const cursorTo = (x, y) =>
  new Promise((resolve) => {
    stdout.cursorTo(x, y, resolve)
  })

const clearScreen = () => stdout.write('\033c')

const render = async (state) => {
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

const onKeyPressNormal = (chunk, key, store) => {
  let dx = 0
  let dy = 0

  const state = store.getState()

  switch (key.name) {
    case 'h':
      store.dispatch({
        type: 'move-cursor',
        payload: {
          dx: -1,
          dy: 0,
        },
      })
      break
    case 'j':
      if (key.shift) {
        store.dispatch({
          type: 'join-line',
        })

        if (state.lines[state.cursor.y]?.length > 0) {
          store.dispatch({
            type: 'move-cursor',
            payload: {
              dx: state.lines[state.cursor.y].length - state.cursor.x,
              dy: 0,
            },
          })
        }
      } else {
        store.dispatch({
          type: 'move-cursor',
          payload: {
            dx: 0,
            dy: 1,
          },
        })
      }
      break
    case 'k':
      store.dispatch({
        type: 'move-cursor',
        payload: {
          dx: 0,
          dy: -1,
        },
      })
      break
    case 'l':
      store.dispatch({
        type: 'move-cursor',
        payload: {
          dx: 1,
          dy: 0,
        },
      })
      break
    case 'i':
      store.dispatch({
        type: 'insert-mode',
      })
      if (key.shift) {
        store.dispatch({
          type: 'move-cursor',
          payload: {
            dx: -Infinity,
            dy: 0,
          },
        })
      }
      break
    case 'o':
      store.dispatch({
        type: 'insert-line',
        payload: {
          y: state.cursor.y + (key.shift ? 0 : 1),
        },
      })

      store.dispatch({
        type: 'insert-mode',
      })

      store.dispatch({
        type: 'move-cursor',
        payload: {
          dx: -Infinity,
          dy: key.shift ? 0 : 1,
        },
      })
      break
    case 'x':
      store.dispatch({
        type: 'remove-char',
      })
      break
  }
}

const INSERT_MODE_IGNORED_KEYS = new Set(['backspace'])
const onKeyPressInsert = (chunk, key, store) => {
  const state = store.getState()

  if (INSERT_MODE_IGNORED_KEYS.has(key.name)) {
    return
  }

  if (key.name === 'escape') {
    store.dispatch({
      type: 'normal-mode',
    })
    return
  }

  if (key.name === 'return') {
    store.dispatch({
      type: 'return',
    })
  } else {
    const input = chunk?.toString() ?? ''
    if (input.length > 0) {
      store.dispatch({
        type: 'insert-input',
        payload: {
          input,
        },
      })
      store.dispatch({
        type: 'move-cursor',
        payload: {
          dx: 1,
          dy: 0,
        },
      })
    }
  }
}

const onKeyPress = (store) => (chunk, key) => {
  const state = store.getState()

  log(JSON.stringify({ key, chunk }))
  if (key.ctrl && key.name === 'c') {
    process.exit(0)
  }

  switch (state.mode) {
    case 'normal':
      onKeyPressNormal(chunk, key, store)
      break
    case 'insert':
      onKeyPressInsert(chunk, key, store)
      break
  }
}

const reducer = (state, action) => {
  if (action.type === 'insert-mode') {
    return {
      ...state,
      mode: 'insert',
    }
  }

  if (action.type === 'insert-input') {
    const chars = state.lines[state.cursor.y].split('')
    const { input } = action.payload

    const lines = state.lines.slice(0)
    lines[state.cursor.y] = [
      ...chars.slice(0, state.cursor.x),
      input,
      ...chars.slice(state.cursor.x),
    ].join('')

    return {
      ...state,
      lines,
    }
  }

  if (action.type === 'move-cursor') {
    const { dx, dy } = action.payload
    let { x, y } = state.cursor
    x = Math.min(stdout.columns, Math.max(0, x + dx))
    y = Math.min(state.lines.length - 1, Math.max(0, y + dy))

    if (state.lines[y] !== undefined) {
      x = Math.min(state.lines[y].length, x)
    }

    return {
      ...state,
      cursor: {
        x,
        y,
      },
    }
  }

  if (action.type === 'insert-line') {
    const { y } = action.payload

    return {
      ...state,
      lines: [...state.lines.slice(0, y), '', ...state.lines.slice(y)],
    }
  }

  if (action.type === 'return') {
    const y = state.cursor.y
    let currentLine = state.lines[y]
    let newLineContent = ''
    if (currentLine !== undefined) {
      newLineContent = currentLine.slice(state.cursor.x)
      currentLine = currentLine.slice(0, state.cursor.x)
    }

    return {
      ...state,
      cursor: {
        x: 0,
        y: y + 1,
      },
      lines: [
        ...state.lines.slice(0, y),
        currentLine,
        newLineContent,
        ...state.lines.slice(y + 1),
      ],
    }
  }

  if (action.type === 'normal-mode') {
    return {
      ...state,
      mode: 'normal',
    }
  }

  if (action.type === 'remove-char') {
    const chars = state.lines[state.cursor.y].split('')
    const lines = state.lines.slice(0)
    lines[state.cursor.y] = [
      ...chars.slice(0, state.cursor.x),
      ...chars.slice(state.cursor.x + 1),
    ].join('')

    return {
      ...state,
      lines,
    }
  }

  if (action.type === 'join-line') {
    const lineA = state.lines[state.cursor.y]
    const lineB = state.lines[state.cursor.y + 1]
    if (lineA === undefined || lineB === undefined) {
      return state
    }

    if (lineA !== undefined && lineB !== undefined) {
      let newLine = lineA

      if (newLine !== '' && newLine[newLine.length - 1] !== ' ') {
        newLine += ' '
      }

      newLine += lineB

      return {
        ...state,
        lines: [
          ...state.lines.slice(0, state.cursor.y),
          newLine,
          ...state.lines.slice(state.cursor.y + 2),
        ],
      }
    }

    return state
  }

  log(`UNHANDLED ACTION ${action.type}`)

  return state
}

const logger = ({ getState }) => (next) => (action) => {
  log('WILL DISPATCH')
  log(JSON.stringify(action))

  // Call the next dispatch method in the middleware chain.
  const returnValue = next(action)

  log('STATE AFTER DISPATCH')
  log(JSON.stringify(getState()))

  // This will likely be the action itself, unless
  // a middleware further in chain changed it.
  return returnValue
}

const main = () => {
  readline.emitKeypressEvents(process.stdin, {
    escapeCodeTimeout: 0,
  })

  const store = redux.createStore(
    reducer,
    {
      line: 0,
      mode: 'normal',
      lines: [''],
      cursor: {
        x: 0,
        y: 0,
      },
    },
    redux.applyMiddleware(logger)
  )

  stdin.on('keypress', onKeyPress(store))
  stdin.setRawMode(true)

  const renderQueue = new PQueue({
    concurrency: 1,
  })

  store.subscribe(() => {
    renderQueue.add(() => render(store.getState()))
  })

  render(store.getState())
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
