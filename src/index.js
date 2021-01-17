#!/usr/bin/env node

const fs = require('fs')
const readline = require('readline')
const path = require('path')
const redux = require('redux')
const { default: PQueue } = require('p-queue')

const buffer = require('./buffer')
const vec = require('./vector')

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
    case 'command':
      stdout.write(`${state.command.prefix}${state.command.input}`)
      break
  }

  await cursorTo(0, stdout.rows - 2)
  const fileName = state.buffer.filepath
    ? path.basename(state.buffer.filepath)
    : '[No Name]'
  stdout.write(
    `${fileName} - ${state.buffer.cursor.y + 1}, ${state.buffer.cursor.x + 1}`
  )

  const lines = buffer.linesToRender(stdout.rows - 2, state.buffer)

  for (let i = 0; i < lines.length; i++) {
    await cursorTo(0, i)
    stdout.write(lines[i])
  }

  if (state.mode === 'command') {
    await cursorTo(state.command.cursor + 1, stdout.rows - 1)
  } else {
    const { x, y } = buffer.screenCursor(state.buffer)
    await cursorTo(x, y)
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
          delta: {
            x: -1,
            y: 0,
          },
        },
      })
      break
    case 'j':
      if (key.shift) {
        store.dispatch({
          type: 'join-line',
        })
      } else {
        store.dispatch({
          type: 'move-cursor',
          payload: {
            delta: {
              x: 0,
              y: 1,
            },
          },
        })
      }
      break
    case 'k':
      store.dispatch({
        type: 'move-cursor',
        payload: {
          delta: {
            x: 0,
            y: -1,
          },
        },
      })
      break
    case 'l':
      store.dispatch({
        type: 'move-cursor',
        payload: {
          delta: {
            x: 1,
            y: 0,
          },
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
            delta: {
              x: -Infinity,
              y: 0,
            },
          },
        })
      }
      break
    case 'o':
      store.dispatch({
        type: 'insert-line',
        payload: {
          above: key.shift,
        },
      })

      store.dispatch({
        type: 'insert-mode',
      })

      store.dispatch({
        type: 'move-cursor',
        payload: {
          delta: {
            x: -Infinity,
            y: key.shift ? 0 : 1,
          },
        },
      })
      break
    case 'x':
      store.dispatch({
        type: 'remove-char',
      })
      break
    case '0':
      store.dispatch({
        type: 'move-cursor',
        payload: {
          delta: {
            x: -Infinity,
            y: 0,
          },
        },
      })
      break
    case 'g':
      if (key.shift) {
        store.dispatch({
          type: 'move-cursor',
          payload: {
            delta: {
              x: 0,
              y: Infinity,
            },
          },
        })
      }
      break
    case 'w':
      store.dispatch({
        type: 'words-motion',
        payload: {
          direction: 'forward',
          position: 'start',
        },
      })
      break
    case 'e':
      store.dispatch({
        type: 'words-motion',
        payload: {
          direction: 'forward',
          position: 'end',
        },
      })
      break
    case 'b':
      store.dispatch({
        type: 'words-motion',
        payload: {
          direction: 'backward',
          position: 'start',
        },
      })
      break
  }

  if (key.ctrl) {
    switch (key.name) {
      case 'd':
        store.dispatch({
          type: 'scroll-screen',
          payload: {
            dy: Math.ceil((stdout.rows - 3) / 2),
          },
        })
        break
      case 'u':
        store.dispatch({
          type: 'scroll-screen',
          payload: {
            dy: Math.ceil((stdout.rows - 3) / 2) * -1,
          },
        })
        break
    }
    return
  }

  switch (key.sequence) {
    case '$':
      store.dispatch({
        type: 'move-cursor',
        payload: {
          delta: {
            x: Infinity,
            y: 0,
          },
        },
      })
      break
    case ':':
    case '/':
      store.dispatch({
        type: 'command-mode',
        payload: {
          prefix: key.sequence,
        },
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
    })
    store.dispatch({
      type: 'move-cursor',
      payload: {
        delta: {
          x: -Infinity,
          y: 1,
        },
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
        delta: {
          x: 1,
          y: 0,
        },
      },
    })
  }
}

const saveFile = async (store, filename) => {
  store.dispatch({
    type: 'set-buffer',
    payload: {
      buffer: buffer.save(store.getState().buffer, filename),
    },
  })
}

const readFile = async (store, filename) => {
  if (!filename) {
    // TODO: Display error message
    return
  }

  store.dispatch({
    type: 'set-buffer',
    payload: {
      buffer: buffer.fromFile(filename),
    },
  })
}

const findInLines = (text, lines) => {
  return lines
    .map((line, i) => {
      const x = line.indexOf(text)
      if (x >= 0) {
        return { x, y: i }
      }

      return null
    })
    .find((position) => position !== null)
}

const search = (text, store) => {
  log(`SEARCH ${text}`)
  const state = store.getState()
  const position = buffer.search(text, state.buffer)
  if (position) {
    store.dispatch({
      type: 'move-cursor',
      payload: {
        delta: vec.sub(position, state.buffer.cursor),
      },
    })
  }
}

const executeCommand = async (store) => {
  const state = store.getState()
  const [command, ...args] = state.command.input.split(' ')

  if (state.command.prefix === '/') {
    const searchText = state.command.input
    if (searchText !== '') {
      search(searchText, store)
    }
  } else {
    switch (command) {
      case 'q!':
        process.exit(0)
        break
      case 'w':
        await saveFile(store, args[0])
        break
      case 'e':
        await readFile(store, args[0])
        break
    }
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
  if (key.ctrl && key.name === 'c') {
    process.exit(0)
  }

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
  if (action.type === 'set-buffer') {
    return {
      ...state,
      buffer: action.payload.buffer,
    }
  }

  if (action.type === 'command-mode') {
    return {
      ...state,
      mode: 'command',
      command: {
        input: '',
        cursor: 0,
        prefix: action.payload.prefix,
      },
    }
  }

  if (action.type === 'command-input') {
    const input = action.payload.input
    return {
      ...state,
      command: {
        ...state.command,
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
    return {
      ...state,
      buffer: buffer.insertStr(action.payload.input, state.buffer),
    }
  }

  if (action.type === 'move-cursor') {
    return {
      ...state,
      buffer: buffer.move(action.payload.delta, stdout.rows - 3, state.buffer),
    }
  }

  if (action.type === 'words-motion') {
    if (action.payload.direction === 'forward') {
      let pos = buffer.nextWord(state.buffer)
      if (pos) {
        if (action.payload.position === 'end') {
          const line = state.buffer.lines[pos.y]
          const slice = line.slice(pos.x)
          const nextWhite = slice.match(/\s|$/)?.index ?? 0

          pos = vec.setX(pos, pos.x + nextWhite - 1)
        }

        return {
          ...state,
          buffer: buffer.move(
            vec.sub(pos, state.buffer.cursor),
            stdout.rows - 3,
            state.buffer
          ),
        }
      }
    }

    if (action.payload.direction === 'backward') {
      const pos = buffer.previousWord(state.buffer)
      return {
        ...state,
        buffer: buffer.move(
          vec.sub(pos, state.buffer.cursor),
          stdout.rows - 3,
          state.buffer
        ),
      }
    }
    return state
  }

  if (action.type === 'scroll-screen') {
    return {
      ...state,
      buffer: buffer.scrollScreen(
        action.payload.dy,
        stdout.rows - 3,
        state.buffer
      ),
    }
  }

  if (action.type === 'insert-line') {
    const { above } = action.payload

    return {
      ...state,
      buffer: buffer.insertLine(above, state.buffer),
    }
  }

  if (action.type === 'split-line') {
    return {
      ...state,
      buffer: buffer.splitLine(state.buffer),
    }
  }

  if (action.type === 'normal-mode') {
    return {
      ...state,
      mode: 'normal',
    }
  }

  if (action.type === 'remove-char') {
    return {
      ...state,
      buffer: buffer.removeChar(state.buffer),
    }
  }

  if (action.type === 'join-line') {
    return {
      ...state,
      buffer: buffer.joinLine(stdout.rows - 3, state.buffer),
    }
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
  const file = process.argv[2]

  readline.emitKeypressEvents(process.stdin, {
    escapeCodeTimeout: 0,
  })

  const store = redux.createStore(
    reducer,
    {
      mode: 'normal',
      command: {
        input: '',
        cursor: 0,
        prefix: ':',
      },
      buffer: file ? buffer.fromFile(file) : buffer.empty(),
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
