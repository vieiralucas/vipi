const fs = require('fs')
const readline = require('readline')
const path = require('path')
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

const currentLineIndex = (state) => state.cursor.y + state.yOffset

const currentLine = (state) => state.lines[currentLineIndex(state)]

const render = async (state) => {
  clearScreen()

  await cursorTo(0, stdout.rows - 1)
  switch (state.mode) {
    case 'insert':
      stdout.write('I')
      break
    case 'command':
      stdout.write(`:${state.command.input}`)
      break
  }

  await cursorTo(0, stdout.rows - 2)
  stdout.write(
    `[No Name] - ${state.cursor.x + 1}, ${currentLineIndex(state) + 1}`
  )

  const linesForFile = stdout.rows - 2
  const lines = state.lines.slice(state.yOffset, state.yOffset + linesForFile)

  for (let i = 0; i < lines.length; i++) {
    await cursorTo(0, i)
    stdout.write(lines[i])
  }

  if (state.mode === 'command') {
    await cursorTo(state.command.cursor + 1, stdout.rows - 1)
  } else {
    await cursorTo(state.cursor.x, state.cursor.y)
  }
}

const onKeyPressNormal = async (chunk, key, store) => {
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

        if (currentLine(state)?.length > 0) {
          store.dispatch({
            type: 'move-cursor',
            payload: {
              dx: currentLine(state).length - state.cursor.x,
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
          y: currentLineIndex(state) + (key.shift ? 0 : 1),
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

  switch (key.sequence) {
    case ':':
      store.dispatch({
        type: 'command-mode',
      })
      break
  }
}

const INSERT_MODE_IGNORED_KEYS = new Set(['backspace'])
const onKeyPressInsert = async (chunk, key, store) => {
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
      type: 'split-line',
      payload: {
        lineIndex: currentLineIndex(state),
        x: state.cursor.x,
      },
    })
    store.dispatch({
      type: 'move-cursor',
      payload: {
        dx: -Infinity,
        dy: 1,
      },
    })
    return
  }

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

const saveFile = async (store, filename) => {
  if (!filename) {
    // TODO: Display error message
    return
  }

  const filepath = path.resolve(process.cwd(), filename)
  const content = store.getState().lines.join('\n')

  // TODO: should this be async?
  fs.writeFileSync(filepath, content)
}

const executeCommand = async (store) => {
  const state = store.getState()
  const [command, ...args] = state.command.input.split(' ')
  switch (command) {
    case 'q!':
      process.exit(0)
      break
    case 'w':
      await saveFile(store, args[0])
      break
  }

  store.dispatch({
    type: 'normal-mode',
  })
}

const onKeyPressCommand = async (chunk, key, store) => {
  if (key.name === 'escape') {
    store.dispatch({
      type: 'normal-mode',
    })
    return
  }

  if (key.name === 'return') {
    await executeCommand(store)
    return
  }

  const state = store.getState()
  if (key.name === 'backspace') {
    store.dispatch({
      type: 'command-input',
      payload: {
        input: state.command.input.slice(0, -1),
      },
    })
    return
  }

  const input = chunk?.toString() ?? ''
  store.dispatch({
    type: 'command-input',
    payload: {
      input: state.command.input + input,
    },
  })
}

const onKeyPress = async (chunk, key, store) => {
  const state = store.getState()

  log(JSON.stringify({ key, chunk, mode: state.mode }))

  switch (state.mode) {
    case 'normal':
      await onKeyPressNormal(chunk, key, store)
      break
    case 'insert':
      await onKeyPressInsert(chunk, key, store)
      break
    case 'command':
      await onKeyPressCommand(chunk, key, store)
      break
  }
}

const reducer = (state, action) => {
  if (action.type === 'command-mode') {
    return {
      ...state,
      mode: 'command',
      command: {
        input: '',
        cursor: 0,
      },
    }
  }

  if (action.type === 'command-input') {
    const input = action.payload.input
    return {
      ...state,
      command: {
        input,
        cursor: input.length,
      },
    }
  }

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
    let yOffset = state.yOffset
    let { x, y } = state.cursor

    const line = state.lines[y + dy + yOffset]
    const hasLine = line !== undefined

    if (y + dy < 0) {
      yOffset = Math.max(0, yOffset + dy)
    } else if (hasLine && y + dy > stdout.rows - 3) {
      yOffset += dy
    }

    x = Math.min(stdout.columns, Math.max(0, x + dx))
    if (hasLine) {
      y = Math.min(stdout.rows - 3, Math.max(0, y + dy))
    }

    if (hasLine) {
      x = Math.min(line.length, x)
    }

    return {
      ...state,
      yOffset,
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

  if (action.type === 'split-line') {
    const { x, lineIndex } = action.payload
    let previousLine = state.lines[lineIndex]
    const newLine = previousLine.slice(x)
    previousLine = previousLine.slice(0, x)

    return {
      ...state,
      lines: [
        ...state.lines.slice(0, lineIndex),
        previousLine,
        newLine,
        ...state.lines.slice(lineIndex + 1),
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
    const chars = currentLine(state).split('')
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
    const index = currentLineIndex(state)
    const lineA = currentLine(state)
    const lineB = state.lines[index + 1]
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
          ...state.lines.slice(0, index),
          newLine,
          ...state.lines.slice(index + 2),
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
      yOffset: 0,
      mode: 'normal',
      command: {
        input: '',
        cursor: 0,
      },
      lines: new Array(40).fill('').map((_, i) => (i + 1).toString()),
      cursor: {
        x: 0,
        y: 0,
      },
    },
    redux.applyMiddleware(logger)
  )

  const keyPressQueue = new PQueue({
    concurrency: 1,
  })

  stdin.on('keypress', (chunk, key) => {
    keyPressQueue.add(() => onKeyPress(chunk, key, store))
  })
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
